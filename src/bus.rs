use std::cell::RefCell;
use std::rc::Weak;

pub trait Device {
    fn get_byte(&self, offset: u16) -> u8;
    fn set_byte(&mut self, byte: u8, offset: u16);
    fn tick(&mut self);

    // Debug purpose
    fn get_bytes_slice(&self, from: u16, to: u16) -> Vec<u8>;
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
            if offset >= conn.from && offset <= conn.to {
                if let Some(dev) = conn.device.upgrade() {
                    (*dev).borrow_mut().set_byte(byte, offset);
                }
            }
        }
    }

    pub fn tick(&mut self) {
        for conn in &mut self.connections {
            if let Some(dev) = conn.device.upgrade() {
                (*dev).borrow_mut().tick();
            }
        }
    }

    pub fn get_byte(&self, offset: u16) -> u8 {
        for conn in &self.connections {
            if offset >= conn.from && offset <= conn.to {
                if let Some(dev) = conn.device.upgrade() {
                    return (*dev).borrow().get_byte(offset);
                }
            }
        }
        panic!(
            "Try to get memory, but device is not assign. Offset: {:#04X}",
            offset
        );
    }

    pub fn get_bytes_slice(&self, from: u16, to: u16) -> Vec<u8> {
        assert!(from <= to);
        for conn in &self.connections {
            if from >= conn.from && to <= conn.to {
                if let Some(dev) = conn.device.upgrade() {
                    return (*dev).borrow().get_bytes_slice(from, to);
                }
            }
        }
        panic!(
            "Try to get memory, but device is not assign. From: {:#04X}, to: {:#04X}",
            from, to
        );
    }

    pub fn get_two_bytes(&self, offset: u16) -> u16 {
        ((self.get_byte(offset + 1) as u16) << 8) + self.get_byte(offset) as u16
    }
}
