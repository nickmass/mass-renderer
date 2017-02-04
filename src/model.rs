use ::renderer::{ Surface, Texture };
use ::{
    ElementWise,
    V2,
    V3,
    V4,
    v2,
    v3,
};

pub struct Face {
    pub verts: Vec<V3>,
    pub texs: Vec<V2>,
    pub norms: Vec<V3>,
}

pub struct Model {
    faces: Vec<Vec<Vec<usize>>>,
    verts: Vec<V3>,
    uvs: Vec<V2>,
    norms: Vec<V3>,
    diffuse: Texture<V4>,
    specular: Texture<V4>,
    normal: Texture<V4>,
}

use ::std::path::Path;
impl Model {
    pub fn load<G, D, S, N>(geometry: G, diffuse: D, specular: S, normal: N) -> Model
        where G: AsRef<Path>, D: AsRef<Path>, S: AsRef<Path>, N: AsRef<Path>
    {
        use ::std::io::{BufReader, BufRead};
        enum ModelObj {
            Vert(V3),
            Tex(V2),
            Norm(V3),
            Face(Vec<Vec<usize>>),
            Invalid,
        }

        let reader = BufReader::new(::std::fs::File::open(geometry).unwrap());
        let (verts, uvs, norms, faces, valid) = reader.lines().filter_map(|l| {
            let l = l.unwrap();
            let parts: Vec<&str> = l.split(' ').filter(|x| x.len() > 0).collect();
            if parts.len() == 0 { return None; }

            match parts[0] {
                "v" => {
                    if parts.len() < 4 {
                        Some(ModelObj::Invalid)
                    } else {
                        let x = parts[1].parse();
                        let y = parts[2].parse();
                        let z = parts[3].parse();

                        if x.is_err() || y.is_err() || z.is_err() {
                            None
                        } else {
                            Some(ModelObj::Vert(v3(
                                x.unwrap(),
                                y.unwrap(),
                                z.unwrap())))
                        }

                    }
                },
                "vt" => {
                    if parts.len() < 3 {
                        Some(ModelObj::Invalid)
                    } else {
                        let x = parts[1].parse();
                        let y = parts[2].parse();

                        if x.is_err() || y.is_err() {
                            None
                        } else {
                            Some(ModelObj::Tex(v2(
                                x.unwrap(),
                                y.unwrap()
                            )))
                        }
                    }
                },
                "vn" => {
                    if parts.len() < 4 {
                        Some(ModelObj::Invalid)
                    } else {
                        let x = parts[1].parse();
                        let y = parts[2].parse();
                        let z = parts[3].parse();

                        if x.is_err() || y.is_err() || z.is_err() {
                            None
                        } else {
                            Some(ModelObj::Norm(v3(
                                x.unwrap(),
                                y.unwrap(),
                                z.unwrap()
                            )))
                        }
                    }
                }
                "f" => {
                    if parts.len() < 4 {
                        Some(ModelObj::Invalid)
                    } else {
                        let mut invalid = false;
                        let faces = parts[1..].iter().map(|p| {
                            let v: Vec<usize> = p.split('/').map(|x| {
                                let x = x.parse();
                                if x.is_err() {
                                    invalid = true;
                                    0
                                } else {
                                    x.unwrap()
                                }
                            }).collect();

                            if v.len() != 3 {
                                invalid = true;
                            }

                            v
                        }).take(3).collect();

                        if invalid {
                            Some(ModelObj::Invalid)
                        } else {
                            Some(ModelObj::Face(faces))
                        }
                    }
                },
                _ => None,
            }
        }).fold((Vec::new(), Vec::new(), Vec::new(), Vec::new(), true), |mut col, item| {
            match item {
                ModelObj::Vert(v) => col.0.push(v),
                ModelObj::Face(f) => col.3.push(f),
                ModelObj::Tex(t) => col.1.push(t),
                ModelObj::Norm(n) => col.2.push(n),
                ModelObj::Invalid => col.4 = false,
            };

            col
        });

        if !valid {
            panic!("Invalid .obj file");
        }

        Model {
            faces: faces,
            verts: verts,
            uvs: uvs,
            norms: norms,
            diffuse: Texture::from_file(diffuse),
            specular: Texture::from_file(specular),
            normal: Texture::from_file(normal),
        }
    }

    pub fn faces<'a>(&'a self) -> FaceIterator<'a> {
        FaceIterator {
            model: self,
            cur: 0,
        }
    }

    pub fn diffuse(&self, uv: V2) -> V4 {
        self.diffuse.get_f(uv.x, uv.y)
    }

    pub fn specular(&self, uv: V2) -> f64 {
        self.specular.get_f(uv.x, uv.y).x * 255.
    }

    pub fn normal(&self, uv: V2) -> V3 {
        (self.normal.get_f(uv.x, uv.y) * 2.).sub_element_wise(1.).truncate()
    }
}

pub struct FaceIterator<'a> {
    model: &'a Model,
    cur: usize,
}

impl<'a> Iterator for FaceIterator<'a> {
    type Item = Face;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.model.faces.get(self.cur) {
            self.cur += 1;
            let (v,t,n) = next.iter().fold((Vec::new(), Vec::new(), Vec::new()), |mut a, t| {
                a.0.push(*self.model.verts.get(t[0] - 1).unwrap());
                a.1.push(*self.model.uvs.get(t[1] - 1).unwrap());
                a.2.push(*self.model.norms.get(t[2] - 1).unwrap());
                a
            });

            Some(Face {
                verts: v,
                texs: t,
                norms: n,
            })
        } else {
            None
        }
    }
}
