use std::fs::read_to_string;
use nalgebra as na;

#[derive(Debug, Clone)]
pub enum LoadError {
    Lmao,
}

#[derive(Debug, Clone)]
pub struct Obj {
    pub vertices: Vec<na::Point3<f32>>,
    pub faces: Vec<(usize, usize, usize)>,
}

impl Obj {
    pub fn load(path: &std::path::Path) -> Result<Obj, LoadError> {
        let mut res = Obj {
            vertices: vec![],
            faces: vec![],
        };
        for line in read_to_string(path).unwrap().lines() {
            let line = line.trim();
            if line.starts_with("#") {
                continue;
            }
            let mut parts = line.split_ascii_whitespace();

            match parts.next() {
                None => continue,
                Some("v") => {
                    let numbers: Result<Vec<f32>, _> = parts.map(|el| el.parse::<f32>()).collect();
                    if let Ok(numbers) = numbers {
                        match numbers.as_slice() {
                            [x, y, z, w] => { 
                                res.vertices.push(na::Point3::new(x / w, y / w, z / w));
                            },
                            [x, y, z] => { 
                                res.vertices.push(na::Point3::new(*x, *y, *z));
                            },
                            _ => (),
                        }
                    }
                },
                Some("f") => {
                    let numbers: Result<Vec<usize>, _> = parts.map(|el| el.parse::<usize>()).collect();
                    if let Ok(numbers) = numbers {
                        match numbers.as_slice() {
                            [a, b, c] => { 
                                res.faces.push((*a, *b, *c));
                            },
                            _ => (),
                        }
                    }
                },
                Some(_) => {
                    continue;
                }
            }
        }

        Ok(res)
    }
}