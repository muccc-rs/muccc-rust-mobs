use nalgebra as na;

mod ppm;

struct Skybox {
    z: f32,
    ground_color: na::Vector3<f32>,
    sky_color: na::Vector3<f32>,
}

struct Sphere {
    center: na::Point3<f32>,
    radius: f32,
    color: na::Vector3<f32>,
}

enum Object {
    Skybox(Skybox),
    Sphere(Sphere),
}

fn main() {
    let camera_position = na::Point3::new(10.0, 10.0, 10.0);
    let view_target = na::Point3::new(0.0, 0.0, 0.0);
    let up = na::Vector3::new(0.0, 1.0, 0.0);

    let view = na::Matrix4::look_at_rh(&camera_position, &view_target, &up);

    // let aspect_ratio = 1.0;
    // let perspective =
    //     na::Matrix4::new_perspective(aspect_ratio, std::f32::consts::FRAC_PI_4, 0.001, 100.0);

    let scene = vec![Object::Skybox(Skybox {
        z: 0.0,
        ground_color: na::Vector3::new(0.0, 255.0, 0.0),
        sky_color: na::Vector3::new(0.0, 0.0, 255.0),
    })];

    let xres = 1024;
    let yres = 1024;

    let mut pixels = vec![[0, 0, 0]; xres * yres];

    println!(
        "{}\t{}\t{}",
        camera_position.x, camera_position.y, camera_position.z
    );

    for ix in 0..xres {
        for iy in 0..yres {
            let ray_target_viewspace = na::Point3::new(
                ix as f32 / (xres - 1) as f32 * 2.0 - 1.0,
                (iy as f32 / (yres - 1) as f32 * 2.0 - 1.0) * -1.0,
                1.0,
            );

            let ray_target_worldspace = view
                .try_inverse()
                .unwrap()
                .transform_point(&ray_target_viewspace);

            println!(
                "{}\t{}\t{}",
                ray_target_worldspace.x, ray_target_worldspace.y, ray_target_worldspace.z
            );

            let ray_vector = ray_target_worldspace - camera_position;
            let color_at_pixel = scene
                .iter()
                .map(|obj| intersect(obj, camera_position, ray_vector))
                .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
                .map(|(color, _)| color)
                .unwrap();

            pixels[ix + iy * xres] = [
                color_at_pixel.x as u8,
                color_at_pixel.y as u8,
                color_at_pixel.z as u8,
            ];
        }
    }

    ppm::write(
        &pixels,
        xres,
        yres,
        std::fs::File::create("test.ppm").unwrap(),
    )
    .unwrap();
}

fn intersect(obj: &Object, cam: na::Point3<f32>, ray: na::Vector3<f32>) -> (na::Vector3<f32>, f32) {
    match obj {
        Object::Skybox(skybox) => {
            if ray.y < skybox.z {
                (skybox.ground_color, f32::INFINITY)
            } else {
                (skybox.sky_color, f32::INFINITY)
            }
        }
        Object::Sphere(s) => {
            unimplemented!();
        }
    }
}
