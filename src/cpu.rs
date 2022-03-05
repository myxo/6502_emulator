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
        println!("tick");
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

        let (address, mut cross_page): (u16, bool) = match op.mode {
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
            AddressMode::Relative => {
                // this will get propper signed number
                let relative = bus.get_byte(self.pc + 1) as i8 as i16;
                let pc_with_offset = (self.pc + op.instruction_bytes as u16) as i16;

                // just assume that input instructions are correct and we won't overflow here...
                let new_pc = (pc_with_offset + relative) as u16;
                let cross_memory_page = self.pc & 0xff00 != new_pc & 0xff00;
                (new_pc, cross_memory_page)
            }
        };
        println!("look at address: {:#04X} ", address);

        self.pc += op.instruction_bytes as u16;
        let mut additional_cycles = 0;

        let mut branch_on = |cond: bool| {
            if cond {
                additional_cycles += 1;
                self.pc = address;
            } else {
                cross_page = false;
            }
        };

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
            Code::AND => {
                println!("A: {:#04X}, op: {:#04X}", self.reg.a, bus.get_byte(address));
                self.reg.a = self.reg.a & bus.get_byte(address);
                self.update_n_z_flags(self.reg.a);
            }
            Code::EOR => {
                self.reg.a = self.reg.a ^ bus.get_byte(address);
                self.update_n_z_flags(self.reg.a);
            }
            Code::ORA => {
                self.reg.a = self.reg.a | bus.get_byte(address);
                self.update_n_z_flags(self.reg.a);
            }
            Code::BIT => {
                let mem = bus.get_byte(address);
                // TODO: is this really accurate?
                self.flags.zero = self.reg.a & mem == 0;
                self.flags.negative = mem & 0b10000000 == 1;
                self.flags.overdlow = mem & 0b01000000 == 1;
            }
            Code::CLC => {
                self.flags.carry = false;
            }
            Code::CLD => {
                self.flags.decimal_mode = false;
            }
            Code::CLI => {
                self.flags.interrupt_disabled = false;
            }
            Code::CLV => {
                self.flags.overdlow = false;
            }
            Code::SEC => {
                self.flags.carry = true;
            }
            Code::SED => {
                self.flags.decimal_mode = true;
            }
            Code::SEI => {
                self.flags.interrupt_disabled = true;
            }
            Code::BCC => {
                branch_on(!self.flags.carry);
            }
            Code::BCS => {
                branch_on(self.flags.carry);
            }
            Code::BEQ => {
                branch_on(self.flags.zero);
            }
            Code::BMI => {
                branch_on(self.flags.negative);
            }
            Code::BNE => {
                branch_on(!self.flags.zero);
            }
            Code::BPL => {
                branch_on(!self.flags.negative);
            }
            Code::BVC => {
                branch_on(!self.flags.overdlow);
            }
            Code::BVS => {
                branch_on(self.flags.overdlow);
            }
            Code::NOP => {}
        }
        self.cycle_left = op.cycles - 1 + additional_cycles;
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

        println!("Assembled code: {:X?}", buf);

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
        // we remove 1 cycle (that already ticked) add 1 cycle due to page boundary cross
        assert_eq!(cpu.cycle_left, 4);
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

    #[test]
    fn and() {
        let (mut cpu, mut bus, _ram) = fixture("AND #06");
        cpu.reg.a = 0x05;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x04);
    }

    #[test]
    fn eor() {
        let (mut cpu, mut bus, _ram) = fixture("EOR #06");
        cpu.reg.a = 0x05;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x03);
    }

    #[test]
    fn ora() {
        let (mut cpu, mut bus, _ram) = fixture("ORA #06");
        cpu.reg.a = 0x05;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x07);
    }

    #[test]
    fn bit() {
        let (mut cpu, mut bus, _ram) = fixture("BIT $000a");
        bus.set_byte(5, 0x000a);
        cpu.reg.a = 0x05;
        cpu.tick(&mut bus);

        assert!(!cpu.flags.zero);
    }

    #[test]
    fn clc() {
        let (mut cpu, mut bus, _ram) = fixture("CLC\n");
        cpu.flags.carry = true;
        cpu.tick(&mut bus);

        assert!(!cpu.flags.carry);
    }

    #[test]
    fn cld() {
        let (mut cpu, mut bus, _ram) = fixture("CLD\n");
        cpu.flags.decimal_mode = true;
        cpu.tick(&mut bus);

        assert!(!cpu.flags.decimal_mode);
    }

    #[test]
    fn cli() {
        let (mut cpu, mut bus, _ram) = fixture("CLI\n");
        cpu.flags.interrupt_disabled = true;
        cpu.tick(&mut bus);

        assert!(!cpu.flags.interrupt_disabled);
    }

    #[test]
    fn clv() {
        let (mut cpu, mut bus, _ram) = fixture("CLV\n");
        cpu.flags.overdlow = true;
        cpu.tick(&mut bus);

        assert!(!cpu.flags.overdlow);
    }

    #[test]
    fn sec() {
        let (mut cpu, mut bus, _ram) = fixture("SEC\n");
        cpu.flags.carry = false;
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry);
    }

    #[test]
    fn sed() {
        let (mut cpu, mut bus, _ram) = fixture("SED\n");
        cpu.flags.decimal_mode = false;
        cpu.tick(&mut bus);

        assert!(cpu.flags.decimal_mode);
    }

    #[test]
    fn sei() {
        let (mut cpu, mut bus, _ram) = fixture("SEI\n");
        cpu.flags.interrupt_disabled = false;
        cpu.tick(&mut bus);

        assert!(cpu.flags.interrupt_disabled);
    }

    #[test]
    fn bcc_forward() {
        let (mut cpu, mut bus, _ram) = fixture("BCC 2");
        cpu.flags.carry = false;
        cpu.tick(&mut bus);

        assert_eq!(cpu.pc, 0x4);
    }

    #[test]
    fn bcc_forward_negative() {
        let (mut cpu, mut bus, _ram) = fixture("BCC 2");
        cpu.flags.carry = true;
        cpu.tick(&mut bus);

        assert_eq!(cpu.pc, 0x2);
    }

    #[test]
    fn bcc_backward() {
        let (mut cpu, mut bus, _ram) = fixture("\n");
        //   NOP
        // notequal:
        //   NOP
        //   BCC notequal

        cpu.flags.carry = false;
        // This is assembled code:
        bus.set_byte(0xea, 0);
        bus.set_byte(0xea, 1);
        bus.set_byte(0x90, 2);
        bus.set_byte(0xfd, 3);

        //cpu.pc = 0xf0;
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);

        cpu.tick(&mut bus);
        cpu.tick(&mut bus);

        cpu.tick(&mut bus);

        assert_eq!(cpu.pc, 0x1);
    }
}
