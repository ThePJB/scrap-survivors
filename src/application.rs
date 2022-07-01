use glow::*;
use crate::lib::kinput::*;
use crate::lib::kimg::*;
use crate::krenderer::*;
use crate::game::*;
use glutin::event::{Event, WindowEvent};

pub struct Application {
    gl: glow::Context,
    window: glutin::WindowedContext<glutin::PossiblyCurrent>,

    renderer: KRenderer,
    event_aggregator: EventAggregator,

    pub xres: f32,
    pub yres: f32,

    game: Game,
}

pub fn load_file(paths: &[&str]) -> String {
    for path in paths {
        if let Ok(s) = std::fs::read_to_string(path) {
            return s
        }
    }
    panic!("couldn't find any of {:?}", paths)
}

impl Application {
    pub fn new(event_loop: &glutin::event_loop::EventLoop<()>) -> Application {
        let default_xres = 1600.0;
        let default_yres = 1600.0;

        let (gl, window) = unsafe { opengl_boilerplate(default_xres, default_yres, event_loop) };
        
        let uvv = &[
            "src/uv.vert",
            "../../src/uv.vert",
            "uv.vert",
        ];
        let uvf = &[
            "src/uv.frag",
            "../../src/uv.frag",
            "uv.frag",
        ];
                
        let uv_shader = make_shader(&gl, uvv, uvf);

        let atlas = ImageBufferA::new_from_file("src/atlas.png")
            .or(ImageBufferA::new_from_file("../../src/atlas.png")
            .or(ImageBufferA::new_from_file("atlas.png")))
            .expect("couldn't load atlas from ./atlas.png");

        let renderer = KRenderer::new(&gl, uv_shader, atlas);

        Application {
            gl,
            window,
            renderer,
            event_aggregator: EventAggregator::new(default_xres, default_yres),

            game: Game::new(),

            xres: default_xres,
            yres: default_yres,
        }
    }

    pub fn handle_event(&mut self, event: &glutin::event::Event<()>) {
        match event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    self.window.resize(*physical_size);
                    self.xres = physical_size.width as f32;
                    self.yres = physical_size.height as f32;
                    unsafe {self.gl.viewport(0, 0, physical_size.width as i32, physical_size.height as i32)};
                },
                WindowEvent::Focused(false) => {
                    self.game.paused = true;
                },
                WindowEvent::Focused(true) => {
                    self.game.paused = false;
                }
                _ => {},
            _ => {},
            }
            _ => {},
        }

        if let Some(inputs) = self.event_aggregator.handle_event(event) {
            
            unsafe { self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT); } 

            let mut kc = KRCanvas::new();

            self.game.frame(&inputs, &mut kc);

            self.renderer.send(&self.gl, &kc.bytes());

            self.window.swap_buffers().unwrap();
        }
    }

    pub fn destroy(&mut self) {
        self.renderer.destroy(&self.gl);
    }
}

fn  make_shader(gl: &glow::Context, vert_paths: &[&str], frag_paths: &[&str]) -> glow::Program {
    unsafe {
        let program = gl.create_program().expect("Cannot create program");
        let shader_version = "#version 410";
        let shader_sources = [
            (glow::VERTEX_SHADER, load_file(vert_paths)),
            (glow::FRAGMENT_SHADER, load_file(frag_paths)),
        ];
        let mut shaders = Vec::with_capacity(shader_sources.len());
        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }
        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
        
        program
    }
}

unsafe fn opengl_boilerplate(xres: f32, yres: f32, event_loop: &glutin::event_loop::EventLoop<()>) -> (glow::Context, glutin::WindowedContext<glutin::PossiblyCurrent>) {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("round survival")
        .with_inner_size(glutin::dpi::PhysicalSize::new(xres, yres));
    let window = glutin::ContextBuilder::new()
        // .with_depth_buffer(0)
        // .with_srgb(true)
        // .with_stencil_buffer(0)
        .with_vsync(true)
        .build_windowed(window_builder, &event_loop)
        .unwrap()
        .make_current()
        .unwrap();


    let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
    gl.enable(DEPTH_TEST);
    // gl.enable(CULL_FACE);
    gl.blend_func(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
    gl.enable(BLEND);
    gl.debug_message_callback(|a, b, c, d, msg| {
        println!("{} {} {} {} msg: {}", a, b, c, d, msg);
    });

    (gl, window)
}