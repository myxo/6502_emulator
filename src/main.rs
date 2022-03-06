pub mod bus;
mod cpu;
mod flags;
mod ops_lookup;
mod ram;

#[macro_use]
extern crate lazy_static;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use asm6502::assemble;
use bus::{Bus, Device};
use cpu::Cpu;
use ram::Ram;

fn main() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    let ram = Rc::new(RefCell::new(Ram::new(0xffff as u16)));

    // let asm = "LDA #1\nADC #1\nCMP #2".as_bytes();
    let asm = "LDA #1\nADC #1\nCMP #2".as_bytes();
    let mut buf = Vec::<u8>::new();
    if let Err(msg) = assemble(asm, &mut buf) {
        panic!("Failed to assemble: {}", msg);
    }

    (*ram).borrow_mut().set_memory(&buf, 0).unwrap();
    bus.connect_device(Rc::downgrade(&ram) as Weak<RefCell<dyn Device>>, 0, 0xffff);

    for _ in 1..10 {
        cpu.tick(&mut bus);
    }
}
