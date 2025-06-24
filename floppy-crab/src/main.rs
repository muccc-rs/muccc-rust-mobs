use bevy::prelude::*;

const GRAVITY: f32 = 420.69;

// ADR:
// 2 options
// 1st move coordinate system of world left
// 2nd move crab right, move camera left, ...
// Choose 1st

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (advance_physics, move_pipe))
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
                handle_input.in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
                // The player's visual representation needs to be updated after the physics
                // simulation has been advanced. This could be run in `Update`, but if we run it
                // here instead, the systems in `Update` will be working with the `Transform` that
                // will actually be shown on screen.
                interpolate_rendered_transform.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
            ),
        )
        .run();
}

/// A vector representing the player's velocity in the physics simulation.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct Velocity(Vec3);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
struct Pipe;

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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_image(asset_server.load("gentleman-ferris-transparent.png")),
        Transform::from_scale(Vec3::splat(0.3)),
        Velocity::default(),
        PhysicalTranslation::default(),
        PreviousPhysicalTranslation::default(),
    ));
}

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
        velocity.0 = velocity.0 + Vec3::new(0., -GRAVITY, 0.) * fixed_time.delta_secs();
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

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    /// Since Bevy's default 2D camera setup is scaled such that
    /// one unit is one pixel, you can think of this as
    /// "How many pixels per second should the player move?"
    const SPEED: f32 = 210.0;
    for mut velocity in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Space) {
            velocity.0 = Vec3::new(0., SPEED, 0.);
        }
        if keyboard_input.pressed(KeyCode::Enter) {
            spawn_pipe(&mut commands, &asset_server);
        }
    }
}

fn spawn_pipe(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.spawn((
        Sprite::from_image(asset_server.load("meta-pipe.png")),
        Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        Pipe,
    ));
}

fn move_pipe(mut query: Query<&mut Transform, With<Pipe>>, fixed_time: Res<Time<Fixed>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x -= 100.0 * fixed_time.delta_secs();
    }
}
