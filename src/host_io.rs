use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub enum Color {
    Black,
    White,
    Red,
    Cyan,
    Pink,
    Green,
    Blue,
    Yellow,
    Orange,
    Brown,
    LightRed,
    DarkGrey,
    MediumGrey,
    LightGreen,
    LightBlue,
    LightGray,
}

pub trait Monitor {
    fn clean(&mut self);
    fn set_symbol(&mut self, x: u16, y: u16, s: char, color: Color);
}

pub struct SdlHandler {
    pub canvas: sdl2::render::WindowCanvas,
    imgui_ctx: Option<imgui::Context>,
    imgui_sdl: Option<imgui_sdl2::ImguiSdl2>,
    renderer: imgui_opengl_renderer::Renderer,
    event_pump: sdl2::EventPump,
}

impl SdlHandler {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        let video = sdl_context.video().unwrap();
        {
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 0);
        }

        let window = video
            .window("rust-sdl2 demo", 800, 600)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();

        let mut imgui_ctx = imgui::Context::create();
        imgui_ctx.set_ini_filename(None);
        imgui_ctx.set_log_filename(None);

        imgui_ctx
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        let imgui_sdl = imgui_sdl2::ImguiSdl2::new(&mut imgui_ctx, &canvas.window());

        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui_ctx, |s| {
            video.gl_get_proc_address(s) as _
        });

        Self {
            canvas,
            imgui_ctx: Some(imgui_ctx),
            imgui_sdl: Some(imgui_sdl),
            renderer,
            event_pump,
        }
    }

    pub fn render_screen(&mut self) {
        self.imgui_sdl.as_mut().unwrap().prepare_frame(
            self.imgui_ctx.as_mut().unwrap().io_mut(),
            self.canvas.window(),
            &self.event_pump.mouse_state(),
        );

        let ui = self.imgui_ctx.as_mut().unwrap().frame();
        ui.show_demo_window(&mut true);

        self.canvas.present();

        self.imgui_sdl
            .as_mut()
            .unwrap()
            .prepare_render(&ui, &self.canvas.window());
        self.renderer.render(ui);
        self.canvas.window().gl_swap_window();
    }

    pub fn process_events(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            self.imgui_sdl
                .as_mut()
                .unwrap()
                .handle_event(&mut self.imgui_ctx.as_mut().unwrap(), &event);
            if self.imgui_sdl.as_mut().unwrap().ignore_event(&event) {
                continue;
            }
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return false;
                }
                _ => {}
            }
        }

        true
    }
}

impl Monitor for SdlHandler {
    fn clean(&mut self) {
        todo!();
    }

    fn set_symbol(&mut self, x: u16, y: u16, s: char, color: Color) {
        let color = sdl2::pixels::Color::RGB(x as u8, x as u8, x as u8);
        println!("color: {:?}", color);
        self.canvas.set_draw_color(color);
        self.canvas.clear();
    }
}
