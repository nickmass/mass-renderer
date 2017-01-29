use ::renderer::Surface;
use ::{
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
    pub texture: Surface<V4>,
    pub faces: Vec<Face>,
    vertices: Vec<V3>,
}

impl Model {
    pub fn load<G: AsRef<::std::path::Path>, D: AsRef<::std::path::Path>>(geometry: G, diffuse: D) -> Model {
        use ::std::io::{BufReader, BufRead};
        enum ModelObj {
            Vert(V3),
            Tex(V2),
            Norm(V3),
            Face(Vec<Vec<usize>>),
            Invalid,
        }

        let reader = BufReader::new(::std::fs::File::open(geometry).unwrap());
        let (verts, texs, norms, faces, valid) = reader.lines().filter_map(|l| {
            let l = l.unwrap();
            let parts: Vec<&str> = l.split(' ').filter(|x| x.len() > 0).collect();
            if parts.len() == 0 { return None; }

            match parts[0] {
                "v" => {
                    if parts.len() != 4 {
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
                    if parts.len() != 4 {
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
                        }).collect();

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

        let faces = faces.iter().map(|f| {
            let (v,t,n) = f.iter().fold((Vec::new(), Vec::new(), Vec::new()), |mut a, t| {
                a.0.push(*verts.get(t[0] - 1).unwrap());
                a.1.push(*texs.get(t[1] - 1).unwrap());
                a.2.push(*norms.get(t[2] - 1).unwrap());
                a
            });

            Face {
                verts: v,
                texs: t,
                norms: n,
            }
        }).collect();

        Model {
            texture: Surface::from_file(diffuse),
            faces: faces,
            vertices: verts,
        }
    }
}
