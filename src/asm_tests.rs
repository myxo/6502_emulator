#[cfg(test)]
mod tests {
    use super::*;

    use crate::bus::{Bus, Device};
    use crate::cpu::Cpu;
    use crate::ram::Ram;
    use asm6502::assemble;
    use assert::*;
    use std::sync::{Arc, Mutex, Weak};

    fn fixture(asm: &'static str) -> (Cpu, Bus, Arc<Mutex<Ram>>) {
        let max_memory = 0xffff;
        let cpu = Cpu::new();
        let mut bus = Bus::new();
        let ram = Arc::new(Mutex::new(Ram::new(max_memory + 1)));

        let mut buf = Vec::<u8>::new();
        assert_ok!(assemble(asm, &mut buf));

        println!("Assembled code: {:X?}", buf);

        (*ram).lock().unwrap().set_memory(&buf, 0).unwrap();
        bus.connect_device(
            Arc::downgrade(&ram) as Weak<Mutex<dyn Device>>,
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
