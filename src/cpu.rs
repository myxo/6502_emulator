use crate::bus::Bus;

pub struct Cpu {
    pc: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu { pc: 0, }
    }

    pub fn tick(&mut self, bus: &Bus) {
        let op_code = bus.get_byte(self.pc);
        print!("{:#04x} ", op_code);
        self.pc += 1;
        // detect command size
        // read command
        // run command from lookup table
    }
}
