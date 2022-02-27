use crate::bus::Bus;
use crate::ops_lookup::{AddressMode, Code, OPCODE_TABLE};

#[derive(Default, Clone, Copy)]
pub struct Registers {
    a: u8,
    x: u8,
    y: u8,
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Flags {
    carry: bool,
    zero: bool,
    interrupt_disabled: bool,
    decimal_mode: bool,
    break_command: bool,
    overdlow: bool,
    negative: bool,
}

#[derive(Default, Clone, Copy)]
pub struct Cpu {
    reg: Registers,
    flags: Flags,
    pc: u16,
    sp: u16,
    cycle_left: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            ..Default::default()
        }
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        if self.cycle_left > 0 {
            self.cycle_left -= 1;
            return;
        }
        let op_code = bus.get_byte(self.pc);
        println!("opcode: {:#04X} ", op_code);

        let op = OPCODE_TABLE[op_code as usize];
        if op.is_none() {
            panic!("Unknown opcode: {:#04x}\n", op_code);
        }
        let op = op.unwrap();

        let (address, cross_page): (u16, bool) = match op.mode {
            AddressMode::Immediate => (self.pc + 1, false),
            AddressMode::ZeroPage => (bus.get_byte(self.pc + 1) as u16, false),
            AddressMode::ZeroPageX => (bus.get_byte(self.pc + 1) as u16 + self.reg.x as u16, false),
            AddressMode::ZeroPageY => (bus.get_byte(self.pc + 1) as u16 + self.reg.y as u16, false),
            AddressMode::Absolute => (bus.get_two_bytes(self.pc + 1), false),
            AddressMode::AbsoluteX => {
                let by_arg = bus.get_two_bytes(self.pc + 1);
                let result = by_arg + self.reg.x as u16;
                let cross_memory_page = (by_arg & 0xff00) != (result & 0xff);
                (result, cross_memory_page)
            }
            AddressMode::AbsoluteY => {
                let by_arg = bus.get_two_bytes(self.pc + 1);
                let result = by_arg + self.reg.y as u16;
                let cross_memory_page = (by_arg & 0xff00) != (result & 0xff);
                (result, cross_memory_page)
            }
            AddressMode::Indirect => {
                let indirect = bus.get_two_bytes(self.pc + 1);
                (bus.get_two_bytes(indirect), false)
            }
            AddressMode::IndirectX => {
                let arg = bus.get_byte(self.pc + 1) as u16;
                (bus.get_two_bytes(arg + self.reg.x as u16), false)
            }
            AddressMode::IndirectY => {
                let by_arg = bus.get_byte(self.pc + 1) as u16;
                let result = bus.get_two_bytes(by_arg) + self.reg.y as u16;
                let cross_memory_page = (by_arg & 0xff00) != (result & 0xff);
                (result, cross_memory_page)
            }
            AddressMode::Implied => {
                (0, false) // Do not address memory in this mode
                           // TODO: make type safe check?
            }
        };
        println!("look at address: {:#04X} ", address);

        // TODO: do I need to delay values change until cycles complete?
        match op.code {
            Code::LDA => {
                self.reg.a = bus.get_byte(address);
                self.update_n_z_flags(self.reg.a);
            }
            Code::LDX => {
                self.reg.x = bus.get_byte(address);
                self.update_n_z_flags(self.reg.x);
            }
            Code::LDY => {
                self.reg.y = bus.get_byte(address);
                self.update_n_z_flags(self.reg.y);
            }
            Code::STA => {
                bus.set_byte(self.reg.a, address);
            }
            Code::STX => {
                bus.set_byte(self.reg.x, address);
            }
            Code::STY => {
                bus.set_byte(self.reg.y, address);
            }
            Code::TAX => {
                self.reg.x = self.reg.a;
                self.update_n_z_flags(self.reg.x);
            }
            Code::TXA => {
                self.reg.a = self.reg.x;
                self.update_n_z_flags(self.reg.a);
            }
            Code::TAY => {
                self.reg.y = self.reg.a;
                self.update_n_z_flags(self.reg.y);
            }
            Code::TYA => {
                self.reg.a = self.reg.y;
                self.update_n_z_flags(self.reg.a);
            }
            Code::INC => {
                let new_val = bus.get_byte(address) + 1;
                bus.set_byte(new_val, address);
                self.update_n_z_flags(new_val);
            }
            Code::INX => {
                let new_val = self.reg.x + 1;
                self.reg.x = new_val;
                self.update_n_z_flags(new_val);
            }
            Code::INY => {
                let new_val = self.reg.y + 1;
                self.reg.y = new_val;
                self.update_n_z_flags(new_val);
            }
            Code::DEC => {
                let new_val = bus.get_byte(address) - 1;
                bus.set_byte(new_val, address);
                self.update_n_z_flags(new_val);
            }
            Code::DEX => {
                let new_val = self.reg.x - 1;
                self.reg.x = new_val;
                self.update_n_z_flags(new_val);
            }
            Code::DEY => {
                let new_val = self.reg.y - 1;
                self.reg.y = new_val;
                self.update_n_z_flags(new_val);
            }
            Code::NOP => {}
        }
        self.pc += op.instruction_bytes as u16;
        self.cycle_left = op.cycles;
        if cross_page && op.page_boundary_cycle {
            self.cycle_left += 1;
        }
    }

    fn update_n_z_flags(&mut self, new_val: u8) {
        self.flags.zero = new_val == 0;
        self.flags.negative = new_val & 0b1000000 != 0;
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
    fn check_page_boundary() {
        let (mut cpu, mut bus, _ram) = fixture("LDA $10E0,X");
        cpu.reg.x = 0x56;

        cpu.tick(&mut bus);
        assert_eq!(cpu.cycle_left, 5); // add 1 cycle due to page boundary cross
    }

    #[test]
    fn lda_im() {
        let (mut cpu, mut bus, _ram) = fixture("LDA #42");

        let flag_old = cpu.flags;
        cpu.tick(&mut bus);
        assert_eq!(cpu.reg.a, 42);
        assert_eq!(cpu.flags, flag_old);
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

    #[test]
    fn lda_n_flag() {
        let (mut cpu, mut bus, _ram) = fixture("LDA #$ff");
        cpu.tick(&mut bus);

        assert!(cpu.flags.negative);
    }

    #[test]
    fn lda_z_flag() {
        let (mut cpu, mut bus, _ram) = fixture("LDA #$0");
        cpu.tick(&mut bus);

        assert!(cpu.flags.zero);
    }

    #[test]
    fn sta_abs() {
        let (mut cpu, mut bus, _ram) = fixture("STA $22ff");
        cpu.reg.a = 0x0a;
        cpu.tick(&mut bus);

        assert_eq!(bus.get_byte(0x22ff), 0x0a);
    }

    #[test]
    fn stx_abs() {
        let (mut cpu, mut bus, _ram) = fixture("STX $22ff");
        cpu.reg.x = 0x0a;
        cpu.tick(&mut bus);

        assert_eq!(bus.get_byte(0x22ff), 0x0a);
    }

    #[test]
    fn sty_abs() {
        let (mut cpu, mut bus, _ram) = fixture("STY $22ff");
        cpu.reg.y = 0x0a;
        cpu.tick(&mut bus);

        assert_eq!(bus.get_byte(0x22ff), 0x0a);
    }

    #[test]
    fn tax() {
        let (mut cpu, mut bus, _ram) = fixture("TAX\n");
        cpu.reg.a = 0x0a;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.x, 0x0a);
    }

    #[test]
    fn txa() {
        let (mut cpu, mut bus, _ram) = fixture("TXA\n");
        cpu.reg.x = 0x0a;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x0a);
    }

    #[test]
    fn tay() {
        let (mut cpu, mut bus, _ram) = fixture("TAY\n");
        cpu.reg.a = 0x0a;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.y, 0x0a);
    }

    #[test]
    fn tya() {
        let (mut cpu, mut bus, _ram) = fixture("TYA\n");
        cpu.reg.y = 0x0a;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x0a);
    }

    #[test]
    fn inc() {
        let (mut cpu, mut bus, _ram) = fixture("INC $0aff");
        bus.set_byte(10, 0x0aff);
        cpu.tick(&mut bus);

        assert_eq!(bus.get_byte(0x0aff), 11);
    }

    #[test]
    fn dec() {
        let (mut cpu, mut bus, _ram) = fixture("DEC $0aff");
        bus.set_byte(10, 0x0aff);
        cpu.tick(&mut bus);

        assert_eq!(bus.get_byte(0x0aff), 9);
    }

    #[test]
    fn inx() {
        let (mut cpu, mut bus, _ram) = fixture("INX\n");
        cpu.reg.x = 10;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.x, 11);
    }

    #[test]
    fn iny() {
        let (mut cpu, mut bus, _ram) = fixture("INY\n");
        cpu.reg.y = 10;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.y, 11);
    }

    #[test]
    fn dex() {
        let (mut cpu, mut bus, _ram) = fixture("DEX\n");
        cpu.reg.x = 10;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.x, 9);
    }

    #[test]
    fn dey() {
        let (mut cpu, mut bus, _ram) = fixture("DEY\n");
        cpu.reg.y = 10;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.y, 9);
    }
}
