pub mod bus;
mod cpu;
mod ops_lookup;
mod ram;

#[macro_use]
extern crate lazy_static;

use std::rc::{Rc, Weak};

use asm6502::assemble;
use bus::{Bus, Device};
use cpu::Cpu;
use ram::Ram;

fn main() {
    let mut cpu: Cpu = Default::default();
    let mut bus = Bus::new();
    let mut ram = Rc::new(Ram::new(0xffff as u16));

    let asm = "LDA #1\nADC #1\nCMP #2".as_bytes();
    let mut buf = Vec::<u8>::new();
    if let Err(msg) = assemble(asm, &mut buf) {
        panic!("Failed to assemble: {}", msg);
    }

    Rc::get_mut(&mut ram).unwrap().set_memory(&buf, 0);
    bus.connect_device(Rc::downgrade(&ram) as Weak<dyn Device>, 0, 0xffff);

    for _ in 1..10 {
        cpu.tick(&mut bus);
    }
}
