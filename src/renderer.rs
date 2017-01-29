use ::model::{Model, Face};

use ::{
    V2,
    V3,
    V4,
    M4,
    v2,
    v3,
    v4,
    SquareMatrix,
    InnerSpace,
    image,
};

pub struct Renderer {
    display_buf: Surface<V3>,
    z_buf: Surface<f64>,
    width: u32,
    height: u32,
    viewport: M4,
    projection: M4,
    modelview: M4,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Renderer {
        Renderer {
            display_buf: Surface::new(width, height, v3(0.,0.,0.)),
            z_buf: Surface::new(width, height, ::std::f64::MIN),
            width: width,
            height: height,
            viewport: M4::identity(),
            projection: M4::identity(),
            modelview: M4::identity(),
        }
    }

    pub fn clear(&mut self, color: V3) {
        self.display_buf = Surface::new(self.width, self.height, color);
        self.z_buf = Surface::new(self.width, self.height, ::std::f64::MIN);
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
        let points: Vec<V3> = (0..3).map(|i| shader.vertex(ctx, face, i)).collect();

        let points_z = v3(
            points[0].z,
            points[1].z,
            points[2].z,
        );

        let (bbmin, bbmax) = {
            let clamp = v2((self.width-1) as f64, (self.height-1) as f64);
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
            let coords = barycentric(&points, &p);
            if coords.x < 0. || coords.y < 0. || coords.z < 0. { continue; }

            let image_x = p.x as u32;
            let image_y = p.y as u32;

            let z = points_z.dot(coords);
            if self.z_buf.get(image_x, image_y) < z {
                shader.fragment(ctx, coords).map(|c| {
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
        projection[2][3] = -1. / camera_distance;

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

pub fn matrix_transform(v: &V3, m: &M4) -> V3 {
    let v = m * v.extend(1.);
    v3(v.x / v.w, v.y / v.w, v.z / v.w)
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

pub struct RenderContext<'a> {
    viewport: M4,
    projection: M4,
    modelview: M4,
    model: &'a Model,
}

pub trait Shader {
    fn prepare(&mut self, ctx: &RenderContext);
    fn vertex(&mut self, ctx: &RenderContext, face: &Face, vert: usize) -> V3;
    fn fragment(&mut self, ctx: &RenderContext, coords: V3) -> Option<V3>;
}

pub struct DefaultShader {
    light_dir: V3,
    uv_x: V3,
    uv_y: V3,
    intensity: V3,
    transform: M4,
}

impl DefaultShader {
    pub fn new(light_dir: V3) -> DefaultShader {
        DefaultShader {
            light_dir: light_dir.normalize(),
            uv_x: V3::unit_z(),
            uv_y: V3::unit_z(),
            intensity: V3::unit_z(),
            transform: M4::identity(),
        }
    }
}

impl Shader for DefaultShader {
    fn prepare(&mut self, ctx: &RenderContext) {
        self.transform = ctx.viewport * ctx.projection * ctx.modelview;
    }

    fn vertex(&mut self, ctx: &RenderContext, face: &Face, vert: usize) -> V3 {
        self.uv_x[vert] = face.texs[vert].x;
        self.uv_y[vert] = face.texs[vert].y;
        self.intensity[vert] = face.norms[vert].dot(self.light_dir);
        matrix_transform(&face.verts[vert], &self.transform)
    }

    fn fragment(&mut self, ctx: &RenderContext, coords: V3) -> Option<V3> {
        let intensity = self.intensity.dot(coords).max(0.0);
        let uv = v2(self.uv_x.dot(coords), self.uv_y.dot(coords));
        Some(ctx.model.diffuse(uv).truncate() * intensity)
    }
}

pub struct Surface<T> {
    pixels: Vec<T>,
    width: u32,
    height: u32,
}

impl<T> Surface<T> {
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }

    fn set(&mut self, x: u32, y: u32, color: T) {
        let ind = ((y * self.width) + x) as usize;
        if ind < self.pixels.len() {
            self.pixels[ind] = color;
        }
    }
}

impl<T: Copy> Surface<T> {
    pub fn new(w: u32, h: u32, default: T) -> Surface<T> {
        let mut pixels = Vec::with_capacity((w * h) as usize);
        pixels.resize((w * h) as usize, default);

        Surface {
            pixels: pixels,
            width: w,
            height: h,
        }
    }

    pub fn get(&self, x: u32, y: u32) -> T {
        self.pixels[((y * self.width) + x) as usize]
    }

    pub fn get_f(&self, x: f64, y: f64) -> T {
        self.get((x*self.width as f64) as u32, (y*self.height as f64) as u32)
    }

    fn line(&mut self, x0: u32, y0: u32, x1: u32, y1: u32, color: T) {
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

impl Surface<V4> {
    pub fn from_file<P: AsRef<::std::path::Path>>(path: P) -> Surface<V4> {
        use image::Pixel;
        let img = image::open(path).unwrap().to_rgba();
        let (width, height) = img.dimensions();
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let (r,g,b,a) = img.get_pixel(x, y).channels4();
                let c = v4(r as f64 / 255., g as f64 / 255., b as f64 / 255., a as f64 / 255.);
                pixels.push(c);
            }
        }

        Surface {
            pixels: pixels,
            width: width,
            height: height,
        }
    }
}

trait WriteSurface {
    fn write<P: AsRef<::std::path::Path>>(&self, path: P) -> ::std::io::Result<()>;
}

impl<T: Into<Color> + Clone + Copy> WriteSurface for Surface<T> {
    fn write<P: AsRef<::std::path::Path>>(&self, path: P) -> ::std::io::Result<()> {
        use image::{Pixel, ImageBuffer, ImageRgba8, Rgba, imageops};
        let mut buf = ImageBuffer::new(self.width, self.height);

        for (x, y, p) in buf.enumerate_pixels_mut() {
            let c = self.get(x, y).into();
            *p = Rgba::from_channels(c.r(), c.g(), c.b(), c.a());
        }

        let buf = imageops::flip_vertical(&buf);
        let mut file = try!(::std::fs::File::create(path));
        let _ = ImageRgba8(buf).save(&mut file, image::PNG);
        Ok(())
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

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color { Self::from_argb(0xff, r, g, b) }

    pub fn from_argb_f(a: f64, r: f64, g: f64, b: f64) -> Color {
        Color {
            red: (r * 255.) as u8,
            green: (g * 255.) as u8,
            blue: (b * 255.) as u8,
            alpha: (a * 255.) as u8,
        }
    }
    pub fn from_rgb_f(r: f64, g: f64, b: f64) -> Color { Self::from_argb_f(1., r, g, b) }

    pub fn a(&self) -> u8 { self.alpha }
    pub fn r(&self) -> u8 { self.red }
    pub fn g(&self) -> u8 { self.green }
    pub fn b(&self) -> u8 { self.blue }
}

impl From<f64> for Color {
    fn from(v: f64) -> Color { Color::from_rgb_f(v, v, v) }
}

impl From<V3> for Color {
    fn from(v: V3) -> Color { Color::from_rgb_f(v.x, v.y, v.z) }
}

impl From<V4> for Color {
    fn from(v: V4) -> Color { Color::from_argb_f(v.w, v.x, v.y, v.z) }
}
