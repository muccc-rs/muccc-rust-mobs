use bevy::prelude::ops::powf;
use bevy::prelude::ops::sqrt;
use bevy::{audio::PlaybackMode, prelude::*};
use rand::Rng;

const GRAVITY: f32 = 420.69;
const VERTICAL_GAP_SIZE: f32 = 200.0;
const HORIZONTAL_GAP_TIME: f32 = 3.5;

// ADR:
// 2 options
// 1st move coordinate system of world left
// 2nd move crab right, move camera left, ...
// Choose 1st

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    InitGame,
    InGame,
    DeathScreen,
    Paused,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .insert_resource(PipeTimer(Timer::from_seconds(
            HORIZONTAL_GAP_TIME,
            TimerMode::Repeating,
        )))
        .insert_resource(ScoreBoard { passed_pipes: 0 })
        // Add onInitGame systems
        .add_systems(OnEnter(GameState::InitGame), reinit_game)
        .add_systems(OnEnter(GameState::DeathScreen), show_game_over_screen)
        .add_systems(OnExit(GameState::DeathScreen), despawn_death_screen)
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                advance_physics,
                move_pipe,
                spawn_pipes,
                update_score,
                check_collision,
                replace_player,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            // The `RunFixedMainLoop` schedule allows us to schedule systems to run before and
            // after the fixed timestep loop.
            RunFixedMainLoop,
            (
                // The physics simulation needs to know the player's input, so we run this before
                // the fixed timestep loop. Note that if we ran it in `Update`, it would be too
                // late, as the physics simulation would already have been advanced. If we ran this
                // in `FixedUpdate`, it would sometimes not register player input, as that schedule
                // may run zero times per frame.
                handle_input
                    .in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop)
                    .run_if(in_state(GameState::InGame)),
                handle_input_unpause
                    .in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop)
                    .run_if(in_state(GameState::Paused)),
                handle_input_when_dead
                    .in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop)
                    .run_if(in_state(GameState::DeathScreen)),
                // The player's visual representation needs to be updated after the physics
                // simulation has been advanced. This could be run in `Update`, but if we run it
                // here instead, the systems in `Update` will be working with the `Transform` that
                // will actually be shown on screen.
                interpolate_rendered_transform
                    .in_set(RunFixedMainLoopSystem::AfterFixedMainLoop)
                    .run_if(in_state(GameState::InGame)),
            ),
        )
        .run();
}

/// A vector representing the player's velocity in the physics simulation.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct Velocity(Vec3);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
struct Pipe;
#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
struct PipeStack;
#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
struct ScoreText;
#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
struct Player;
#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
struct DeathScreen;

/// The actual position of the player in the physics simulation.
/// This is separate from the `Transform`, which is merely a visual representation.
///
/// If you want to make sure that this component is always initialized
/// with the same value as the `Transform`'s translation, you can
/// use a [component lifecycle hook](https://docs.rs/bevy/0.14.0/bevy/ecs/component/struct.ComponentHooks.html)
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct PhysicalTranslation(Vec3);

/// The value [`PhysicalTranslation`] had in the last fixed timestep.
/// Used for interpolation in the `interpolate_rendered_transform` system.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct PreviousPhysicalTranslation(Vec3);

#[derive(Resource)]
struct PipeTimer(Timer);

#[derive(Resource)]
struct ScoreBoard {
    passed_pipes: u32,
}

fn create_player(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.spawn((
        Player,
        Sprite {
            image: asset_server.load("gentleman-ferris-transparent.png"),
            custom_size: Some(Vec2::new(100., 100.)),
            ..Default::default()
        },
        Velocity::default(),
        PhysicalTranslation::default(),
        PreviousPhysicalTranslation::default(),
    ));
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut timer: ResMut<PipeTimer>) {
    let projection = OrthographicProjection {
        scaling_mode: bevy::render::camera::ScalingMode::FixedVertical {
            viewport_height: 1000.0,
        },
        ..OrthographicProjection::default_2d()
    };

    commands.spawn((Camera2d, Projection::Orthographic(projection)));

    commands.spawn((
        ScoreText,
        Text("123".to_owned()),
        TextFont::from_font(asset_server.load("ComicNeue-Regular.ttf")).with_font_size(100.0),
        //TextLayout::new(JustifyText::Left, linebreak),
        //BackgroundColor(Color::srgb(0.8 - j as f32 * 0.2, 0., 0.)),
        Node {
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    commands.spawn((
        AudioPlayer::new(asset_server.load("crabrave.ogg")),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..default()
        },
    ));

    timer.0.tick(std::time::Duration::from_secs_f32(1.8));
}

/**
 * Calculation of new velocity and stuff
 */
fn advance_physics(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut PhysicalTranslation,
        &mut PreviousPhysicalTranslation,
        &mut Velocity,
    )>,
) {
    for (mut current_physical_translation, mut previous_physical_translation, mut velocity) in
        query.iter_mut()
    {
        previous_physical_translation.0 = current_physical_translation.0;
        current_physical_translation.0 += velocity.0 * fixed_time.delta_secs();
        velocity.0 += Vec3::new(0., -GRAVITY, 0.) * fixed_time.delta_secs();
    }
}

fn interpolate_rendered_transform(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut Transform,
        &PhysicalTranslation,
        &PreviousPhysicalTranslation,
    )>,
) {
    for (mut transform, current_physical_translation, previous_physical_translation) in
        query.iter_mut()
    {
        let previous = previous_physical_translation.0;
        let current = current_physical_translation.0;
        // The overstep fraction is a value between 0 and 1 that tells us how far we are between
        // two fixed timesteps.
        let alpha = fixed_time.overstep_fraction();

        let rendered_translation = previous.lerp(current, alpha);
        transform.translation = rendered_translation;
    }
}

fn reinit_game(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut score: ResMut<ScoreBoard>,
    pipes: Query<Entity, With<PipeStack>>,
    mut player: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut PhysicalTranslation,
            &mut PreviousPhysicalTranslation,
            Entity,
        ),
        With<Player>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    //delete all pipestacks
    for pipe in pipes.iter() {
        commands.entity(pipe).despawn();
    }
    // reset player by recreating it
    if let Ok(player) = player.single() {
        commands.entity(player.4).despawn();
    }
    create_player(&mut commands, &asset_server);

    //reset score
    score.passed_pipes = 0;
    next_state.set(GameState::InGame);
}

fn show_game_over_screen(mut commands: Commands, mut score: ResMut<ScoreBoard>, window: Query<&Window>) {
    // draw a transparent black rectangle
    let resolution = &window.single().unwrap().resolution;
    let rect = Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..Default::default()
    };
    commands.spawn((DeathScreen, (rect, Transform::default(), Visibility::default(), BackgroundColor(Color::BLACK.with_alpha(0.5)))));
}

fn despawn_death_screen(
    mut commands: Commands,
    death_screen: Query<Entity, With<DeathScreen>>,
) {
    for death_screen in death_screen.iter() {
        commands.entity(death_screen).despawn();
    }
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut score: ResMut<ScoreBoard>,
    death_screen: Query<Entity, With<DeathScreen>>,
    pipes: Query<Entity, With<PipeStack>>,
    mut player: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut PhysicalTranslation,
            &mut PreviousPhysicalTranslation,
            Entity,
        ),
        With<Player>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    /// Since Bevy's default 2D camera setup is scaled such that
    /// one unit is one pixel, you can think of this as
    /// "How many pixels per second should the player move?"
    const SPEED: f32 = 210.0;

    if keyboard_input.just_pressed(KeyCode::Space) {
        let (_, mut velocity, _, _, _) = player.single_mut().expect("No player found");
        velocity.0 = Vec3::new(0., SPEED, 0.);
        commands.spawn(AudioPlayer::new(asset_server.load("jump.wav")));
    } else if keyboard_input.just_released(KeyCode::KeyR) {
        reinit_game(asset_server, commands, score, pipes, player, next_state);
    } else if keyboard_input.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Paused);
    }
}

fn handle_input_unpause(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    /// Since Bevy's default 2D camera setup is scaled such that
    /// one unit is one pixel, you can think of this as
    /// "How many pixels per second should the player move?"
    const SPEED: f32 = 210.0;

    if keyboard_input.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::InGame);
    }
}

fn handle_input_when_dead(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Game over screen, show score, show instructions to restart

    if keyboard_input.just_released(KeyCode::KeyR) {
        next_state.set(GameState::InitGame);
    }
}

fn move_pipe(
    mut query: Query<(&mut Transform, Entity), With<PipeStack>>,
    fixed_time: Res<Time<Fixed>>,
    mut score: ResMut<ScoreBoard>,
    mut commands: Commands,
) {
    for (mut transform, pipe_stack_ent) in query.iter_mut() {
        let initial_position = transform.translation.x;
        transform.translation.x -= 100.0 * fixed_time.delta_secs();
        if initial_position > 0. && transform.translation.x <= 0. {
            score.passed_pipes += 1;
            println!("Passed: {}", score.passed_pipes);
            dbg!(score.passed_pipes);
        }

        if transform.translation.x < -2000.0 {
            commands.entity(pipe_stack_ent).despawn()
        }
    }
}

fn check_collision(
    pipe_stack_transforms_query_factory_abstract_visitor: Query<&mut Transform, With<PipeStack>>,
    player: Query<&Transform, (With<Player>, Without<PipeStack>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let player_transform = player.single().expect("No player found");
    // TODO: Factor out this constante
    let player_width = 100.;
    let pipe_width = 100.;
    for pipe_of_things_you_all_are_smoking in
        pipe_stack_transforms_query_factory_abstract_visitor.iter()
    {
        let (pipe_x, pipe_y) = (
            pipe_of_things_you_all_are_smoking.translation.x,
            pipe_of_things_you_all_are_smoking.translation.y,
        );

        // Check if this pipe coordinates and the player coordinates are intersecting
        let pipe_x_start = pipe_x - pipe_width / 2.;
        let pipe_x_end = pipe_x + pipe_width / 2.;
        let player_x_start = player_transform.translation.x - player_width / 2.;
        let player_x_end = player_transform.translation.x + player_width / 2.;

        let intersects_horizontal = pipe_x_start <= player_x_end && pipe_x_end >= player_x_start;
        if !intersects_horizontal {
            continue;
        }

        // Check vertical intersection
        let gap_bottom = pipe_y - VERTICAL_GAP_SIZE / 2.;
        let gap_top = pipe_y + VERTICAL_GAP_SIZE / 2.;

        let half_height = if player_transform.translation.x < pipe_x_start {
            sqrt(
                powf(player_width / 2.0, 2.0)
                    - powf(pipe_x_start - player_transform.translation.x, 2.0),
            )
        } else if player_transform.translation.x > pipe_x_end {
            sqrt(
                powf(player_width / 2.0, 2.0)
                    - powf(player_transform.translation.x - pipe_x_end, 2.0),
            )
        } else {
            player_width / 2.0
        };

        let player_bottom = player_transform.translation.y - half_height;
        let player_top = player_transform.translation.y + half_height;
        let ingap = gap_top > player_top && gap_bottom < player_bottom;
        if !ingap {
            next_state.set(GameState::DeathScreen);
        }
    }
}

fn spawn_pipes(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut timer: ResMut<PipeTimer>,
) {
    // update our timer with the time elapsed since the last update
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    spawn_pipe_at(600.0, &mut commands, asset_server);
}

fn spawn_pipe_at(x: f32, commands: &mut Commands, asset_server: Res<AssetServer>) {
    let mut rng = rand::rng();
    let gap_y_center = rng.random::<f32>() * 500.0 - 250.0;

    commands.spawn((
        PipeStack,
        Transform::from_translation(Vec3::new(x, gap_y_center, 0.0)),
        Visibility::default(),
        children![
            (
                Sprite {
                    image: asset_server.load("meta-pipe.png"),
                    custom_size: Some(Vec2::new(100., 500.)),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::new(0.0, -VERTICAL_GAP_SIZE / 2.0 - 250., 0.0,)),
                Pipe,
            ),
            (
                Sprite {
                    image: asset_server.load("meta-pipe.png"),
                    custom_size: Some(Vec2::new(100., 500.)),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::new(0.0, -VERTICAL_GAP_SIZE / 2.0 - 750., 0.0,)),
                Pipe,
            ),
            (
                Sprite {
                    image: asset_server.load("meta-pipe.png"),
                    custom_size: Some(Vec2::new(100., 500.)),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::new(0.0, VERTICAL_GAP_SIZE / 2.0 + 250., 0.0,)),
                Pipe,
            ),
            (
                Sprite {
                    image: asset_server.load("meta-pipe.png"),
                    custom_size: Some(Vec2::new(100., 500.)),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::new(0.0, VERTICAL_GAP_SIZE / 2.0 + 750., 0.0,)),
                Pipe,
            ),
        ],
    ));
}

fn update_score(mut query: Query<&mut Text, With<ScoreText>>, score_board: Res<ScoreBoard>) {
    for mut text in query.iter_mut() {
        text.0 = format!("Score: {}", score_board.passed_pipes);
    }
}

/**
 * Allows the player to wrap around the screen top and bottom
 */
fn replace_player(mut query: Query<&mut PhysicalTranslation, With<Player>>) {
    let mut player_transform = query.single_mut().unwrap();
    if player_transform.0.y < -500.0 {
        player_transform.0.y = 500.0;
    }
}
