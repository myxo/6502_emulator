use crate::bus::Device;
use crate::host_io::{Color, Monitor};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct SimpleVic {
    pub memory: Vec<u8>,
    pub monitor: Rc<RefCell<dyn Monitor>>,
}

#[derive(Debug)]
pub struct SetMemoryError {}

impl SimpleVic {
    pub fn new(monitor: Rc<RefCell<dyn Monitor>>, size: usize) -> Self {
        Self {
            memory: vec![0; size],
            monitor,
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
        let x = self.memory[0];
        let mut mon = self.monitor.borrow_mut();
        mon.set_symbol(x as u16, 0, 'a', Color::Red);
    }
}
