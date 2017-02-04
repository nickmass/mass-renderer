use ::glium;
use glium::Surface;
use glium::DisplayBuild;
use glium::texture::{
    PixelValue,
    RawImage2d,
    ClientFormat,
    texture2d,
};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub struct Window {
    display: glium::Display,
    indicies: glium::index::NoIndices,
    program: glium::Program,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    closed: bool,
}

impl Window {
    pub fn new(width: u32, height: u32) -> Window {
        let display = glium::glutin::WindowBuilder::new()
            .with_dimensions(width, height)
            .with_title(format!("Mass Renderer"))
            .build_glium()
            .unwrap();

        let vert_shader = r#"
            #version 140

            in vec2 position;
            in vec2 tex_coords;

            out vec2 v_tex_coords;

            void main() {
                v_tex_coords = tex_coords;
                gl_Position = vec4(position, 0.0, 1.0);
            }
        "#;

        let frag_shader = r#"
            #version 140

            in vec2 v_tex_coords;
            out vec4 color;

            uniform sampler2D tex;

            void main() {
                color = texture(tex, v_tex_coords);
            }
        "#;

        let top_right = Vertex { position: [1.0, 1.0], tex_coords: [1.0, 0.0] };
        let top_left = Vertex { position: [-1.0, 1.0], tex_coords: [0.0, 0.0] };
        let bottom_left = Vertex { position: [-1.0, -1.0],  tex_coords: [0.0, 1.0] };
        let bottom_right = Vertex { position: [1.0, -1.0], tex_coords: [1.0, 1.0] };

        let shape = vec![top_right, top_left, bottom_left, bottom_right];

        let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
        let indicies = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

        let program = glium::Program::from_source(&display, vert_shader, frag_shader, None).unwrap();

        Window {
            display: display,
            indicies: indicies,
            program: program,
            vertex_buffer: vertex_buffer,
            closed: false,
        }
    }

    fn process_events(&mut self) {
        for ev in self.display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => self.closed = true,
                _ => ()
            }
        }
    }

    pub fn render<'a, T: 'a + Clone + PixelValue, I: Into<RawImage2d<'a, T>>>(&'a mut self, image: I) {
        let texture = texture2d::Texture2d::new(&self.display, image.into()).unwrap();
        let uniforms = uniform! {
            tex: texture.sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest),
        };

        let mut target = self.display.draw(); 
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&self.vertex_buffer,
                    &self.indicies,
                    &self.program,
                    &uniforms,
                    &Default::default()).unwrap();
        target.finish().unwrap();
        self.process_events();
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

use ::renderer::{
    Texture as RTexture,
    Color as RColor,
    Surface as RSurface,
};

impl<'a, 'b, T> Into<RawImage2d<'a, (u8, u8, u8, u8)>> for &'b RTexture<T> where T: Into<RColor> + Copy {
    fn into(self) -> RawImage2d<'a, (u8, u8, u8, u8)> {
        let mut data = Vec::new();
        for y in 0..self.height() {
            for x in 0..self.width() {
                let c = self.get(x, self.height() - y - 1).into().to_linear();
                data.push((c.r(), c.g(), c.b(), c.a()));
            }
        }
        RawImage2d {
            width: self.width(),
            height: self.height(),
            format: ClientFormat::U8U8U8U8,
            data: data.into(),
        }
    }
}
