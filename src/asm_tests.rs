#[cfg(test)]
mod tests {
    use super::*;

    use crate::bus::{Bus, Device};
    use crate::cpu::Cpu;
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
    fn multiply_test() {
        let (mut cpu, mut bus, _ram) = fixture(
            r#"
        LDX #5
        LDY #6
        LDA #0
        STY $0

    loop:
        CLC
        ADC $0
        DEX

        CPX #0
        BNE loop

    finish:
        TAX
        BRK
    "#,
        );

        cpu.run_until_brk(&mut bus);
        assert_eq!(cpu.reg.x, 5 * 6);
    }
}
