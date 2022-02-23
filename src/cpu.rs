use crate::bus::Bus;
use crate::ops_lookup::{AddressMode, Code, OPCODE_TABLE};

#[derive(Default, Clone, Copy)]
pub struct Registers {
    a: u8,
    x: u8,
    y: u8,
}

#[derive(Default, Clone, Copy)]
pub struct Cpu {
    reg: Registers,
    pc: u16,
    sp: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            ..Default::default()
        }
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        let op_code = bus.get_byte(self.pc);
        println!("opcode: {:#04X} ", op_code);

        let op = OPCODE_TABLE[op_code as usize];
        if op.is_none() {
            print!("Unknown opcode: {:#04x}\n", op_code);
            self.pc += 1;
            return;
        }
        let op = op.unwrap();

        let address = match op.mode {
            AddressMode::ImmediateAddress => self.pc + 1,
            AddressMode::ZeroPage => {
                todo!();
            }
            AddressMode::ZeroPageX => {
                todo!();
            }
            AddressMode::ZeroPageY => {
                todo!();
            }
            AddressMode::Absolute => {
                ((bus.get_byte(self.pc + 2) as u16) << 8) + bus.get_byte(self.pc + 1) as u16
            }
            AddressMode::AbsoluteX => {
                todo!();
            }
            AddressMode::AbsoluteY => {
                todo!();
            }
            AddressMode::IndirectX => {
                todo!();
            }
            AddressMode::IndirectY => {
                todo!();
            }
        };
        println!("look at address: {:#04X} ", address);

        // TODO: get rid of double lookup?
        match op.code {
            Code::LDA => {
                self.reg.a = bus.get_byte(address);
            }
        }
        self.pc += op.instruction_bytes as u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ram::Ram;
    use crate::Device;
    use asm6502::assemble;
    use assert::*;
    use std::cell::RefCell;
    use std::rc::{Rc, Weak};

    fn fixture(asm: &'static str) -> (Cpu, Bus, Rc<RefCell<Ram>>) {
        let max_memory = 0xffff;
        let cpu = Cpu::new();
        let mut bus = Bus::new();
        let ram = Rc::new(RefCell::new(Ram::new(max_memory as u16)));

        let mut buf = Vec::<u8>::new();
        let asm = asm.as_bytes();
        assert_ok!(assemble(asm, &mut buf));

        (*ram).borrow_mut().set_memory(&buf, 0).unwrap();
        bus.connect_device(
            Rc::downgrade(&ram) as Weak<RefCell<dyn Device>>,
            0,
            max_memory,
        );
        (cpu, bus, ram)
    }

    #[test]
    fn lda_im() {
        let (mut cpu, mut bus, _ram) = fixture("LDA #42");

        cpu.tick(&mut bus);
        assert_eq!(cpu.reg.a, 42);
    }

    #[test]
    fn lda_abs() {
        let (mut cpu, mut bus, _ram) = fixture("LDA $4000");
        bus.set_byte(42, 0x4000);

        cpu.tick(&mut bus);
        assert_eq!(cpu.reg.a, 42);
    }
}
