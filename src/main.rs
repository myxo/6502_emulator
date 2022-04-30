mod bus;
mod cpu;
mod flags;
mod ops_lookup;
mod ram;
mod vic;
mod c64;

#[cfg(test)]
mod asm_tests;

#[macro_use]
extern crate lazy_static;

use asm6502::assemble;
use c64::C64;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

fn main() {
    let asm = r#"
    LDX #0
    loop:
    INX
    INX
    STX $b000
    NOP
    CLC
    BCC loop
        "#.as_bytes();
    let mut buf = Vec::<u8>::new();
    if let Err(msg) = assemble(asm, &mut buf) {
        panic!("Failed to assemble: {}", msg);
    }

    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut c64 = C64::new(&sdl_context);
    (*c64.ram).borrow_mut().set_memory(&buf, 0).unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        c64.tick();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
