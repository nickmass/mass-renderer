extern crate image;

fn main() {
    let obj = ::std::fs::File::open("../../tinyrenderer/obj/african_head/african_head.obj");
    let model = Model::load(obj.unwrap());
    let (width, height) = (1024, 1024);
    let mut image = Image::new(width, height);

    let (x_scale, y_scale) = (width as f64 / 2., height as f64 / 2.);
    for face in model.faces.iter() {
        for l in 0..3 {
            let v0 = face.verts[l];
            let v1 = face.verts[(l + 1) % 3];
            let x0 = ((v0.x() + 1.) * x_scale) as u32;
            let x1 = ((v1.x() + 1.) * x_scale) as u32;
            let y0 = ((v0.y() + 1.) * y_scale) as u32;
            let y1 = ((v1.y() + 1.) * y_scale) as u32;

            image.line(x0, y0, x1, y1, Color::white());
        }
    }

    image.line(100,0,0,100, Color::blue());
    image.line(20,13,40,80, Color::red());
    image.line(80,40,13,20, Color::red());

    let _ = image.write("image.png");
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

    fn set(&mut self, x: u32, y: u32, color: Color) {
        let ind = ((y * self.width) + x) as usize;
        if ind < self.pixels.len() {
            self.pixels[ind] = color;
        }
    }

    fn get(&self, x: u32, y: u32) -> Color {
        self.pixels[((y * self.width) + x) as usize]
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


#[derive(Clone, Copy, Debug)]
struct V3(f64, f64, f64);
impl V3 {
    fn x(&self) -> f64 { self.0 }
    fn y(&self) -> f64 { self.1 }
    fn z(&self) -> f64 { self.2 }
}

struct Model {
    faces: Vec<Face>,
    vertices: Vec<V3>,
}

impl Model {
    fn load<R: ::std::io::Read>(read: R) -> Model {
        use ::std::io::{BufReader, BufRead};
        enum ModelObj {
            Vert(V3),
            Face(Vec<Vec<usize>>),
            None,
        }

        let mut reader = BufReader::new(read);
        let (verts, faces) = reader.lines().map(|l| {
            let l = l.unwrap();
            let parts: Vec<&str> = l.split(' ').collect();
            if parts.len() == 0 { return ModelObj::None; }

            match parts[0] {
                "v" => {
                    if parts.len() != 4 {
                        ModelObj::None
                    } else {
                        ModelObj::Vert(V3(
                            parts[1].parse().unwrap(),
                            parts[2].parse().unwrap(),
                            parts[3].parse().unwrap()))
                    }
                },
                "f" => {
                    if parts.len() != 4 {
                        ModelObj::None
                    } else {
                        let faces = parts[1..].iter().map(|p| {
                            p.split('/').map(|x| x.parse().unwrap()).collect()
                        }).collect();

                        ModelObj::Face(faces)
                    }
                },
                _ => ModelObj::None,
            }
        }).fold((Vec::new(), Vec::new()), |mut col, item| {
            match item {
                ModelObj::Vert(v) => col.0.push(v),
                ModelObj::Face(f) => col.1.push(f),
                ModelObj::None => ()
            };

            col
        });

        let faces = faces.iter().map(|f| {
            let face = f.iter().map(|t| {
                *verts.get(t[0] - 1).unwrap()
            }).collect();

            Face { verts: face }
        }).collect();

        Model {
            faces: faces,
            vertices: verts,
        }
    }
}

struct Face {
    verts: Vec<V3>
}
