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

const MAX_LINES: u32 = 25;
const MAX_ROWS: u32 = 40;

pub struct SdlHandler {
    pub canvas: sdl2::render::WindowCanvas,
    event_pump: sdl2::EventPump,

    char_size_px: u8,
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

        let char_size_px = 16;
        let screen_x = MAX_ROWS * char_size_px as u32;
        let screen_y = MAX_LINES * char_size_px as u32;
        let window = video
            .window("C64 emulator", screen_x, screen_y)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();


        Self {
            canvas,
            event_pump,
            char_size_px,
        }
    }

    pub fn render_screen(&mut self) {
        self.canvas.present();

        self.canvas.window().gl_swap_window();
    }

    pub fn process_events(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
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
        trace!("color: {:?}", color);
        self.canvas.set_draw_color(color);
        self.canvas.clear();
    }
}
