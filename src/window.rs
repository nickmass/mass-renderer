use glium::glutin;
use glium::texture::{texture2d, ClientFormat, PixelValue, RawImage2d};
use glium::{Program, Surface};

use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

use std::cell::Cell;

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
    event_loop: EventLoop<()>,
    closed: Cell<bool>,
}

impl Window {
    pub fn new(width: u32, height: u32) -> Window {
        let event_loop = EventLoop::new();
        let window_size = PhysicalSize::new(width, height);

        let window_builder = WindowBuilder::new()
            .with_inner_size(window_size)
            .with_title("Mass Renderer");

        let context_builder = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_srgb(true)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 2)));

        let display = glium::Display::new(window_builder, context_builder, &event_loop)
            .expect("Unable to create display");

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

        let top_right = Vertex {
            position: [1.0, 1.0],
            tex_coords: [1.0, 0.0],
        };
        let top_left = Vertex {
            position: [-1.0, 1.0],
            tex_coords: [0.0, 0.0],
        };
        let bottom_left = Vertex {
            position: [-1.0, -1.0],
            tex_coords: [0.0, 1.0],
        };
        let bottom_right = Vertex {
            position: [1.0, -1.0],
            tex_coords: [1.0, 1.0],
        };

        let shape = vec![top_right, top_left, bottom_left, bottom_right];

        let program = Program::from_source(&display, vert_shader, frag_shader, None)
            .expect("Unable to create gl program");
        let vertex_buffer =
            glium::VertexBuffer::new(&display, &shape).expect("Unable to create vertex buffer");

        let indicies = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

        Window {
            display,
            indicies,
            program,
            vertex_buffer,
            event_loop,
            closed: Cell::new(false),
        }
    }

    fn process_events(&mut self) {
        let closed = self.closed.get_mut();
        self.event_loop.run_return(|event, _window, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *closed = true,
                _ => (),
            }
            *control_flow = ControlFlow::Exit;
        })
    }

    pub fn render<'a, T: 'a + Clone + PixelValue, I: Into<RawImage2d<'a, T>>>(
        &'a mut self,
        image: I,
    ) {
        let texture = texture2d::Texture2d::new(&self.display, image.into()).unwrap();
        let uniforms = uniform! {
            tex: texture.sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest),
        };

        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target
            .draw(
                &self.vertex_buffer,
                &self.indicies,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();
        self.display.gl_window().window().request_redraw();
        self.process_events();
    }

    pub fn is_closed(&self) -> bool {
        self.closed.get()
    }
}

use crate::renderer::{Color as RColor, Surface as RSurface, Texture as RTexture};

impl<'a, 'b, T> Into<RawImage2d<'a, (u8, u8, u8, u8)>> for &'b RTexture<T>
where
    T: Into<RColor> + Copy,
{
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
