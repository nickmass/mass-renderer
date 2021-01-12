use crate::model::{Face, Model};

use crate::{image, v2, v3, v4, InnerSpace, SquareMatrix, M4, V2, V3, V4};

pub struct Renderer {
    display_buf: Texture<V3>,
    z_buf: Texture<f64>,
    width: u32,
    height: u32,
    pub viewport: M4,
    pub projection: M4,
    pub modelview: M4,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Renderer {
        Renderer {
            display_buf: Texture::new(width, height, v3(0., 0., 0.)),
            z_buf: Texture::new(width, height, ::std::f64::MIN),
            width,
            height,
            viewport: M4::identity(),
            projection: M4::identity(),
            modelview: M4::identity(),
        }
    }

    pub fn display_buffer<'a>(&'a self) -> &'a Texture<V3> {
        &self.display_buf
    }

    pub fn z_buffer<'a>(&'a self) -> &'a Texture<f64> {
        &self.z_buf
    }

    pub fn clear(&mut self, color: V3) {
        self.display_buf = Texture::new(self.width, self.height, color);
        self.z_buf = Texture::new(self.width, self.height, ::std::f64::MIN);
    }

    pub fn render<S: Shader>(&mut self, shader: &mut S, model: &Model) {
        let ctx = RenderContext {
            viewport: self.viewport,
            projection: self.projection,
            modelview: self.modelview,
            model: &model,
        };
        shader.prepare(&ctx);
        for face in model.faces() {
            self.triangle(shader, &ctx, &face);
        }
    }

    pub fn dump(&self) {
        let _ = self.display_buf.write("image.png");
        let _ = self.z_buf.write("z_buf.png");
    }

    fn triangle<S: Shader>(&mut self, shader: &mut S, ctx: &RenderContext, face: &Face) {
        let points: Vec<V4> = (0..3).map(|i| shader.vertex(ctx, face, i)).collect();

        let points_z = v3(points[0].z, points[1].z, points[2].z);

        let points_w = v3(points[0].w, points[1].w, points[2].w);

        let points: Vec<V3> = points
            .iter()
            .map(|p| v3(p.x / p.w, p.y / p.w, p.z / p.w))
            .collect();

        let (bbmin, bbmax) = {
            let clamp = v2((self.width - 1) as f64, (self.height - 1) as f64);
            let range = (
                v2(::std::f64::MAX, ::std::f64::MAX),
                v2(::std::f64::MIN, ::std::f64::MIN),
            );
            let (min, max) = points.iter().fold(range, |mut a, p| {
                if p.x < a.0.x {
                    a.0.x = p.x;
                }
                if p.y < a.0.y {
                    a.0.y = p.y;
                }
                if p.x > a.1.x {
                    a.1.x = p.x;
                }
                if p.y > a.1.y {
                    a.1.y = p.y;
                }
                a
            });
            (
                v2((0_f64).max(min.x.floor()), (0_f64).max(min.y.floor())),
                v2(clamp.x.min(max.x.floor()), clamp.y.min(max.y.floor())),
            )
        };

        for p in V2Box::new(bbmin, bbmax) {
            let coords = barycentric(&points, &p);
            if coords.x < 0. || coords.y < 0. || coords.z < 0. {
                continue;
            }

            let clip = v3(
                coords.x / points_w.x,
                coords.y / points_w.y,
                coords.z / points_w.z,
            );

            let clip = clip / (clip.x + clip.y + clip.z);
            let z = points_z.dot(clip);

            let image_x = p.x as u32;
            let image_y = p.y as u32;
            if self.z_buf.get(image_x, image_y) < z {
                shader.fragment(ctx, clip).map(|c| {
                    self.display_buf.set(image_x, image_y, c);
                    self.z_buf.set(image_x, image_y, z);
                });
            }
        }
    }

    pub fn viewport(&mut self, x: f64, y: f64, width: f64, height: f64) {
        let mut viewport = M4::identity();
        viewport[0][0] = width / 2.;
        viewport[1][1] = height / 2.;
        viewport[3][0] = width / 2. + x;
        viewport[3][1] = height / 2. + y;

        viewport[2][2] = 0.5;
        viewport[3][2] = 0.5;

        self.viewport = viewport;
    }

    pub fn projection(&mut self, camera_distance: f64) {
        let mut projection = M4::identity();
        projection[2][3] = camera_distance;

        self.projection = projection;
    }

    pub fn lookat(&mut self, eye: V3, center: V3, up: V3) {
        let z = (eye - center).normalize();
        let x = up.cross(z).normalize();
        let y = z.cross(x).normalize();
        let mut modelview = M4::identity();
        for i in 0..3 {
            modelview[i][0] = x[i];
            modelview[i][1] = y[i];
            modelview[i][2] = z[i];
            modelview[3][i] = -center[i];
        }

        self.modelview = modelview;
    }
}

pub fn matrix_transform(v: V3, m: M4) -> V3 {
    let v = m * v.extend(1.);
    v3(v.x / v.w, v.y / v.w, v.z / v.w)
}

fn barycentric(tri: &Vec<V3>, p: &V2) -> V3 {
    let u = v3(tri[2].x - tri[0].x, tri[1].x - tri[0].x, tri[0].x - p.x).cross(v3(
        tri[2].y - tri[0].y,
        tri[1].y - tri[0].y,
        tri[0].y - p.y,
    ));

    if u.z.abs() < 1. {
        v3(-1., 1., 1.)
    } else {
        v3(1. - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z)
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
            if cur.x > end.x {
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
        if self.done {
            return None;
        }
        let next = self.cur;
        self.cur.y += 1.;
        if self.cur.y > self.end.y {
            self.cur.y = self.start.y;
            self.cur.x += 1.;
            if self.cur.x > self.end.x {
                self.done = true;
            }
        }
        Some(next)
    }
}

pub struct RenderContext<'a> {
    pub viewport: M4,
    pub projection: M4,
    pub modelview: M4,
    pub model: &'a Model,
}

pub trait Shader {
    fn prepare(&mut self, ctx: &RenderContext);
    fn vertex(&mut self, ctx: &RenderContext, face: &Face, vert: usize) -> V4;
    fn fragment(&mut self, ctx: &RenderContext, coords: V3) -> Option<V3>;
}

#[derive(Clone)]
pub struct Texture<T> {
    pixels: Vec<T>,
    width: u32,
    height: u32,
}

impl<T: Copy> Texture<T> {
    pub fn new(w: u32, h: u32, default: T) -> Texture<T> {
        let mut pixels = Vec::with_capacity((w * h) as usize);
        pixels.resize((w * h) as usize, default);

        Texture {
            pixels,
            width: w,
            height: h,
        }
    }

    pub fn line(&mut self, x0: u32, y0: u32, x1: u32, y1: u32, color: T) {
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
}

impl Texture<V4> {
    pub fn from_file<P: AsRef<::std::path::Path>>(path: P) -> Texture<V4> {
        use image::Pixel;
        let img = image::open(path).unwrap().to_rgba();
        let (width, height) = img.dimensions();
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let (r, g, b, a) = img.get_pixel(x, y).channels4();
                let c = v4(
                    r as f64 / 255.,
                    g as f64 / 255.,
                    b as f64 / 255.,
                    a as f64 / 255.,
                );
                pixels.push(c.into());
            }
        }

        Texture {
            pixels,
            width,
            height,
        }
    }
}

impl<T: Into<Color> + Clone + Copy> Texture<T> {
    fn write<P: AsRef<::std::path::Path>>(&self, path: P) -> ::std::io::Result<()> {
        use image::{imageops, ImageBuffer, ImageRgba8, Pixel, Rgba};
        let mut buf = ImageBuffer::new(self.width, self.height);

        for (x, y, p) in buf.enumerate_pixels_mut() {
            let c = self.get(x, y).into();
            *p = Rgba::from_channels(c.r(), c.g(), c.b(), c.a());
        }

        let buf = imageops::flip_vertical(&buf);
        let mut file = ::std::fs::File::create(path)?;
        let _ = ImageRgba8(buf).save(&mut file, image::PNG);
        Ok(())
    }
}

impl<T: Copy> Surface for Texture<T> {
    type Item = T;
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }

    fn get(&self, x: u32, y: u32) -> T {
        self.pixels[((y * self.width) + x) as usize]
    }

    fn get_f(&self, x: f64, y: f64) -> T {
        self.get(
            (x * (self.width - 1) as f64) as u32,
            (y * (self.height - 1) as f64) as u32,
        )
    }

    fn set(&mut self, x: u32, y: u32, color: T) {
        let ind = ((y * self.width) + x) as usize;
        if ind < self.pixels.len() {
            self.pixels[ind] = color;
        }
    }
}

pub trait Surface {
    type Item;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn get(&self, x: u32, y: u32) -> Self::Item;
    fn get_f(&self, x: f64, y: f64) -> Self::Item;
    fn set(&mut self, x: u32, y: u32, value: Self::Item);
}

pub struct BilinearSampler<T> {
    inner: T,
}

impl<T: Surface> BilinearSampler<T> {
    pub fn new(image: T) -> BilinearSampler<T> {
        BilinearSampler { inner: image }
    }
}

impl<
        T: Surface<Item = U>,
        U: ::std::ops::Mul<f64, Output = U> + ::std::ops::Add<U, Output = U>,
    > Surface for BilinearSampler<T>
{
    type Item = T::Item;
    fn width(&self) -> u32 {
        self.inner.width()
    }
    fn height(&self) -> u32 {
        self.inner.height()
    }
    fn get(&self, x: u32, y: u32) -> T::Item {
        self.inner.get(x, y)
    }
    fn get_f(&self, x: f64, y: f64) -> T::Item {
        let x = x.max(0.0).min(1.0) * (self.width() - 1) as f64;
        let y = y.max(0.0).min(1.0) * (self.height() - 1) as f64;
        let x0 = x.floor();
        let x1 = x.ceil();
        let y0 = y.floor();
        let y1 = y.ceil();

        let t = x - x0;
        let v0 = self.inner.get(x0 as u32, y0 as u32);
        let v1 = self.inner.get(x1 as u32, y0 as u32);
        let r0 = v0 * (1. - t) + v1 * t;

        let v0 = self.inner.get(x0 as u32, y1 as u32);
        let v1 = self.inner.get(x1 as u32, y1 as u32);
        let r1 = v0 * (1. - t) + v1 * t;

        let t = y - y0;

        r0 * (1. - t) + r1 * t
    }
    fn set(&mut self, x: u32, y: u32, value: T::Item) {
        self.inner.set(x, y, value);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Color {
    pub fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Color {
        Color {
            red: r,
            green: g,
            blue: b,
            alpha: a,
        }
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Self::from_argb(0xff, r, g, b)
    }

    pub fn from_argb_f(a: f64, r: f64, g: f64, b: f64) -> Color {
        Color {
            red: (r * 255.) as u8,
            green: (g * 255.) as u8,
            blue: (b * 255.) as u8,
            alpha: (a * 255.) as u8,
        }
    }
    pub fn from_rgb_f(r: f64, g: f64, b: f64) -> Color {
        Self::from_argb_f(1., r, g, b)
    }

    pub fn a(&self) -> u8 {
        self.alpha
    }
    pub fn r(&self) -> u8 {
        self.red
    }
    pub fn g(&self) -> u8 {
        self.green
    }
    pub fn b(&self) -> u8 {
        self.blue
    }

    pub fn to_linear(self) -> Color {
        let a = self.a() as f64 / 255.;
        let mut c: V3 = self.into();

        for i in 0..3 {
            c[i] = if c[i] <= 0.04045 {
                c[i] / 12.92
            } else {
                ((c[i] + 0.055) / 1.055).powf(2.4)
            };

            c[i] = c[i].max(0.0).min(1.0);
        }

        Color::from_argb_f(a, c.x, c.y, c.z)
    }

    pub fn to_srgb(self) -> Color {
        let a = self.a() as f64 / 255.;
        let mut c: V3 = self.into();

        for i in 0..3 {
            c[i] = if c[i] <= 0.0031308 {
                12.92 * c[i]
            } else {
                1.055 * c[i].powf(0.41666) - 0.055
            };

            c[i] = c[i].max(0.0).min(1.0);
        }

        Color::from_argb_f(a, c.x, c.y, c.z)
    }
}

impl From<Color> for V3 {
    fn from(c: Color) -> V3 {
        v3(
            c.r() as f64 / 255.0,
            c.g() as f64 / 255.0,
            c.b() as f64 / 255.0,
        )
    }
}

impl From<Color> for V4 {
    fn from(c: Color) -> V4 {
        v4(
            c.r() as f64 / 255.0,
            c.g() as f64 / 255.0,
            c.b() as f64 / 255.0,
            c.a() as f64 / 255.0,
        )
    }
}

impl From<f64> for Color {
    fn from(v: f64) -> Color {
        Color::from_rgb_f(v, v, v)
    }
}

impl From<V3> for Color {
    fn from(v: V3) -> Color {
        Color::from_rgb_f(v.x, v.y, v.z)
    }
}

impl From<V4> for Color {
    fn from(v: V4) -> Color {
        Color::from_argb_f(v.w, v.x, v.y, v.z)
    }
}
