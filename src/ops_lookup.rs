use std::option::Option;

#[derive(Clone, Copy)]
#[allow(non_snake_case)]
pub enum Code {
    LDA
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

impl OpDescription {
    fn new(code: Code, mode: AddressMode, instruction_bytes: u8, cycles: u8, name: &'static str) -> Option<OpDescription> {
        Some(OpDescription { code, mode, instruction_bytes, cycles, name})
    }
}

lazy_static! {
    pub static ref OPCODE_TABLE: [Option<OpDescription>; 256] = {
        let mut l = [None; 256];
        l[0xa5] = OpDescription::new(Code::LDA, AddressMode::ZeroPage, 2, 3, "LDA");
        l[0xa9] = OpDescription::new(Code::LDA, AddressMode::ImmediateAddress, 2, 2, "LDA");

        l
    };
}
