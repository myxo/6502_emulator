use crate::bus::Device;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct SimpleVic {
    memory: Vec<u8>,
    monitor: Monitor,
}

struct Monitor {
    video_subsystem: sdl2::VideoSubsystem,
    canvas: sdl2::render::WindowCanvas,
}

#[derive(Debug)]
pub struct SetMemoryError {}

impl SimpleVic {
    pub fn new(size: usize, sdl_context: &sdl2::Sdl) -> Self {

        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        Self {
            memory: vec![0; size],
            monitor: Monitor {video_subsystem, canvas},
        }
    }

    pub fn set_memory(&mut self, data: &[u8], offset: u16) -> Result<(), SetMemoryError> {
        let offset = offset as usize;
        if offset + data.len() > self.memory.len() {
            return Err(SetMemoryError {});
        }
        for i in 0..data.len() {
            self.memory[i + offset] = data[i]
        }
        Ok(())
    }
}

impl Device for SimpleVic {
    fn set_byte(&mut self, byte: u8, offset: u16) {
        let offset = offset - 0xb000; 
        self.memory[offset as usize] = byte;
    }

    fn get_byte(&self, offset: u16) -> u8 {
        let offset = offset - 0xb000; 
        self.memory[offset as usize]
    }

    fn get_bytes_slice(&self, from: u16, to: u16) -> Vec<u8> {
        self.memory[from as usize..to as usize].to_vec()
    }

    fn tick(&mut self) {
        let c = self.memory[0];
        let color = Color::RGB(c, c, c);
        println!("color: {:?}", color);
        self.monitor.canvas.set_draw_color(color);
        self.monitor.canvas.clear();
        self.monitor.canvas.present();
        self.monitor.canvas.present();
    }
}

