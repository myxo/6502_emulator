static CARRY_BIT: u8 = 0b0000_0001;
static ZERO_BIT: u8 = 0b0000_0010;
static INT_BIT: u8 = 0b0000_0100;
static DEC_BIT: u8 = 0b0000_1000;
static BRK_BIT: u8 = 0b0001_0000;
static _UNUSED: u8 = 0b0010_0000;
static OVERFLOW_BIT: u8 = 0b0100_0000;
static NEG_BIT: u8 = 0b0100_0000;

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Flags {
    register: u8,
}

impl Flags {
    pub fn new(register: u8) -> Self {
        Self { register }
    }

    // TODO: rewrite with macros?
    pub fn carry(&self) -> bool {
        self.register & CARRY_BIT != 0
    }

    #[allow(dead_code)]
    pub fn carry_byte(&self) -> u8 {
        self.register & CARRY_BIT
    }

    pub fn set_carry(&mut self, flag: bool) {
        if flag != self.carry() {
            self.register = self.register ^ CARRY_BIT;
        }
    }

    pub fn zero(&self) -> bool {
        self.register & ZERO_BIT != 0
    }

    #[allow(dead_code)]
    pub fn zero_byte(&self) -> u8 {
        self.register & ZERO_BIT
    }

    pub fn set_zero(&mut self, flag: bool) {
        if flag != self.zero() {
            self.register = self.register ^ ZERO_BIT;
        }
    }

    pub fn interrupt_disabled(&self) -> bool {
        self.register & INT_BIT != 0
    }

    #[allow(dead_code)]
    pub fn interrupt_disabled_byte(&self) -> u8 {
        self.register & INT_BIT
    }

    pub fn set_interrupt_disabled(&mut self, flag: bool) {
        if flag != self.interrupt_disabled() {
            self.register = self.register ^ INT_BIT;
        }
    }

    pub fn decimal_mode(&self) -> bool {
        self.register & DEC_BIT != 0
    }

    #[allow(dead_code)]
    pub fn decimal_mode_byte(&self) -> u8 {
        self.register & DEC_BIT
    }

    pub fn set_decimal_mode(&mut self, flag: bool) {
        if flag != self.decimal_mode() {
            self.register = self.register ^ DEC_BIT;
        }
    }

    #[allow(dead_code)]
    pub fn break_cmd(&self) -> bool {
        self.register & BRK_BIT != 0
    }

    #[allow(dead_code)]
    pub fn break_cmd_byte(&self) -> u8 {
        self.register & BRK_BIT
    }

    #[allow(dead_code)]
    pub fn set_break_cmd(&mut self, flag: bool) {
        if flag != self.break_cmd() {
            self.register = self.register ^ BRK_BIT;
        }
    }

    pub fn overflow(&self) -> bool {
        self.register & OVERFLOW_BIT != 0
    }

    #[allow(dead_code)]
    pub fn overflow_byte(&self) -> u8 {
        self.register & OVERFLOW_BIT
    }

    pub fn set_overflow(&mut self, flag: bool) {
        if flag != self.overflow() {
            self.register = self.register ^ OVERFLOW_BIT;
        }
    }

    pub fn negative(&self) -> bool {
        self.register & NEG_BIT != 0
    }

    #[allow(dead_code)]
    pub fn negative_byte(&self) -> u8 {
        self.register & NEG_BIT
    }

    pub fn set_negative(&mut self, flag: bool) {
        if flag != self.negative() {
            self.register = self.register ^ NEG_BIT;
        }
    }

    pub fn set_register(&mut self, reg: u8) {
        self.register = reg;
    }

    pub fn get_register(&mut self) -> u8 {
        self.register
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_flag() {
        let mut flags: Flags = Default::default();

        flags.set_zero(true);
        assert_eq!(flags.zero(), true);
    }

    #[test]
    fn get_bit() {
        let mut flags: Flags = Default::default();

        flags.set_zero(true);
        assert_eq!(flags.zero_byte(), ZERO_BIT);
    }
}
