use crate::bus::Bus;
use crate::ops_lookup::{Code, OPCODE_TABLE};

#[derive(Default)]
pub struct Registers {
    a: u8,
    x: u8,
    y: u8,
}

#[derive(Default)]
pub struct Cpu {
    reg: Registers,
    pc: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            ..Default::default()
        }
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        let op_code = bus.get_byte(self.pc);
        print!("{:#04x} ", op_code);
        if let Some(op) = OPCODE_TABLE[op_code as usize] {
            // TODO: get rid of double lookup
            match op.code {
                Code::LDA => {
                    // TODO: address mode
                    self.reg.a = bus.get_byte(self.pc + 1);
                }
            }
            self.pc += op.instruction_bytes as u16;
        } else {
            print!("Unknown opcode: {:#04x}\n", op_code);
            self.pc += 1;
        }
        // detect command size
        // read command
        // run command from lookup table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ram::Ram;
    use crate::Device;
    use asm6502::assemble;
    use assert::*;
    use std::rc::{Rc, Weak};

    fn fixture(asm: &'static str) -> (Cpu, Bus, Rc<Ram>) {
        let max_memory = 0xff;
        let cpu = Cpu::new();
        let mut bus = Bus::new();
        let mut ram = Rc::new(Ram::new(max_memory as u16));

        let mut buf = Vec::<u8>::new();
        let asm = asm.as_bytes();
        assert_ok!(assemble(asm, &mut buf));

        Rc::get_mut(&mut ram).unwrap().set_memory(&buf, 0).unwrap();
        bus.connect_device(Rc::downgrade(&ram) as Weak<dyn Device>, 0, max_memory);
        (cpu, bus, ram)
    }

    #[test]
    fn lda_im() {
        let (mut cpu, mut bus, _ram) = fixture("LDA #42");

        cpu.tick(&mut bus);
        assert_eq!(cpu.reg.a, 42);
    }
}
