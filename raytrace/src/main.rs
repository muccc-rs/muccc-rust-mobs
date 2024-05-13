use colour::colour_from_kelvin;
use nalgebra as na;
use rayon::prelude::*;

mod colour;
mod obj;
mod ppm;

struct Skybox {
    ground_color: na::Vector3<f32>,
    sky_color: na::Vector3<f32>,
}

#[derive(Default, Debug, Clone, Copy)]
struct Material {
    ambient: na::Vector3<f32>,
    diffuse: na::Vector3<f32>,
    specular: na::Vector3<f32>,
    emissive: na::Vector3<f32>,
    reflectivity: na::Vector3<f32>,
}

impl Material {
    pub fn new_ambient_only(ambient: na::Vector3<f32>) -> Self {
        Self {
            ambient,
            diffuse: na::Vector3::zeros(),
            specular: na::Vector3::zeros(),
            emissive: na::Vector3::zeros(),
            reflectivity: na::Vector3::zeros(),
        }
    }
}

struct Ball {
    center: na::Point3<f32>,
    radius: f32,
    material: Material,
}

struct Scene {
    objects: Vec<Object>,
    light: PointLight,
}

struct PointLight {
    position: na::Point3<f32>,
    color: na::Vector3<f32>,
}

struct Triangle {
    points: [na::Point3<f32>; 3],
    material: Material,

    // precomputed
    normal: na::Vector3<f32>,
}

impl Triangle {
    fn new(points: [na::Point3<f32>; 3], material: Material) -> Self {
        let [a, b, c] = points;
        let normal = (b - a).cross(&(c - a)).normalize();

        Self { points, material, normal }
    }
}

enum Object {
    Skybox(Skybox),
    Ball(Ball),
    Plane(Triangle),
    Triangle(Triangle),
}

const SURFACE_TEMP_SUN_KELVIN: f32 = 5778.0; // pretty hot
const SURFACE_TEMP_EARTH_KELVIN: f32 = 288.0; // pretty nice
const SURFACE_TEMP_ICE_KELVIN: f32 = 273.0; // pretty cold

fn main() {
    let camera_position = na::Point3::new(-40.0, 20.0, -40.0);
    let view_target = na::Point3::new(0.0, 0.0, 0.0);
    let up = na::Vector3::new(0.0, 1.0, 0.0);

    let monkey = obj::Obj::load(std::path::Path::new("monkey 🙈.obj")).unwrap();

    let view = na::Matrix4::look_at_rh(&camera_position, &view_target, &up);

    // let aspect_ratio = 1.0;
    // let perspective =
    //     na::Matrix4::new_perspective(aspect_ratio, std::f32::consts::FRAC_PI_4, 0.001, 100.0);
    let sun = PointLight {
        position: na::Point3::new(-10., 50., 0.),
        color: colour_from_kelvin(SURFACE_TEMP_SUN_KELVIN),
    };

    // Coordinate system: RH, Y up
    // Camera: X+ is right.

    let mut scene = Scene {
        objects: vec![
            Object::Skybox(Skybox {
                ground_color: na::Vector3::new(0.0, 255.0, 0.0),
                sky_color: na::Vector3::new(0.0, 0.0, 255.0),
            }),
            #[cfg(feature = "no-triangle-dont-do-it")]
            Object::Triangle(Triangle::new(
                [
                    na::Point3::new(20., 0., 0.),
                    na::Point3::new(0., 20., 0.),
                    na::Point3::new(0., 0., 20.),
                ],
                Material {
                    ambient: na::Vector3::new(10., 10., 10.),
                    diffuse: na::Vector3::new(255., 255., 255.),
                    specular: na::Vector3::new(0., 0., 0.),
                    reflectivity: na::Vector3::new(0.5, 0.5, 0.5),
                }),
            ),
            Object::Ball(Ball {
                center: na::Point3::new(0., 0., 0.),
                radius: 10.,
                material: Material {
                    ambient: na::Vector3::new(0., 0., 0.),
                    diffuse: na::Vector3::new(255., 0., 0.),
                    specular: na::Vector3::new(255., 255., 0.),
                    reflectivity: na::Vector3::new(0.5, 0.5, 0.5),
                    ..Default::default()
                },
            }),
            Object::Ball(Ball {
                center: na::Point3::new(17., 0., 0.),
                radius: 5.,
                material: Material {
                    ambient: na::Vector3::new(0., 0., 0.),
                    diffuse: na::Vector3::new(255., 0., 255.),
                    specular: na::Vector3::new(255., 255., 0.),
                    ..Default::default()
                },
            }),
            Object::Plane(Triangle::new(
                [
                    na::Point3::new(-100., 0., 0.),
                    na::Point3::new(-100., 1., 0.),
                    na::Point3::new(-100., 0., 1.),
                ],
                Material {
                    ambient: na::Vector3::new(0., 0., 0.),
                    diffuse: na::Vector3::new(0., 255., 0.),
                    specular: na::Vector3::new(0., 0., 0.),
                    reflectivity: na::Vector3::new(0.5, 0.5, 0.5),
                    ..Default::default()
                },
            )),
            Object::Plane(Triangle::new(
                [
                    na::Point3::new(100., 0., 0.),
                    na::Point3::new(100., 0., 1.),
                    na::Point3::new(100., 1., 0.),
                ],
                Material {
                    ambient: na::Vector3::new(10., 10., 10.),
                    diffuse: na::Vector3::new(255., 0., 0.),
                    specular: na::Vector3::new(0., 0., 0.),
                    reflectivity: na::Vector3::new(0.5, 0.5, 0.5),
                    ..Default::default()
                },
            )),
            Object::Plane(Triangle::new(
                [
                    na::Point3::new(0., -23., 0.),
                    na::Point3::new(0., -23., 1.),
                    na::Point3::new(1., -23., 0.),
                ],
                Material {
                    ambient: na::Vector3::new(10., 10., 10.),
                    diffuse: na::Vector3::new(0., 0., 255.),
                    reflectivity: na::Vector3::new(0.5, 0.5, 0.5),

                    specular: na::Vector3::new(0., 0., 0.),
                    ..Default::default()
                },
            )),
            Object::Plane(Triangle::new(
                [
                    na::Point3::new(0., 42. * 2., 0.),
                    na::Point3::new(1., 42. * 2., 0.),
                    na::Point3::new(0., 42. * 2., 1.),
                ],
                Material {
                    ambient: na::Vector3::new(0., 0., 0.),
                    // this gonna be good :))))
                    // UPDATE: wasn't good.
                    //diffuse: colour_from_kelvin(SURFACE_TEMP_EARTH_KELVIN),
                    reflectivity: na::Vector3::new(0.5, 0.5, 0.5),

                    diffuse: na::Vector3::new(255.0, 0., 255.),
                    specular: na::Vector3::new(0., 0., 0.),
                    ..Default::default()
                },
            )),
            Object::Plane(Triangle::new(
                [
                    na::Point3::new(0., 0., -100.),
                    na::Point3::new(1., 0., -100.),
                    na::Point3::new(0., 1., -100.),
                ],
                Material {
                    ambient: na::Vector3::new(10., 10., 10.),
                    diffuse: na::Vector3::new(255., 255., 0.),
                    specular: na::Vector3::new(0., 0., 0.),
                    reflectivity: na::Vector3::new(0.5, 0.5, 0.5),

                    ..Default::default()
                },
            )),
            // lmao sun
            Object::Ball(Ball {
                center: sun.position, // - na::Vector3::new(0., 5., 0.),
                radius: 10.,
                material: Material {
                    emissive: sun.color,
                    ..Default::default()
                },
            }),
        ],
        light: sun,
    };

    for &(i, j, k) in &monkey.faces {
        let mut a = monkey.vertices[i - 1];
        let mut b = monkey.vertices[j - 1];
        let mut c = monkey.vertices[k - 1];
        a *= 4.0;
        b *= 4.0;
        c *= 4.0;
        a += na::Vector3::new(-30.0, 0.0, 0.0);
        b += na::Vector3::new(-30.0, 0.0, 0.0);
        c += na::Vector3::new(-30.0, 0.0, 0.0);
        scene.objects.push(Object::Triangle(Triangle::new(
            [a, b, c],
            Material {
                ambient: na::Vector3::new(10., 10., 10.),
                diffuse: na::Vector3::new(128., 255., 255.),
                specular: na::Vector3::new(0., 0., 0.),
                ..Default::default()
            },
        )));
    }

    for i in 0..30 {
        scene.objects.push(Object::Ball(Ball {
            center: na::Point3::new(i as f32, 0., 0.),
            radius: 1.,
            material: Material::new_ambient_only(na::Vector3::new(255., 0., 0.)),
        }));
        scene.objects.push(Object::Ball(Ball {
            center: na::Point3::new(0., i as f32, 0.),
            radius: 1.,
            material: Material::new_ambient_only(na::Vector3::new(0., 255., 0.)),
        }));
        scene.objects.push(Object::Ball(Ball {
            center: na::Point3::new(0., 0., i as f32),
            radius: 1.,
            material: Material::new_ambient_only(na::Vector3::new(0., 0., 255.)),
        }));
    }

    let start = std::time::Instant::now();

    let xres = 512;
    let yres = 512;

    let mut pixels = vec![[0, 0, 0]; xres * yres];
    let mut timing_pixels = vec![[0, 0, 0]; xres * yres];

    let coords = (0..xres).flat_map(|ix| (0..yres).map(move |iy| (ix, iy)));
    let res: Vec<(std::time::Duration, usize, usize, _)> = coords
        .par_bridge()
        .map(|(ix, iy)| {
            let start_pixel = std::time::Instant::now();
            let ray_target_viewspace = na::Point3::new(
                ix as f32 / (xres - 1) as f32 * 2.0 - 1.0,
                (iy as f32 / (yres - 1) as f32 * 2.0 - 1.0) * -1.,
                -1.0,
            );

            let ray_target_worldspace = view
                .try_inverse()
                .unwrap()
                .transform_point(&ray_target_viewspace);

            let ray_vector = (ray_target_worldspace - camera_position);

            let color_at_pixel = sample_color_for_ray(
                &scene,
                camera_position,
                ray_vector,
                3,
                na::Vector3::new(1.0, 1.0, 1.0),
            );
            let took = start_pixel.elapsed();
            (took, ix, iy, color_at_pixel)
        })
        .collect();

    let took_max = res.iter().map(|(took, _, _, _)| took.as_nanos()).max().unwrap();
    for (took, ix, iy, color_at_pixel) in res {
        let took_scaled = took.as_nanos() * 255 / took_max;
        let took_scaled = (took_scaled * 100).min(255);
        timing_pixels[ix + iy * xres] = [
            took_scaled as u8,
            took_scaled as u8,
            took_scaled as u8,
        ];
        pixels[ix + iy * xres] = [
            color_at_pixel.x.min(255.0) as u8,
            color_at_pixel.y.min(255.0) as u8,
            color_at_pixel.z.min(255.0) as u8,
        ];
        if iy == 0 {
            pixels[ix + iy * xres] = [255, 0, 0];
        }
        if iy == 1 {
            pixels[ix + iy * xres] = [255, 255, 255];
        }
        if iy == 2 {
            pixels[ix + iy * xres] = [0, 0, 255];
        }
    }

    ppm::write(
        &pixels,
        xres,
        yres,
        std::fs::File::create("test.ppm").unwrap(),
    )
    .unwrap();
    ppm::write(
        &timing_pixels,
        xres,
        yres,
        std::fs::File::create("timing.ppm").unwrap(),
    )
    .unwrap();

    eprintln!("it took {:?}", start.elapsed());
}

fn sample_color_for_ray(
    scene: &Scene,
    src: na::Point3<f32>,
    ray: na::Vector3<f32>,
    bounces_left: u32,
    importance: na::Vector3<f32>,
) -> na::Vector3<f32> {
    let intersection = trace_ray(&scene, src, ray);
    let intersection_point = src + ray * intersection.distance; // ISP

    // Real Raytraced Specular (tm)
    let importance = importance.component_mul(&intersection.material.reflectivity);
    let reflection = if bounces_left > 0 && importance > na::Vector3::new(0.1, 0.1, 0.1) {
        if ray.dot(&intersection.normal) < 0. {
            let new_ray = ray - 2.0 * ray.dot(&intersection.normal) * intersection.normal;
            sample_color_for_ray(
                scene,
                intersection_point,
                new_ray,
                bounces_left - 1,
                importance,
            )
        } else {
            na::Vector3::new(0.0, 0.0, 0.0)
        }
    } else {
        na::Vector3::new(0.0, 0.0, 0.0)
    };

    // Phake Phong Lighting
    let light_to_telekom = -(intersection_point - scene.light.position).normalize();
    let halfway_to_telekom = (light_to_telekom - ray.normalize()).normalize();

    let diffuse =
        intersection.material.diffuse * light_to_telekom.dot(&intersection.normal).max(0.);
    // let diffuse = na::Vector3::zeros();
    let ambient = intersection.material.ambient;

    let color_at_pixel = diffuse
        + ambient
        + reflection.component_mul(&intersection.material.reflectivity)
        + intersection.material.emissive;
    color_at_pixel
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
    material: Material,
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
            material: Material::new_ambient_only(if ray.y < 0. {
                skybox.ground_color
            } else {
                skybox.sky_color
            }),
            distance: f32::INFINITY,
            normal: -ray.normalize(),
        }),
        Object::Plane(p) => {
            let [a, b, c] = p.points;
            let normal = (b - a).cross(&(c - a)).normalize();
            let denom = ray.dot(&normal);

            if denom.abs() < 1e-8 {
                return None;
            }

            let t = (a - cam).dot(&normal) / denom;
            if t < 0.0 {
                return None;
            }

            if ray.dot(&normal) < 0.0 {
                Some(IntersectionResult {
                    normal,
                    material: p.material,
                    distance: t,
                })
            } else {
                None
            }
        }
        Object::Triangle(p) => {
            let [a, b, c] = p.points;
            let normal = p.normal;
            let denom = ray.dot(&normal);

            // Not looking towards the plane.
            if denom.abs() < 1e-8 {
                return None;
            }

            let t = (a - cam).dot(&normal) / denom;
            if t < 0.0 {
                return None;
            }

            // Not on the right side of the plane.
            if ray.dot(&normal) > 0.0 {
                return None;
            }

            // TODO: check if point is in triangle bounds.
            let point = cam + ray * t;

            let area = normal.dot(&(b - a).cross(&(c - a)));
            let area1 = normal.dot(&(b - point).cross(&(c - point)));
            let area2 = normal.dot(&(point - a).cross(&(c - a)));
            let u = area1 / area;
            let v = area2 / area;
            if u < 0. || v < 0. || u + v > 1. {
                return None;
            }

            Some(IntersectionResult {
                normal,
                material: p.material,
                distance: t,
            })
        }
        Object::Ball(s) => {
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
                if dist < 0. {
                    return None;
                }
                Some(IntersectionResult {
                    material: s.material,
                    distance: dist,
                    normal: (cam + dist * ray - s.center).normalize(),
                })
            } else {
                None
            }
        }
    }
}
