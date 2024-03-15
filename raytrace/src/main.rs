use std::collections::btree_set::Intersection;

use colour::colour_from_kelvin;
use na::OPoint;
use nalgebra as na;

mod colour;
mod ppm;

struct Skybox {
    ground_color: na::Vector3<f32>,
    sky_color: na::Vector3<f32>,
}

struct Sphere {
    center: na::Point3<f32>,
    radius: f32,
    color: na::Vector3<f32>,
}

struct Scene {
    objects: Vec<Object>,
    light: PointLight,
}

struct PointLight {
    position: na::Point3<f32>,
    color: na::Vector3<f32>,
}

enum Object {
    Skybox(Skybox),
    Sphere(Sphere),
}

const SURFACE_TEMP_SUN_KELVIN: f32 = 5778.0; // pretty hot
const SURFACE_TEMP_EARTH_KELVIN: f32 = 288.0; // pretty nice
const SURFACE_TEMP_ICE_KELVIN: f32 = 273.0; // pretty cold

fn main() {
    let camera_position = na::Point3::new(0., 0., 100.0);
    let view_target = na::Point3::new(0.0, 0.0, 0.0);
    let up = na::Vector3::new(0.0, 1.0, 0.0);

    let view = na::Matrix4::look_at_rh(&camera_position, &view_target, &up);

    // let aspect_ratio = 1.0;
    // let perspective =
    //     na::Matrix4::new_perspective(aspect_ratio, std::f32::consts::FRAC_PI_4, 0.001, 100.0);
    let sun = PointLight {
        position: na::Point3::new(-10., 50., 0.),
        color: colour_from_kelvin(SURFACE_TEMP_SUN_KELVIN),
    };

    let mut scene = Scene {
        objects: vec![
            Object::Skybox(Skybox {
                ground_color: na::Vector3::new(0.0, 255.0, 0.0),
                sky_color: na::Vector3::new(0.0, 0.0, 255.0),
            }),
            Object::Sphere(Sphere {
                center: na::Point3::new(0., 0., 0.),
                radius: 10.,
                color: na::Vector3::new(255., 0., 0.),
            }),
            Object::Sphere(Sphere {
                center: na::Point3::new(12., 0., 0.),
                radius: 5.,
                color: na::Vector3::new(255., 0., 255.),
            }),
            // lmao sun
            Object::Sphere(Sphere {
                center: sun.position, // - na::Vector3::new(0., 5., 0.),
                radius: 10.,
                color: sun.color,
            }),
        ],
        light: sun,
    };
    for i in 0..30 {
        scene.objects.push(Object::Sphere(Sphere {
            center: na::Point3::new(i as f32, 0., 0.),
            radius: 1.,
            color: na::Vector3::new(255., 0., 0.),
        }));
        scene.objects.push(Object::Sphere(Sphere {
            center: na::Point3::new(0., i as f32, 0.),
            radius: 1.,
            color: na::Vector3::new(0., 255., 0.),
        }));
        scene.objects.push(Object::Sphere(Sphere {
            center: na::Point3::new(0., 0., i as f32),
            radius: 1.,
            color: na::Vector3::new(0., 0., 255.),
        }));
    }

    let xres = 1024;
    let yres = 1024;

    let mut pixels = vec![[0, 0, 0]; xres * yres];

    for ix in 0..xres {
        for iy in 0..yres {
            let ray_target_viewspace = na::Point3::new(
                ix as f32 / (xres - 1) as f32 * 2.0 - 1.0,
                (iy as f32 / (yres - 1) as f32 * 2.0 - 1.0) * -1.,
                -1.0,
            );

            let ray_target_worldspace = view
                .try_inverse()
                .unwrap()
                .transform_point(&ray_target_viewspace);

            let ray_vector = ray_target_worldspace - camera_position;
            if ix == 0 && iy == 0 {
                dbg!(ray_vector);
            }
            let intersection = trace_ray(&scene, camera_position, ray_vector);

            // Phake Phong Lighting
            let intersection_point = camera_position + ray_vector * intersection.distance;
            let light_to_telekom = -(intersection_point - scene.light.position).normalize();
            let mut color_at_pixel =
                intersection.color * light_to_telekom.dot(&intersection.normal).max(0.);
            color_at_pixel += intersection.color * 0.5;

            pixels[ix + iy * xres] = [
                color_at_pixel.x as u8,
                color_at_pixel.y as u8,
                color_at_pixel.z as u8,
            ];
            if iy == 0 {
                pixels[ix + iy * xres] = [255, 0, 0];
            }
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

fn trace_ray(
    scene: &Scene,
    camera_position: na::Point3<f32>,
    ray_vector: na::Vector3<f32>,
) -> IntersectionResult {
    scene
        .objects
        .iter()
        .filter_map(|obj| intersect(obj, camera_position, ray_vector))
        .min_by(|r1, r2| r1.distance.partial_cmp(&r2.distance).unwrap())
        .unwrap()
}

struct IntersectionResult {
    color: na::Vector3<f32>,
    distance: f32,
    normal: na::Vector3<f32>,
}

fn intersect(
    obj: &Object,
    cam: na::Point3<f32>,
    ray: na::Vector3<f32>,
) -> Option<IntersectionResult> {
    match obj {
        Object::Skybox(skybox) => Some(IntersectionResult {
            color: if ray.y < 0. {
                skybox.ground_color
            } else {
                skybox.sky_color
            },
            distance: f32::INFINITY,
            normal: -ray.normalize(),
        }),
        Object::Sphere(s) => {
            // original code do not steal
            //vec3 oc = r.origin() - center;
            //float a = dot(r.direction(), r.direction());
            //float b = 2.0 * dot(oc, r.direction());
            //float c = dot(oc,oc) - radius*radius;
            //float discriminant = b*b - 4*a*c;
            //return (discriminant>0);

            let oc = cam - s.center;
            let a = ray.dot(&ray);
            let b = 2. * oc.dot(&ray);
            let c = oc.dot(&oc) - s.radius * s.radius;
            let discriminant = b * b - 4. * a * c;
            if discriminant > 0. {
                let dist = (-b - discriminant.sqrt()) / (2.0 * a);
                Some(IntersectionResult {
                    color: s.color,
                    distance: dist,
                    normal: (cam + dist * ray - s.center).normalize(),
                })
            } else {
                None
            }
        }
    }
}
