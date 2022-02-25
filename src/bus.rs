use std::cell::RefCell;
use std::rc::Weak;

pub trait Device {
    fn get_byte(&self, offset: u16) -> u8;
    fn set_byte(&mut self, byte: u8, offset: u16);
}

struct DeviceConnection {
    device: Weak<RefCell<dyn Device>>,
    from: u16,
    to: u16,
}

pub struct Bus {
    connections: Vec<DeviceConnection>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            connections: vec![],
        }
    }

    pub fn connect_device(&mut self, device: Weak<RefCell<dyn Device>>, from: u16, to: u16) {
        self.connections.push(DeviceConnection { device, from, to })
    }

    pub fn set_byte(&mut self, byte: u8, offset: u16) {
        for conn in &mut self.connections {
            if offset >= conn.from && offset < conn.to {
                if let Some(dev) = conn.device.upgrade() {
                    (*dev).borrow_mut().set_byte(byte, offset);
                }
            }
        }
    }

    pub fn get_byte(&self, offset: u16) -> u8 {
        for conn in &self.connections {
            if offset >= conn.from && offset < conn.to {
                if let Some(dev) = conn.device.upgrade() {
                    return (*dev).borrow().get_byte(offset);
                }
            }
        }
        todo!()
    }

    pub fn get_two_bytes(&self, offset: u16) -> u16 {
        ((self.get_byte(offset + 1) as u16) << 8) + self.get_byte(offset) as u16
    }
}
