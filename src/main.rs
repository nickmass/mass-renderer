extern crate image;
extern crate cgmath;

use cgmath::{
    InnerSpace,
    SquareMatrix,
    vec2 as v2,
    vec3 as v3,
    vec4 as v4,
    Vector2,
    Vector3,
    Vector4,
    Matrix3,
    Matrix4,
};

type V2 = Vector2<f64>;
type V3 = Vector3<f64>;
type V4 = Vector4<f64>;
type M3 = Matrix3<f64>;
type M4 = Matrix4<f64>;

fn main() {
    let obj = ::std::fs::File::open("../../tinyrenderer/obj/african_head/african_head.obj");
    let model = Model::load(obj.unwrap(), "../../tinyrenderer/obj/african_head/african_head_diffuse.tga");
    let (width, height) = (1024, 1024);
    let mut image = Image::new(width, height);
    let mut z_buf = vec![::std::f64::MIN; (width*height) as usize];
    for face in model.faces.iter() {
        triangle(&mut image, &mut z_buf, &face, &model);
    }

    let _ = image.write("image.png");
}

fn barycentric(tri: &Vec<V3>, p: &V2) -> V3 {
    let u = v3(
        tri[2].x - tri[0].x,
        tri[1].x - tri[0].x,
        tri[0].x - p.x
    ).cross(v3(
        tri[2].y - tri[0].y,
        tri[1].y - tri[0].y,
        tri[0].y - p.y
    ));

    if u.z.abs() < 1. {
        v3(-1., 1., 1.)
    } else {
        v3(1. - (u.x+u.y) / u.z, u.y / u.z, u.x / u.z)
    }
}

fn triangle(image: &mut Image, z_buf: &mut Vec<f64>, face: &Face, model: &Model) {
    let light_dir = v3(0.,0.,1.);
    let camera_distance = 6.;
    let mut camera_matrix = M4::identity();
    camera_matrix[2][3] = -1./camera_distance;

    let points: Vec<V3> = face.verts.iter()
        .map(|p| {
            let p = camera_matrix * p.extend(1.);
            v3(p.x / p.w, p.y / p.w, p.z / p.w)
        })
        .map(|p| v3(
            (p.x + 1.) * image.width() as f64 /2.,
            (p.y + 1.) * image.height() as f64 /2.,
            p.z
        )).collect();

    let points_z = v3(
        points[0].z,
        points[1].z,
        points[2].z,
    );

    let light = v3(
        face.norms[0].dot(light_dir),
        face.norms[1].dot(light_dir),
        face.norms[2].dot(light_dir)
    );

    let tex_x = v3(
        face.texs[0].x,
        face.texs[1].x,
        face.texs[2].x
    );

    let tex_y = v3(
        face.texs[0].y,
        face.texs[1].y,
        face.texs[2].y,
    );

    let (bbmin, bbmax) = {
        let clamp = v2((image.width()-1) as f64, (image.height()-1) as f64);
        let range  = (v2(::std::f64::MAX, ::std::f64::MAX), v2(::std::f64::MIN, ::std::f64::MIN));
        let (min, max) = points.iter().fold(range, |mut a, p|{
            if p.x < a.0.x { a.0.x = p.x; }
            if p.y < a.0.y { a.0.y = p.y; }
            if p.x > a.1.x { a.1.x = p.x; }
            if p.y > a.1.y { a.1.y = p.y; }
            a
        });
        (
            v2((0_f64).max(min.x.floor()), (0_f64).max(min.y.floor())),
            v2(clamp.x.min(max.x.floor()), clamp.y.min(max.y.floor()))
        )
    };

    for p in V2Box::new(bbmin, bbmax) {
        let screen = barycentric(&points, &p);
        if screen.x < 0. || screen.y < 0. || screen.z < 0. { continue; }

        let image_x = p.x as u32;
        let image_y = p.y as u32;
        let ind = (image_y * image.width() + image_x) as usize;

        let z = points_z.dot(screen);
        if z_buf[ind] < z {
            z_buf[ind] = z;
            let intensity = light.dot(screen);
            if intensity > 0. {
                let x = tex_x.dot(screen);
                let y = tex_y.dot(screen);
                let c: V3 = model.texture.get_f(x,y).into();
                image.set(image_x, image_y, (c * intensity).into());
            }
        }
    }
}

struct V2Box {
    start: V2,
    end: V2,
    cur: V2,
    done: bool,
}

impl V2Box {
    fn new(start: V2, end: V2) -> V2Box {
        let mut cur = start;
        let mut done = false;
        if cur.y > end.y {
            cur.y = start.y;
            cur.x += 1.;
            if cur.x >  end.x {
                done = true;
            }
        }
        V2Box {
            start: start,
            end: end,
            cur: cur,
            done: done,
        }
    }
}

impl Iterator for V2Box {
    type Item = V2;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done { return None; }
        let next = self.cur;
        self.cur.y += 1.;
        if self.cur.y > self.end.y {
            self.cur.y = self.start.y;
            self.cur.x += 1.;
            if self.cur.x >  self.end.x {
                self.done = true;
            }
        }
        Some(next)
    }
}


struct Image {
    pixels: Vec<Color>,
    width: u32,
    height: u32,
}

impl Image {
    fn new(w: u32, h: u32) -> Image {
        let mut pixels = Vec::with_capacity((w * h) as usize);
        pixels.resize((w * h) as usize, Color::black());

        Image {
            pixels: pixels,
            width: w,
            height: h,
        }
    }

    fn from_file<P: AsRef<::std::path::Path>>(path: P) -> Image {
        use image::Pixel;
        let img = image::open(path).unwrap().to_rgba();
        let (width, height) = img.dimensions();
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let (r,g,b,a) = img.get_pixel(x, y).channels4();
                let c = Color::from_argb(a,r,g,b);
                pixels.push(c);
            }
        }

        Image {
            pixels: pixels,
            width: width,
            height: height,
        }
    }

    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }

    fn set(&mut self, x: u32, y: u32, color: Color) {
        let ind = ((y * self.width) + x) as usize;
        if ind < self.pixels.len() {
            self.pixels[ind] = color;
        }
    }

    fn get(&self, x: u32, y: u32) -> Color {
        self.pixels[((y * self.width) + x) as usize]
    }

    fn get_f(&self, x: f64, y: f64) -> Color {
        self.get((x*self.width as f64) as u32, (y*self.height as f64) as u32)
    }

    fn line(&mut self, x0: u32, y0: u32, x1: u32, y1: u32, color: Color) {
        let (mut x0, mut x1, mut y0, mut y1) = (x0 as i32, x1 as i32, y0 as i32, y1 as i32);
        let mut steep = false;
        if (x0 - x1).abs() < (y0 - y1).abs() {
            ::std::mem::swap(&mut x0, &mut y0);
            ::std::mem::swap(&mut x1, &mut y1);
            steep = true;
        }

        if x0 > x1 {
            ::std::mem::swap(&mut x0, &mut x1);
            ::std::mem::swap(&mut y0, &mut y1);
        }

        let dx = x1 - x0;
        let dy = y1 - y0;
        let derror = dy.abs() * 2;
        let mut error = 0;
        let mut y = y0;

        for x in x0..x1 + 1 {
            if steep {
                self.set(y as u32, x as u32, color);
            } else {
                self.set(x as u32, y as u32, color);
            }
            error += derror;
            if error > dx {
                y += if y1 > y0 { 1 } else { -1 };
                error -= dx * 2;
            }
        }
    }

    fn write<P: AsRef<::std::path::Path>>(&self, path: P) -> ::std::io::Result<()> {
        use image::{Pixel, ImageBuffer, ImageRgba8, Rgba, imageops};
        let mut buf = ImageBuffer::new(self.width, self.height);

        for (x, y, p) in buf.enumerate_pixels_mut() {
            let c = self.get(x, y);
            *p = Rgba::from_channels(c.r(), c.g(), c.b(), c.a());
        }

        let buf = imageops::flip_vertical(&buf);
        let mut file = try!(::std::fs::File::create(path));
        let _ = ImageRgba8(buf).save(&mut file, image::PNG);
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Color {
    fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Color {
        Color {
            red: r,
            green: g,
            blue: b,
            alpha: a,
        }
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Color { Self::from_argb(0xff, r, g, b) }

    fn from_argb_f(a: f64, r: f64, g: f64, b: f64) -> Color {
        Color {
            red: (r * 255.) as u8,
            green: (g * 255.) as u8,
            blue: (b * 255.) as u8,
            alpha: (a * 255.) as u8,
        }
    }
    fn from_rgb_f(r: f64, g: f64, b: f64) -> Color { Self::from_argb_f(1., r, g, b) }

    fn red() -> Color { Self::from_rgb(0xff, 0, 0) }

    fn green() -> Color { Self::from_rgb(0, 0xff, 0) }

    fn blue() -> Color { Self::from_rgb(0, 0, 0xff) }

    fn white() -> Color { Self::from_rgb(0xff, 0xff, 0xff) }

    fn black() -> Color { Self::from_rgb(0, 0, 0) }

    fn a(&self) -> u8 { self.alpha }
    fn r(&self) -> u8 { self.red }
    fn g(&self) -> u8 { self.green }
    fn b(&self) -> u8 { self.blue }
    fn a_f(&self) -> f64 { self.alpha as f64 / 255.0 }
    fn r_f(&self) -> f64 { self.red as f64 / 255.0 }
    fn g_f(&self) -> f64 { self.green as f64 / 255.0 }
    fn b_f(&self) -> f64 { self.blue as f64 / 255.0 }
}

impl From<Color> for V3 {
    fn from(c: Color) -> V3 { v3(c.r_f(), c.g_f(), c.b_f()) }
}

impl From<Color> for V4 {
    fn from(c: Color) -> V4 { v4(c.r_f(), c.g_f(), c.b_f(), c.a_f()) }
}

impl From<V3> for Color {
    fn from(v: V3) -> Color { Color::from_rgb_f(v.x, v.y, v.z) }
}

impl From<V4> for Color {
    fn from(v: V4) -> Color { Color::from_argb_f(v.w, v.x, v.y, v.z) }
}


struct Model {
    texture: Image,
    faces: Vec<Face>,
    vertices: Vec<V3>,
}

impl Model {
    fn load<R: ::std::io::Read, P: AsRef<::std::path::Path>>(read: R, tex_path: P) -> Model {
        use ::std::io::{BufReader, BufRead};
        enum ModelObj {
            Vert(V3),
            Tex(V2),
            Norm(V3),
            Face(Vec<Vec<usize>>),
            Invalid,
        }

        let reader = BufReader::new(read);
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
            texture: Image::from_file(tex_path),
            faces: faces,
            vertices: verts,
        }
    }
}

struct Face {
    verts: Vec<V3>,
    texs: Vec<V2>,
    norms: Vec<V3>,
}
