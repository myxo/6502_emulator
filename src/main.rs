mod bus;
mod c64;
mod cpu;
mod flags;
mod host_io;
mod ops_lookup;
mod ram;
mod vic;

#[cfg(test)]
mod asm_tests;

#[macro_use]
extern crate lazy_static;

use asm6502::assemble;
use c64::C64;
use host_io::SdlHandler;

extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate sdl2;

use std::cell::RefCell;
use std::rc::Rc;
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
"#
    .as_bytes();

    let mut buf = Vec::<u8>::new();
    assemble(asm, &mut buf).unwrap();

    let sdl_handler = Rc::new(RefCell::new(SdlHandler::new()));

    let mut c64 = C64::new(sdl_handler.clone());
    (*c64.ram).borrow_mut().set_memory(&buf, 0).unwrap();

    'running: loop {
        if !sdl_handler.borrow_mut().process_events() {
            break 'running;
        }

        c64.tick();
        sdl_handler.borrow_mut().render_screen();
        //deb_window.gl_swap_window();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
