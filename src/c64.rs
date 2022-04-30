use crate::cpu::Cpu;
use crate::bus::{Bus, Device};
use crate::vic::SimpleVic;
use crate::ram::Ram;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct C64{
    pub cpu: Cpu,
    pub bus: Bus,
    pub vic: Rc<RefCell<SimpleVic>>,
    pub ram: Rc<RefCell<Ram>>,
}

impl C64 {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let mut c64 = Self {
            cpu: Cpu::new(),
            bus: Bus::new(),
            vic: Rc::new(RefCell::new(SimpleVic::new(100, &sdl_context))),
            ram: Rc::new(RefCell::new(Ram::new(0xffff + 1))),
        };

        c64.bus.connect_device(Rc::downgrade(&c64.ram) as Weak<RefCell<dyn Device>>, 0, 0xafff);
        c64.bus.connect_device(Rc::downgrade(&c64.vic) as Weak<RefCell<dyn Device>>, 0xb000, 0xb100);


        c64
    }

    pub fn tick(&mut self) {
        self.cpu.tick(&mut self.bus);
        self.bus.tick();
    }
}
