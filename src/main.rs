mod asm_tests;
mod bus;
mod c64;
mod cpu;
mod debugger;
mod debug_server;
mod flags;
mod host_io;
mod ops_lookup;
mod ram;
mod vic;

use asm6502::assemble;
use c64::C64;
use debug_server::DebuggerServer;
use host_io::SdlHandler;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate sdl2;
extern crate serde;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

fn main() {
    configure_log();

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
    (*c64.ram).lock().unwrap().set_memory(&buf, 0).unwrap();

    let mut debug_server = DebuggerServer::new();

    'running: loop {
        if !sdl_handler.borrow_mut().process_events() {
            break 'running;
        }

        c64.tick();
        sdl_handler.borrow_mut().render_screen();

        if let Some(req) = debug_server.get_request() {
            let responce = debugger::process_command(&c64, req);

            debug_server.set_responce(responce.unwrap());
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn configure_log() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                //record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}
