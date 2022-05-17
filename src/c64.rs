use crate::bus::{Bus, Device};
use crate::cpu::Cpu;
use crate::host_io::Monitor;
use crate::ram::Ram;
use crate::vic::SimpleVic;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, Weak};

pub struct C64 {
    pub cpu: Cpu,
    pub bus: Bus,
    pub vic: Arc<Mutex<SimpleVic>>,
    pub ram: Arc<Mutex<Ram>>,
}

impl C64 {
    pub fn new(monitor: Rc<RefCell<dyn Monitor>>) -> Self {
        let mut c64 = Self {
            cpu: Cpu::new(),
            bus: Bus::new(),
            vic: Arc::new(Mutex::new(SimpleVic::new(monitor, 100))),
            ram: Arc::new(Mutex::new(Ram::new(0xffff + 1))),
        };

        c64.bus.connect_device(
            Arc::downgrade(&c64.ram) as Weak<Mutex<dyn Device>>,
            0,
            0xafff,
        );
        c64.bus.connect_device(
            Arc::downgrade(&c64.vic) as Weak<Mutex<dyn Device>>,
            0xb000,
            0xb100,
        );

        c64
    }

    pub fn tick(&mut self) {
        self.cpu.tick(&mut self.bus);
        self.bus.tick();
    }
}
