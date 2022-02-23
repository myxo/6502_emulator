use std::option::Option;

#[derive(Clone, Copy)]
#[allow(non_snake_case)]
pub enum Code {
    LDA,
}

#[derive(Clone, Copy)]
pub enum AddressMode {
    ImmediateAddress,
    ZeroPage,
}

#[derive(Clone, Copy)]
pub struct OpDescription {
    pub code: Code,
    pub mode: AddressMode,
    pub instruction_bytes: u8,
    pub cycles: u8,
    pub name: &'static str,
}

// For now it's just for read clarity. Not sure if we need to expose this types into OpDescription.
struct Byte(u8);
struct Cycle(u8);

impl OpDescription {
    #[rustfmt::skip]
    fn new(code: Code, mode: AddressMode, bytes: Byte, cycles: Cycle, name: &'static str) -> Option<OpDescription> {
        Some(OpDescription { code, mode, instruction_bytes: bytes.0, cycles: cycles.0, name})
    }
}

#[rustfmt::skip]
lazy_static! {
    pub static ref OPCODE_TABLE: [Option<OpDescription>; 256] = {
        let mut l = [None; 256];
        l[0xa5] = OpDescription::new(Code::LDA, AddressMode::ZeroPage, Byte(2), Cycle(3), "LDA");
        l[0xa9] = OpDescription::new(Code::LDA, AddressMode::ImmediateAddress, Byte(2), Cycle(2), "LDA");

        l
    };
}
