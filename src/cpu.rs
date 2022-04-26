use crate::bus::Bus;
use crate::flags::Flags;
use crate::ops_lookup::{AddressMode, Code, OPCODE_TABLE};

static START_PC: u16 = 0xfffc;
static INTERRUPT_PC: u16 = 0xfffe;

fn merge_bytes(hi: u8, lo: u8) -> u16 {
    ((hi as u16) << 8) + lo as u16
}

#[derive(Default, Clone, Copy)]
pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
}

#[derive(Clone, Copy)]
pub struct Cpu {
    pub reg: Registers,
    pub flags: Flags,
    pc: u16,
    sp: u8,
    cycle_left: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            reg: Default::default(),
            flags: Flags::new(0u8),
            pc: 0x0000,
            sp: 0xff,
            cycle_left: 0,
        }
    }

    pub fn reset(&mut self, bus: &Bus) {
        self.reg = Default::default();
        self.flags = Flags::new(0u8);
        self.sp = 0xff;
        self.cycle_left = 0;

        self.pc = bus.get_two_bytes(START_PC);
    }

    pub fn nmi(&mut self, bus: &mut Bus) {
        self.write_u16_to_stack(bus, self.pc);
        self.write_u8_to_stack(bus, self.flags.get_register());

        self.pc = bus.get_two_bytes(INTERRUPT_PC);
        // println!("nmi: this is new pc: {:#04X}", self.pc);

        // TODO: takes 8 (?) cycles
    }

    pub fn run_until_brk(&mut self, bus: &mut Bus) {
        while !self.flags.break_cmd() {
            self.tick(bus);
        }
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        if self.cycle_left > 0 {
            self.cycle_left -= 1;
            return;
        }
        println!("------");
        let op_code = bus.get_byte(self.pc);

        let op = OPCODE_TABLE[op_code as usize];
        if op.is_none() {
            panic!("Unknown opcode: {:#04x}\n", op_code);
        }
        let op = op.unwrap();

        println!("opcode: {} ({:#04X}) ", op.name, op_code);

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
                let lo = bus.get_byte(self.pc + 1);
                let hi = bus.get_byte(self.pc + 2);
                if lo == 0xff {
                    // CPU bug: we crossed page bound, however we read
                    // high byte not from next page, but from current.
                    let lo_res = bus.get_byte(merge_bytes(hi, 0xff));
                    let hi_res = bus.get_byte(merge_bytes(hi, 0x00));

                    (merge_bytes(hi_res, lo_res), false)
                } else {
                    (bus.get_two_bytes(merge_bytes(hi, lo)), false)
                }
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
            AddressMode::Accumulator => (0, false),
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
                self.reg.a &= bus.get_byte(address);
                self.update_n_z_flags(self.reg.a);
            }
            Code::EOR => {
                self.reg.a ^= bus.get_byte(address);
                self.update_n_z_flags(self.reg.a);
            }
            Code::ORA => {
                self.reg.a |= bus.get_byte(address);
                self.update_n_z_flags(self.reg.a);
            }
            Code::BIT => {
                let mem = bus.get_byte(address);
                // TODO: is this really accurate?
                self.flags.set_zero(self.reg.a & mem == 0);
                self.flags.set_negative(mem & 0b10000000 != 0);
                self.flags.set_overflow(mem & 0b01000000 != 0);
            }
            Code::CLC => {
                self.flags.set_carry(false);
            }
            Code::CLD => {
                self.flags.set_decimal_mode(false);
            }
            Code::CLI => {
                self.flags.set_interrupt_disabled(false);
            }
            Code::CLV => {
                self.flags.set_overflow(false);
            }
            Code::SEC => {
                self.flags.set_carry(true);
            }
            Code::SED => {
                self.flags.set_decimal_mode(true);
            }
            Code::SEI => {
                self.flags.set_interrupt_disabled(true);
            }
            Code::BCC => {
                branch_on(!self.flags.carry());
            }
            Code::BCS => {
                branch_on(self.flags.carry());
            }
            Code::BEQ => {
                branch_on(self.flags.zero());
            }
            Code::BMI => {
                branch_on(self.flags.negative());
            }
            Code::BNE => {
                branch_on(!self.flags.zero());
            }
            Code::BPL => {
                branch_on(!self.flags.negative());
            }
            Code::BVC => {
                branch_on(!self.flags.overflow());
            }
            Code::BVS => {
                branch_on(self.flags.overflow());
            }
            Code::ASL => {
                let mem = match op.mode {
                    AddressMode::Accumulator => self.reg.a,
                    _ => bus.get_byte(address),
                };

                self.flags.set_carry(mem & 0x80 == 0x80);
                let result = mem << 1;
                self.update_n_z_flags(result);

                match op.mode {
                    AddressMode::Accumulator => self.reg.a = result,
                    _ => bus.set_byte(result, address),
                };
            }
            Code::LSR => {
                let mem = match op.mode {
                    AddressMode::Accumulator => self.reg.a,
                    _ => bus.get_byte(address),
                };

                self.flags.set_carry(mem & 0x01 == 0x01);
                let result = mem >> 1;
                self.update_n_z_flags(result);

                match op.mode {
                    AddressMode::Accumulator => self.reg.a = result,
                    _ => bus.set_byte(result, address),
                };
            }
            Code::ROL => {
                let mem = match op.mode {
                    AddressMode::Accumulator => self.reg.a,
                    _ => bus.get_byte(address),
                };

                let result = if self.flags.carry() {
                    (mem << 1) | 0x01
                } else {
                    mem << 1
                };
                self.flags.set_carry(mem & 0x80 == 0x80);
                self.update_n_z_flags(result);

                match op.mode {
                    AddressMode::Accumulator => self.reg.a = result,
                    _ => bus.set_byte(result, address),
                };
            }
            Code::ROR => {
                let mem = match op.mode {
                    AddressMode::Accumulator => self.reg.a,
                    _ => bus.get_byte(address),
                };

                let result = if self.flags.carry() {
                    (mem >> 1) | 0x80
                } else {
                    mem >> 1
                };
                self.flags.set_carry(mem & 0x01 == 0x01);
                self.update_n_z_flags(result);

                match op.mode {
                    AddressMode::Accumulator => self.reg.a = result,
                    _ => bus.set_byte(result, address),
                };
            }
            Code::TSX => {
                self.reg.x = self.sp;
            }
            Code::TXS => {
                self.sp = self.reg.x;
            }
            Code::PHA => {
                self.write_u8_to_stack(bus, self.reg.a);
            }
            Code::PHP => {
                self.write_u8_to_stack(bus, self.flags.get_register());
            }
            Code::PLA => {
                let a = self.read_u8_from_stack(bus);
                self.reg.a = a;
            }
            Code::PLP => {
                let register = self.read_u8_from_stack(bus);
                self.flags.set_register(register);
            }
            Code::JSR => {
                self.write_u16_to_stack(bus, self.pc - 1);
                self.pc = address;
            }
            Code::RTS => {
                self.pc = self.read_u16_from_stack(bus);
                self.pc += 1;
            }
            Code::JMP => {
                self.pc = address;
            }
            Code::CMP => {
                let mem = bus.get_byte(address);
                self.flags.set_carry(self.reg.a >= mem);
                self.flags.set_zero(self.reg.a == mem);
                // TODO: can I do it without sub?
                self.flags.set_negative((self.reg.a - mem) & 0x80 != 0);
            }
            Code::CPX => {
                let mem = bus.get_byte(address);
                self.flags.set_carry(self.reg.x >= mem);
                self.flags.set_zero(self.reg.x == mem);
                // TODO: can I do it without sub?
                self.flags.set_negative((self.reg.x - mem) & 0x80 != 0);
            }
            Code::CPY => {
                let mem = bus.get_byte(address);
                self.flags.set_carry(self.reg.y >= mem);
                self.flags.set_zero(self.reg.y == mem);
                // TODO: can I do it without sub?
                self.flags.set_negative((self.reg.y - mem) & 0x80 != 0);
            }
            Code::ADC => {
                let mem = bus.get_byte(address);
                self.adc_impl(mem);
            }
            Code::SBC => {
                let mem = bus.get_byte(address);
                self.adc_impl(!mem);
            }
            Code::BRK => {
                if !self.flags.interrupt_disabled() {
                    self.nmi(bus);
                    self.flags.set_break_cmd(true);
                }
            }
            Code::RTI => {
                let register = self.read_u8_from_stack(bus);
                self.flags.set_register(register);
                self.pc = self.read_u16_from_stack(bus) + 1;
            }
            Code::NOP => {}
        }
        self.cycle_left = op.cycles - 1 + additional_cycles;
        if cross_page && op.page_boundary_cycle {
            self.cycle_left += 1;
        }
    }

    fn update_n_z_flags(&mut self, new_val: u8) {
        self.flags.set_zero(new_val == 0);
        self.flags.set_negative(new_val & 0b10000000 != 0);
    }

    fn adc_impl(&mut self, mem_value: u8) {
        let mut res = self.reg.a as u16 + mem_value as u16;
        if self.flags.carry() {
            res += 1;
        }
        self.flags.set_carry(res & 0xff00 != 0);
        let res = (res & 0x00ff) as u8;

        let a_bit = self.reg.a & 0x80 != 0;
        let m_bit = mem_value & 0x80 != 0;
        let res_bit = res & 0x80 != 0;

        let a_m_bits_same = !(a_bit ^ m_bit);
        let a_res_bits_diff = a_bit ^ res_bit;

        self.flags.set_overflow(a_m_bits_same && a_res_bits_diff);
        self.reg.a = res;
        self.update_n_z_flags(self.reg.a);
    }

    fn write_u8_to_stack(&mut self, bus: &mut Bus, data: u8) {
        // println!("write {:#04X} to stack at byte {:#04X}", data, 0x0100 + self.sp as u16);
        bus.set_byte(data, 0x0100 + self.sp as u16);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn read_u8_from_stack(&mut self, bus: &mut Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        // println!("get byte from stack {:#04X}", 0x0100 + self.sp as u16);
        bus.get_byte(0x0100 + self.sp as u16)
    }

    fn write_u16_to_stack(&mut self, bus: &mut Bus, data: u16) {
        // println!("write {:#04X} to stack at byte {:#04X}", ((data & 0xff00) >> 8) as u8, 0x0100 + self.sp as u16);
        bus.set_byte(((data & 0xff00) >> 8) as u8, 0x0100 + self.sp as u16);
        self.sp = self.sp.wrapping_sub(1);

        // println!("write {:#04X} to stack at byte {:#04X}", (data & 0x00ff) as u8, 0x0100 + self.sp as u16);
        bus.set_byte((data & 0x00ff) as u8, 0x0100 + self.sp as u16);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn read_u16_from_stack(&mut self, bus: &mut Bus) -> u16 {
        self.sp = self.sp.wrapping_add(1);
        let pc_lo: u16 = bus.get_byte(0x0100 + self.sp as u16) as u16;
        // println!("get byte {:#04X} from stack {:#04X}", pc_lo, 0x0100 + self.sp as u16);

        self.sp = self.sp.wrapping_add(1);
        let pc_hi: u16 = bus.get_byte(0x0100 + self.sp as u16) as u16;
        // println!("get byte {:#04X} from stack {:#04X}", pc_hi, 0x0100 + self.sp as u16);

        (pc_hi << 8) | pc_lo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Device;
    use crate::ram::Ram;
    use asm6502::assemble;
    use assert::*;
    use std::cell::RefCell;
    use std::rc::{Rc, Weak};

    fn fixture(asm: &'static str) -> (Cpu, Bus, Rc<RefCell<Ram>>) {
        let max_memory = 0xffff;
        let cpu = Cpu::new();
        let mut bus = Bus::new();
        let ram = Rc::new(RefCell::new(Ram::new(max_memory + 1)));

        let mut buf = Vec::<u8>::new();
        let asm = asm.as_bytes();
        assert_ok!(assemble(asm, &mut buf));

        println!("Assembled code: {:X?}", buf);

        (*ram).borrow_mut().set_memory(&buf, 0).unwrap();
        bus.connect_device(
            Rc::downgrade(&ram) as Weak<RefCell<dyn Device>>,
            0,
            max_memory as u16,
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

        assert!(cpu.flags.negative());
    }

    #[test]
    fn lda_z_flag() {
        let (mut cpu, mut bus, _ram) = fixture("LDA #$0");
        cpu.tick(&mut bus);

        assert!(cpu.flags.zero());
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
        let (mut cpu, mut bus, _ram) = fixture("TAX");
        cpu.reg.a = 0x0a;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.x, 0x0a);
    }

    #[test]
    fn txa() {
        let (mut cpu, mut bus, _ram) = fixture("TXA");
        cpu.reg.x = 0x0a;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x0a);
    }

    #[test]
    fn tay() {
        let (mut cpu, mut bus, _ram) = fixture("TAY");
        cpu.reg.a = 0x0a;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.y, 0x0a);
    }

    #[test]
    fn tya() {
        let (mut cpu, mut bus, _ram) = fixture("TYA");
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
        let (mut cpu, mut bus, _ram) = fixture("INX");
        cpu.reg.x = 10;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.x, 11);
    }

    #[test]
    fn iny() {
        let (mut cpu, mut bus, _ram) = fixture("INY");
        cpu.reg.y = 10;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.y, 11);
    }

    #[test]
    fn dex() {
        let (mut cpu, mut bus, _ram) = fixture("DEX");
        cpu.reg.x = 10;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.x, 9);
    }

    #[test]
    fn dey() {
        let (mut cpu, mut bus, _ram) = fixture("DEY");
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

        assert!(!cpu.flags.zero());
    }

    #[test]
    fn clc() {
        let (mut cpu, mut bus, _ram) = fixture("CLC");
        cpu.flags.set_carry(true);
        cpu.tick(&mut bus);

        assert!(!cpu.flags.carry());
    }

    #[test]
    fn cld() {
        let (mut cpu, mut bus, _ram) = fixture("CLD");
        cpu.flags.set_decimal_mode(true);
        cpu.tick(&mut bus);

        assert!(!cpu.flags.decimal_mode());
    }

    #[test]
    fn cli() {
        let (mut cpu, mut bus, _ram) = fixture("CLI");
        cpu.flags.set_interrupt_disabled(true);
        cpu.tick(&mut bus);

        assert!(!cpu.flags.interrupt_disabled());
    }

    #[test]
    fn clv() {
        let (mut cpu, mut bus, _ram) = fixture("CLV");
        cpu.flags.set_overflow(true);
        cpu.tick(&mut bus);

        assert!(!cpu.flags.overflow());
    }

    #[test]
    fn sec() {
        let (mut cpu, mut bus, _ram) = fixture("SEC");
        cpu.flags.set_carry(false);
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry());
    }

    #[test]
    fn sed() {
        let (mut cpu, mut bus, _ram) = fixture("SED");
        cpu.flags.set_decimal_mode(false);
        cpu.tick(&mut bus);

        assert!(cpu.flags.decimal_mode());
    }

    #[test]
    fn sei() {
        let (mut cpu, mut bus, _ram) = fixture("SEI");
        cpu.flags.set_interrupt_disabled(false);
        cpu.tick(&mut bus);

        assert!(cpu.flags.interrupt_disabled());
    }

    #[test]
    fn bcc_forward() {
        let (mut cpu, mut bus, _ram) = fixture("BCC 2");
        cpu.flags.set_carry(false);
        cpu.tick(&mut bus);

        assert_eq!(cpu.pc, 0x4);
    }

    #[test]
    fn bcc_forward_negative() {
        let (mut cpu, mut bus, _ram) = fixture("BCC 2");
        cpu.flags.set_carry(true);
        cpu.tick(&mut bus);

        assert_eq!(cpu.pc, 0x2);
    }

    #[test]
    fn bcc_backward() {
        let (mut cpu, mut bus, _ram) = fixture(
            r#"
           NOP
         notequal:
           NOP
           BCC notequal
        "#,
        );
        cpu.flags.set_carry(false);

        cpu.tick(&mut bus);
        cpu.tick(&mut bus);

        cpu.tick(&mut bus);
        cpu.tick(&mut bus);

        cpu.tick(&mut bus);

        assert_eq!(cpu.pc, 0x1);
    }

    #[test]
    fn asl_accumulator() {
        let (mut cpu, mut bus, _ram) = fixture("ASL A");
        cpu.reg.a = 0b01010101;
        cpu.tick(&mut bus);

        assert!(!cpu.flags.carry());
        assert_eq!(cpu.reg.a, 0b10101010);
    }

    #[test]
    fn asl_mem() {
        let (mut cpu, mut bus, _ram) = fixture("ASL $44");
        bus.set_byte(0b10101010, 0x44);
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry());
        assert_eq!(bus.get_byte(0x44), 0b01010100);
    }

    #[test]
    fn lsr_accumulator() {
        let (mut cpu, mut bus, _ram) = fixture("LSR A");
        cpu.reg.a = 0b01010101;
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry());
        assert_eq!(cpu.reg.a, 0b00101010);
    }

    #[test]
    fn lsr_mem() {
        let (mut cpu, mut bus, _ram) = fixture("LSR $44");
        bus.set_byte(0b10101010, 0x44);
        cpu.tick(&mut bus);

        assert!(!cpu.flags.carry());
        assert_eq!(bus.get_byte(0x44), 0b01010101);
    }

    #[test]
    fn rol_accumulator_with_carry() {
        let (mut cpu, mut bus, _ram) = fixture("ROL A");
        cpu.flags.set_carry(true);
        cpu.reg.a = 0b01010101;
        cpu.tick(&mut bus);

        assert!(!cpu.flags.carry());
        assert_eq!(cpu.reg.a, 0b10101011);
    }

    #[test]
    fn rol_accumulator_witout_carry() {
        let (mut cpu, mut bus, _ram) = fixture("ROL A");
        cpu.flags.set_carry(false);
        cpu.reg.a = 0b01010101;
        cpu.tick(&mut bus);

        assert!(!cpu.flags.carry());
        assert_eq!(cpu.reg.a, 0b10101010);
    }

    #[test]
    fn rol_mem() {
        let (mut cpu, mut bus, _ram) = fixture("ROL $44");
        bus.set_byte(0b10101010, 0x44);
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry());
        assert_eq!(bus.get_byte(0x44), 0b01010100);
    }

    #[test]
    fn ror_accumulator_with_carry() {
        let (mut cpu, mut bus, _ram) = fixture("ROR A");
        cpu.flags.set_carry(true);
        cpu.reg.a = 0b01010101;
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry());
        assert_eq!(cpu.reg.a, 0b10101010);
    }

    #[test]
    fn ror_accumulator_witout_carry() {
        let (mut cpu, mut bus, _ram) = fixture("ROR A");
        cpu.flags.set_carry(false);
        cpu.reg.a = 0b01010101;
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry());
        assert_eq!(cpu.reg.a, 0b00101010);
    }

    #[test]
    fn ror_mem() {
        let (mut cpu, mut bus, _ram) = fixture("ROR $44");
        bus.set_byte(0b10101010, 0x44);
        cpu.tick(&mut bus);

        assert!(!cpu.flags.carry());
        assert_eq!(bus.get_byte(0x44), 0b01010101);
    }

    #[test]
    fn stack_init() {
        let (cpu, _bus, _ram) = fixture("TSX");
        assert_eq!(cpu.sp, 0xff);
    }

    #[test]
    fn tsx() {
        let (mut cpu, mut bus, _ram) = fixture("TSX");
        cpu.sp = 0x15;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.x, 0x15);
    }

    #[test]
    fn txs() {
        let (mut cpu, mut bus, _ram) = fixture("TXS");
        cpu.reg.x = 0x15;
        cpu.tick(&mut bus);

        assert_eq!(cpu.sp, 0x15);
    }

    #[test]
    fn pha() {
        let (mut cpu, mut bus, _ram) = fixture("PHA");
        cpu.reg.a = 0xAB;
        cpu.tick(&mut bus);

        assert_eq!(bus.get_byte(0x01ff), 0xAB);
        assert_eq!(cpu.sp, 0xfe);
    }

    #[test]
    fn php() {
        let (mut cpu, mut bus, _ram) = fixture("PHP");
        cpu.flags.set_register(0xAC);
        cpu.tick(&mut bus);

        assert_eq!(bus.get_byte(0x01ff), 0xAC);
        assert_eq!(cpu.sp, 0xfe);
    }

    #[test]
    fn pla() {
        let (mut cpu, mut bus, _ram) = fixture("PLA");
        bus.set_byte(0xAB, 0x01ff);
        cpu.sp = 0xfe;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0xAB);
        assert_eq!(cpu.sp, 0xff);
    }

    #[test]
    fn plp() {
        let (mut cpu, mut bus, _ram) = fixture("PLP");
        bus.set_byte(0xAC, 0x01ff);
        cpu.sp = 0xfe;
        cpu.tick(&mut bus);

        assert_eq!(cpu.flags.get_register(), 0xAC);
        assert_eq!(cpu.sp, 0xff);
    }

    #[test]
    fn jsr_rts() {
        let (mut cpu, mut bus, _ram) = fixture(
            r#"
            JSR $0009
            LDX #$ab
            NOP
            NOP
            NOP
            NOP
            LDY #$bc
            RTS
        "#,
        );

        // When
        for _ in 1..100 {
            if cpu.reg.x == 0xab {
                break;
            }
            cpu.tick(&mut bus);
        }

        // Then
        assert_eq!(cpu.reg.x, 0xab);
        assert_eq!(cpu.reg.y, 0xbc);
    }

    #[test]
    fn cmp_greater() {
        let (mut cpu, mut bus, _ram) = fixture("CMP #$44");
        cpu.reg.a = 0x45;
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry());
        assert!(!cpu.flags.zero());
    }

    #[test]
    fn cmp_equal() {
        let (mut cpu, mut bus, _ram) = fixture("CMP #$44");
        cpu.reg.a = 0x44;
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry());
        assert!(cpu.flags.zero());
    }

    #[test]
    fn cmx_greater() {
        let (mut cpu, mut bus, _ram) = fixture("CPX #$44");
        cpu.reg.x = 0x45;
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry());
        assert!(!cpu.flags.zero());
    }

    #[test]
    fn cmy_equal() {
        let (mut cpu, mut bus, _ram) = fixture("CPY #$44");
        cpu.reg.y = 0x44;
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry());
        assert!(cpu.flags.zero());
    }

    #[test]
    fn cmy_greater() {
        let (mut cpu, mut bus, _ram) = fixture("CPY #$44");
        cpu.reg.y = 0x45;
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry());
        assert!(!cpu.flags.zero());
    }

    #[test]
    fn cmx_equal() {
        let (mut cpu, mut bus, _ram) = fixture("CPX #$44");
        cpu.reg.x = 0x44;
        cpu.tick(&mut bus);

        assert!(cpu.flags.carry());
        assert!(cpu.flags.zero());
    }

    #[test]
    fn jmp_abs() {
        let (mut cpu, mut bus, _ram) = fixture("JMP $5597");
        cpu.tick(&mut bus);

        assert_eq!(cpu.pc, 0x5597);
    }

    #[test]
    fn jmp_ind() {
        let (mut cpu, mut bus, _ram) = fixture("JMP ($5597)");
        bus.set_byte(0x00, 0x5597);
        bus.set_byte(0x55, 0x5598);
        cpu.tick(&mut bus);

        assert_eq!(cpu.pc, 0x5500);
    }

    #[test]
    fn jmp_page_boundary_bug() {
        let (mut cpu, mut bus, _ram) = fixture("JMP ($30ff)");
        bus.set_byte(0x40, 0x3000);
        bus.set_byte(0x80, 0x30ff);
        bus.set_byte(0x50, 0x3100);
        cpu.tick(&mut bus);

        assert_eq!(cpu.pc, 0x4080);
    }

    #[test]
    fn adc_zero_zero() {
        let (mut cpu, mut bus, _ram) = fixture("ADC #$0");
        cpu.reg.a = 0x0;
        cpu.flags.set_carry(false);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x0);
        assert!(!cpu.flags.carry());
        assert!(!cpu.flags.overflow());
    }

    #[test]
    fn adc_no_carry() {
        let (mut cpu, mut bus, _ram) = fixture("ADC #$5");
        cpu.reg.a = 0x15;
        cpu.flags.set_carry(false);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x1a);
        assert!(!cpu.flags.carry());
        assert!(!cpu.flags.overflow());
    }

    #[test]
    fn adc_with_carry() {
        let (mut cpu, mut bus, _ram) = fixture("ADC #$5");
        cpu.reg.a = 0x15;
        cpu.flags.set_carry(true);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x1b);
        assert!(!cpu.flags.carry());
        assert!(!cpu.flags.overflow());
    }

    #[test]
    fn adc_carry_bit_is_setted() {
        let (mut cpu, mut bus, _ram) = fixture("ADC #$f0");
        cpu.reg.a = 0x15;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x5);
        assert!(cpu.flags.carry());
        assert!(!cpu.flags.overflow());
    }

    #[test]
    fn adc_overflow_from_pos() {
        let (mut cpu, mut bus, _ram) = fixture("ADC #$7f");
        cpu.reg.a = 0x01;
        cpu.flags.set_carry(false);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x80);
        assert!(!cpu.flags.carry());
        assert!(cpu.flags.overflow());
        assert!(cpu.flags.negative());
    }

    #[test]
    fn adc_overflow_from_neg() {
        let (mut cpu, mut bus, _ram) = fixture("ADC #$f0");
        cpu.reg.a = 0x80;
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x70);
        assert!(cpu.flags.carry());
        assert!(cpu.flags.overflow());
    }

    #[test]
    fn adc_overflow_on_edge() {
        // 63 + 64 + 1 = 128
        let (mut cpu, mut bus, _ram) = fixture("ADC #$40");
        cpu.reg.a = 0x3f;
        cpu.flags.set_carry(true);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x80);
        assert!(!cpu.flags.carry());
        assert!(cpu.flags.overflow());
        assert!(cpu.flags.negative())
    }

    #[test]
    fn sbc_zero_from_zero() {
        let (mut cpu, mut bus, _ram) = fixture("SBC #$0");
        cpu.reg.a = 0x0;
        cpu.flags.set_carry(false);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0xff);
        assert!(!cpu.flags.carry());
    }

    #[test]
    fn sbc_zero_from_zero_with_carry() {
        let (mut cpu, mut bus, _ram) = fixture("SBC #$0");
        cpu.reg.a = 0x0;
        cpu.flags.set_carry(true);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x00);
        assert!(cpu.flags.carry());
        assert!(!cpu.flags.overflow());
        assert!(cpu.flags.zero());
    }

    #[test]
    fn sbc_minus_one_from_zero() {
        let (mut cpu, mut bus, _ram) = fixture("SBC #$ff");
        cpu.reg.a = 0x0;
        cpu.flags.set_carry(false);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x00);
        assert!(!cpu.flags.carry());
    }

    #[test]
    fn sbc_minus_one_from_zero_with_carry() {
        let (mut cpu, mut bus, _ram) = fixture("SBC #$ff");
        cpu.reg.a = 0x0;
        cpu.flags.set_carry(true);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x01);
        assert!(!cpu.flags.carry());
        assert!(!cpu.flags.overflow());
    }

    #[test]
    fn sbc_no_carry() {
        let (mut cpu, mut bus, _ram) = fixture("SBC #$5");
        cpu.reg.a = 0x15;
        cpu.flags.set_carry(false);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x0f);
        assert!(cpu.flags.carry());
        assert!(!cpu.flags.overflow());
    }

    #[test]
    fn sbc_carry() {
        let (mut cpu, mut bus, _ram) = fixture("SBC #$5");
        cpu.reg.a = 0x15;
        cpu.flags.set_carry(true);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x10);
        assert!(cpu.flags.carry());
        assert!(!cpu.flags.overflow());
    }

    #[test]
    fn sbc_overflow_from_neg_pos() {
        let (mut cpu, mut bus, _ram) = fixture("SBC #$01");
        cpu.reg.a = 0x80;
        cpu.flags.set_carry(true);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x7f);
        assert!(cpu.flags.carry());
        assert!(cpu.flags.overflow());
    }

    #[test]
    fn sbc_overflow_from_pos_neg() {
        let (mut cpu, mut bus, _ram) = fixture("SBC #$ff");
        cpu.reg.a = 0x7f;
        cpu.flags.set_carry(true);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x80);
        assert!(!cpu.flags.carry());
        assert!(cpu.flags.overflow());
        assert!(cpu.flags.negative());
    }

    #[test]
    fn sbc_overflow_on_edge() {
        // -64 - 64 - 1 = -129
        let (mut cpu, mut bus, _ram) = fixture("SBC #$40");
        cpu.reg.a = 0xc0;
        cpu.flags.set_carry(false);
        cpu.tick(&mut bus);

        assert_eq!(cpu.reg.a, 0x7f);
        assert!(cpu.flags.carry());
        assert!(cpu.flags.overflow());
        assert!(!cpu.flags.negative());
    }

    #[test]
    fn brk_rti() {
        let (mut cpu, mut bus, _ram) = fixture(
            r#"
            LDA #$05
            BRK
            LDX #$05
            NOP
            NOP
            LDY #$05
            RTI
        "#,
        );

        // Given
        bus.set_byte(0x07, 0xfffe);
        bus.set_byte(0x00, 0xffff);

        // When
        for _ in 1..100 {
            if cpu.reg.x == 0x05 {
                break;
            }
            cpu.tick(&mut bus);
        }

        // Then
        assert_eq!(cpu.reg.x, 0x05);
        assert_eq!(cpu.reg.y, 0x05);
    }
}
