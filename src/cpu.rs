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
            AddressMode::ZeroPage => bus.get_byte(self.pc + 1) as u16,
            AddressMode::ZeroPageX => bus.get_byte(self.pc + 1) as u16 + self.reg.x as u16,
            AddressMode::ZeroPageY => bus.get_byte(self.pc + 1) as u16 + self.reg.y as u16,
            AddressMode::Absolute => bus.get_two_bytes(self.pc + 1),
            AddressMode::AbsoluteX => bus.get_two_bytes(self.pc + 1) + self.reg.x as u16,
            AddressMode::AbsoluteY => bus.get_two_bytes(self.pc + 1) + self.reg.y as u16,
            AddressMode::Indirect => {
                let indirect = bus.get_two_bytes(self.pc + 1);
                bus.get_two_bytes(indirect)
            }
            AddressMode::IndirectX => {
                let arg = bus.get_byte(self.pc + 1) as u16;
                bus.get_two_bytes(arg + self.reg.x as u16)
            }
            AddressMode::IndirectY => {
                let arg = bus.get_byte(self.pc + 1) as u16;
                bus.get_two_bytes(arg) + self.reg.y as u16
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

    #[test]
    fn lda_zero() {
        let (mut cpu, mut bus, _ram) = fixture("LDA $c1");
        bus.set_byte(42, 0x00c1);

        cpu.tick(&mut bus);
        assert_eq!(cpu.reg.a, 42);
    }

    #[test]
    fn lda_zero_x() {
        let (mut cpu, mut bus, _ram) = fixture("LDA $c0,X");
        cpu.reg.x = 1;
        bus.set_byte(42, 0x00c1);

        cpu.tick(&mut bus);
        assert_eq!(cpu.reg.a, 42);
    }

    #[test]
    fn lda_abs_x() {
        let (mut cpu, mut bus, _ram) = fixture("LDA $4000,X");
        cpu.reg.x = 1;
        bus.set_byte(42, 0x4001);

        cpu.tick(&mut bus);
        assert_eq!(cpu.reg.a, 42);
    }

    #[test]
    fn lda_abs_y() {
        let (mut cpu, mut bus, _ram) = fixture("LDA $4000,Y");
        cpu.reg.y = 1;
        bus.set_byte(42, 0x4001);

        cpu.tick(&mut bus);
        assert_eq!(cpu.reg.a, 42);
    }

    #[test]
    fn lda_indirect_x() {
        let (mut cpu, mut bus, _ram) = fixture("LDA ($f0,X)");
        cpu.reg.x = 1;
        bus.set_byte(5, 0x00f1);
        bus.set_byte(7, 0x00f2);
        bus.set_byte(0x0a, 0x0705);

        cpu.tick(&mut bus);
        assert_eq!(cpu.reg.a, 0x0a);
    }

    #[test]
    fn lda_indirect_y() {
        let (mut cpu, mut bus, _ram) = fixture("LDA ($f1),Y");
        cpu.reg.y = 1;
        bus.set_byte(3, 0x00f1);
        bus.set_byte(7, 0x00f2);
        bus.set_byte(0x0a, 0x0704);

        cpu.tick(&mut bus);
        assert_eq!(cpu.reg.a, 0x0a);
    }

    // TODO: cross page tests. lda_abs_x, lda_abs_y
}
