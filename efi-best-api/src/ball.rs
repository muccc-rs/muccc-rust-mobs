use core::intrinsics::{
    expf32,
    sqrtf32,
    powf32,
};
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

fn mix(x: f32, y: f32, a: f32) -> f32 {
    let a = clamp(a, 0f32, 1f32);
    x * (1f32 - a) + y * a
}

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v < min {
        return min
    }
    if v > max {
        return max
    }
    v
}

fn step(edge: f32, x: f32) -> f32 {
    if x < edge {
        0.0f32
    } else {
        1.0f32
    }
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 {
            x, y, z,
        }
    }
    fn length(&self) -> f32 {
        unsafe {
            sqrtf32(self.x * self.x + self.y * self.y + self.z * self.z)
        }
    }

    fn cross(&self, o: &Self) -> Self {
        let x = self.y * o.z - self.z * o.y;
        let y = self.z * o.x - self.x * o.z;
        let z = self.x * o.y - self.y * o.x;
        Vec3::new(x, y, z)
    }

    fn dot(&self, o: &Self) -> f32 {
        (self.x * o.x) + (self.y * o.y) + (self.z * o.z)
    }

    fn normalize(&self) -> Vec3 {
        let l = self.length();
        Vec3 {
            x: self.x / l,
            y: self.y / l,
            z: self.z / l,
        }
    }

    fn mult(&self, o: &Self) -> Self {
        Vec3::new(self.x * o.x, self.y * o.y, self.z * o.z)
    }

    fn mult_scal(&self, o: f32) -> Self {
        Vec3::new(self.x * o ,self.y * o, self.z * o)
    }

    fn add(&self, o: &Self) -> Self {
        Vec3 {
            x: self.x + o.x,
            y: self.y + o.y,
            z: self.z + o.z,
        }
    }

    fn sub(&self, o: &Self) -> Self {
        Vec3 {
            x: self.x - o.x,
            y: self.y - o.y,
            z: self.z - o.z,
        }
    }
}

fn map(pos: &Vec3) -> f32 {
    let d1 = pos.sub(&Vec3::new(0f32, 0f32, 0f32)).length() - 0.25f32;
    let d2 = pos.sub(&Vec3::new(0.3f32, 0.0f32, 0f32)).length() - 0.15f32;
    let d3 = pos.sub(&Vec3::new(-0.3f32, -0.1f32, 0.1f32)).length() - 0.10f32;
    let d4 = pos.y - (-0.25f32);
    d1.min(d2).min(d3).min(d4)
}

fn calc_normal(pos: &Vec3) -> Vec3 {
    let e = 0.0001;
    let dx = map(&pos.add(&Vec3::new(e, 0.0f32, 0.0f32))) - map(&pos.sub(&Vec3::new(e, 0.0f32, 0.0f32)));
    let dy = map(&pos.add(&Vec3::new(0.0f32, e, 0.0f32))) - map(&pos.sub(&Vec3::new(0.0f32, e, 0.0f32)));
    let dz = map(&pos.add(&Vec3::new(0.0f32, 0.0f32, e))) - map(&pos.sub(&Vec3::new(0.0f32, 0.0f32, e)));
    Vec3::new(dx, dy, dz).normalize()
}

fn cast_ray(ro: &Vec3, rd: &Vec3) -> f32 {
    let mut t = 0.0f32;
    for _ in 0..300 {
        let pos = ro.add(&rd.mult_scal(t));
        let h = map(&pos);
        if h < 0.00001 {
            break;
        }
        t += h;

        if t > 50.0 {
            return -1f32;
        }
    }
    t
}

pub fn mainImage(x: f32, y: f32) -> Vec3 {
    let ro = Vec3::new(0.6f32, 0f32, 0.8f32);
    let ta = Vec3::new(0f32, 0f32, 0f32);

    let ww = (ta.sub(&ro)).normalize();
    let uu = (ww.cross(&Vec3::new(0f32, 1f32, 0f32))).normalize();
    let vv = uu.cross(&ww).normalize();

    let rd = uu.mult_scal(x).add(&vv.mult_scal(y)).add(&ww.mult_scal(1.5)).normalize();

    let grad = 0.5 * rd.y;
    let mut col = Vec3::new(0.4f32 - grad, 0.75f32 - grad, 1f32 - grad);
    let e = unsafe { expf32(-10f32 * rd.y) };
    col.x = mix(col.x, 0.7, e);
    col.y = mix(col.y, 0.75, e);
    col.z = mix(col.z, 0.8, e);

    let t = cast_ray(&ro, &rd);
    if t > 0.0f32 {
        let pos = ro.add(&rd.mult_scal(t));
        let nor = calc_normal(&pos);

        let mate = Vec3::new(0.18f32, 0.18f32, 0.18f32);
        let sun_dir = Vec3::new(0.8f32, 0.4f32, 0.2f32).normalize();
        let sun_dif = clamp(nor.dot(&sun_dir), 0f32, 1f32);
        let sun_sha = step(cast_ray(&pos.add(&nor.mult_scal(0.001)), &sun_dir), 0f32);
        let sky_dif = clamp(0.5f32 + 0.5f32 * nor.dot(&Vec3::new(0f32, 1f32, 0f32)), 0f32, 1f32);
        let bou_dif = clamp(0.5f32 + 0.5f32 * nor.dot(&Vec3::new(0f32, -1f32, 0f32)), 0f32, 1f32);

        col = mate.mult(&Vec3::new(7f32, 4.5f32, 3f32).mult_scal(sun_dif*sun_sha));
        col = col.add(&mate.mult(&Vec3::new(0.5f32, 0.8f32, 0.9f32).mult_scal(sky_dif)));
        col = col.add(&mate.mult(&Vec3::new(0.7f32, 0.3f32, 0.2f32).mult_scal(bou_dif)));
    }

    col.x = unsafe { powf32(col.x, 0.4545) };
    col.y = unsafe { powf32(col.y, 0.4545) };
    col.z = unsafe { powf32(col.z, 0.4545) };

    col
}