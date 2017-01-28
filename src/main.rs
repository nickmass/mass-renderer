extern crate image;

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
    let u = v3(tri[2].x()-tri[0].x(), tri[1].x()-tri[0].x(), tri[0].x()-p.x())
        .cross(v3(tri[2].y()-tri[0].y(), tri[1].y()-tri[0].y(), tri[0].y()-p.y()));
    if u.z().abs() < 1. { return v3(-1.,1.,1.); }
    v3(1.-(u.x()+u.y())/u.z(), u.y()/u.z(), u.x()/u.z())
}

fn triangle(image: &mut Image, z_buf: &mut Vec<f64>, face: &Face, model: &Model) {
    let light_dir = v3(0.,0.,1.);
    let points: Vec<V3> = face.verts.iter().map(|p| {
        v3((p.x() + 1.) * image.width() as f64 /2.,
         (p.y() + 1.) * image.height() as f64 /2., p.z())
    }).collect();

    let light = v3(
        face.norms[0].dot(light_dir),
        face.norms[1].dot(light_dir),
        face.norms[2].dot(light_dir)
    );

    let (tex_x, tex_y) = {
        let x = v3(
            face.texs[0].x(),
            face.texs[1].x(),
            face.texs[2].x()
        );

        let y = v3(
            face.texs[0].y(),
            face.texs[1].y(),
            face.texs[2].y(),
        );

        (x,y)
    };

    let mut bbmin = v2(::std::f64::MAX, ::std::f64::MAX);
    let mut bbmax = v2(::std::f64::MIN, ::std::f64::MIN);
    let clamp = v2((image.width()-1) as f64, (image.height()-1) as f64);

    for i in 0..3 {
        for j in 0..2 {
            bbmin[j] = (0.0 as f64).max(bbmin[j].min(points[i][j])).floor();
            bbmax[j] = clamp[j].min(bbmax[j].max(points[i][j])).floor();
        }
    }

    let mut p = v2(bbmin.x(), bbmin.y());
    while p.x() <= bbmax.x() {
        while p.y() <= bbmax.y() {
            let screen = barycentric(&points, &p);
            if screen.x() < 0. || screen.y() < 0. || screen.z() < 0. {
                p[1] = p[1] + 1.0;
                continue;
            }
            let mut z = 0.0;
            for i in 0..3 {
                z += points[i].z() * screen[i];
            }

            let screen_x = p.x() as u32;
            let screen_y = p.y() as u32;
            let ind = (screen_y * image.width() + screen_x) as usize;

            if z_buf[ind] < z {
                z_buf[ind] = z;
                let x = tex_x.dot(screen);
                let y = tex_y.dot(screen);
                let c = model.texture.get_f(x,y);
                let intensity = (light.dot(screen) + 1.) / 2.;
                image.set(screen_x, screen_y, Color::from_rgb(
                    (c.r() as f64 * intensity) as u8,
                    (c.g() as f64 * intensity) as u8,
                    (c.b() as f64  * intensity) as u8
                ));
            }

            p[1] = p[1] + 1.0;
        }
        p[0] = p[0] + 1.0;
        p[1] = bbmin.y();
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
        use image::imageops::flip_vertical;
        use image::Pixel;
        let img = image::open(path).unwrap().to_rgba();
        //let img = flip_vertical(&img);
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
}

fn v3(x: f64, y: f64, z: f64) -> V3 {
    V3::new(x,y,z)
}

#[derive(Clone, Copy, Debug)]
struct V3 {
    elems: [f64;3],
}
impl V3 {
    fn new(x: f64, y: f64, z: f64) -> V3 {
        V3 {
            elems: [x, y, z]
        }
    }
    fn x(&self) -> f64 { self.elems[0] }
    fn y(&self) -> f64 { self.elems[1] }
    fn z(&self) -> f64 { self.elems[2] }

    fn magnitude(self) -> f64 {
        self.elems.iter().map(|i| i*i).sum::<f64>().sqrt()
    }

    fn normalize(self) -> V3 {
        let mag = self.magnitude();
        V3::new(self.x()/mag, self.y()/mag, self.z()/mag)
    }

    fn dot(self, other: V3) -> f64 {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a*b).sum()
    }

    fn cross(self, other: V3) -> V3 {
        V3::new((self.y()*other.z())-(self.z()*other.y()),
               (self.z()*other.x())-(self.x()*other.z()),
               (self.x()*other.y()-(self.y()*other.x())))
    }
}

impl ::std::ops::Index<usize> for V3 {
    type Output = f64;
    fn index(&self, index: usize) -> &f64 {
        self.elems.index(index)
    }
}

impl ::std::ops::IndexMut<usize> for V3 {
    fn index_mut(&mut self, index: usize) -> &mut f64 {
        self.elems.index_mut(index)
    }
}

impl ::std::iter::FromIterator<f64> for V3 {
    fn from_iter<T>(iter: T) -> V3 where T: IntoIterator<Item=f64> {
        let mut iter = iter.into_iter();
        V3::new(iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap())
    }
}

impl ::std::convert::From<f64> for V3 {
    fn from(item: f64) -> V3 {
        V3::new(item, item, item)
    }
}

impl ::std::ops::Add for V3 {
    type Output = V3;
    fn add(self, other: V3) -> V3 {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a+b).collect()
    }
}

impl ::std::ops::Sub for V3 {
    type Output = V3;
    fn sub(self, other: V3) -> V3 {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a-b).collect()
    }
}

impl ::std::ops::Mul for V3 {
    type Output = V3;
    fn mul(self, other: V3) -> V3 {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a*b).collect()
    }
}

impl ::std::ops::Div for V3 {
    type Output = V3;
    fn div(self, other: V3) -> V3 {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a/b).collect()
    }
}

fn v2(x: f64, y: f64) -> V2 {
    V2::new(x,y)
}

#[derive(Clone, Copy, Debug)]
struct V2 {
    elems: [f64;2],
}
impl V2 {
    fn new(x: f64, y: f64) -> V2 {
        V2 {
            elems: [x, y]
        }
    }
    fn x(&self) -> f64 { self.elems[0] }
    fn y(&self) -> f64 { self.elems[1] }

    fn magnitude(self) -> f64 {
        self.elems.iter().map(|i| i*i).sum::<f64>().sqrt()
    }

    fn normalize(self) -> V2 {
        let mag = self.magnitude();
        V2::new(self.x()/mag, self.y()/mag)
    }

    fn dot(self, other: V2) -> f64 {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a*b).sum()
    }
}

impl ::std::iter::FromIterator<f64> for V2 {
    fn from_iter<T>(iter: T) -> V2 where T: IntoIterator<Item=f64> {
        let mut iter = iter.into_iter();
        V2::new(iter.next().unwrap(), iter.next().unwrap())
    }
}

impl ::std::convert::From<f64> for V2 {
    fn from(item: f64) -> V2 {
        V2::new(item, item)
    }
}

impl ::std::convert::From<V3> for V2 {
    fn from(item: V3) -> V2 {
        V2::new(item.x(), item.y())
    }
}

impl ::std::ops::Index<usize> for V2 {
    type Output = f64;
    fn index(&self, index: usize) -> &f64 {
        self.elems.index(index)
    }
}

impl ::std::ops::IndexMut<usize> for V2 {
    fn index_mut(&mut self, index: usize) -> &mut f64 {
        self.elems.index_mut(index)
    }
}

impl ::std::ops::Add for V2 {
    type Output = V2;
    fn add(self, other: V2) -> V2 {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a+b).collect()
    }
}

impl ::std::ops::Sub for V2 {
    type Output = V2;
    fn sub(self, other: V2) -> V2 {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a-b).collect()
    }
}

impl ::std::ops::Mul for V2 {
    type Output = V2;
    fn mul(self, other: V2) -> V2 {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a*b).collect()
    }
}

impl ::std::ops::Div for V2 {
    type Output = V2;
    fn div(self, other: V2) -> V2 {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a/b).collect()
    }
}

fn v2i(x: i32, y: i32) -> V2i {
    V2i::new(x,y)
}

#[derive(Clone, Copy, Debug)]
struct V2i {
    elems: [i32;2],
}
impl V2i {
    fn new(x: i32, y: i32) -> V2i {
        V2i {
            elems: [x, y]
        }
    }
    fn x(&self) -> i32 { self.elems[0] }
    fn y(&self) -> i32 { self.elems[1] }
}

impl ::std::iter::FromIterator<i32> for V2i {
    fn from_iter<T>(iter: T) -> V2i where T: IntoIterator<Item=i32> {
        let mut iter = iter.into_iter();
        V2i::new(iter.next().unwrap(), iter.next().unwrap())
    }
}

impl ::std::convert::From<i32> for V2i {
    fn from(item: i32) -> V2i {
        V2i::new(item, item)
    }
}

impl ::std::ops::Add for V2i {
    type Output = V2i;
    fn add(self, other: V2i) -> V2i {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a+b).collect()
    }
}

impl ::std::ops::Sub for V2i {
    type Output = V2i;
    fn sub(self, other: V2i) -> V2i {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a-b).collect()
    }
}

impl ::std::ops::Mul for V2i {
    type Output = V2i;
    fn mul(self, other: V2i) -> V2i {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a*b).collect()
    }
}

impl ::std::ops::Div for V2i {
    type Output = V2i;
    fn div(self, other: V2i) -> V2i {
        self.elems.iter().zip(other.elems.iter()).map(|(a,b)| a/b).collect()
    }
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

        let mut reader = BufReader::new(read);
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
