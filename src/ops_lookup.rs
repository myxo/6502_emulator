use std::option::Option;

#[derive(Clone, Copy)]
#[allow(non_snake_case)]
pub enum Code {
    LDA,
    LDX,
    LDY,
    NOP,
}

#[derive(Clone, Copy)]
pub enum AddressMode {
    ImmediateAddress,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Implied,
}

#[derive(Clone, Copy)]
pub struct OpDescription {
    pub code: Code,
    pub mode: AddressMode,
    pub instruction_bytes: u8,
    pub cycles: u8,
    pub name: &'static str,
    pub page_boundary_cycle: bool,
}

// For now it's just for read clarity. Not sure if we need to expose this types into OpDescription.
struct Byte(u8);
struct Cycle(u8);

#[derive(PartialEq)]
enum PageBound {
    No,
    Yes,
}

impl OpDescription {
    #[rustfmt::skip]
    fn new(code: Code, mode: AddressMode, bytes: Byte, cycles: Cycle, name: &'static str, check_bound: PageBound) -> Option<OpDescription> {
        let check_page_bound : bool = check_bound == PageBound::Yes;
        Some(OpDescription { code, mode, instruction_bytes: bytes.0, cycles: cycles.0, name, page_boundary_cycle: check_page_bound})
    }
}

#[rustfmt::skip]
lazy_static! {
    pub static ref OPCODE_TABLE: [Option<OpDescription>; 256] = {
        let mut l = [None; 256];
        l[0xa9] = OpDescription::new(Code::LDA, AddressMode::ImmediateAddress, Byte(2), Cycle(2), "LDA", PageBound::No);
        l[0xa5] = OpDescription::new(Code::LDA, AddressMode::ZeroPage, Byte(2), Cycle(3), "LDA", PageBound::No);
        l[0xb5] = OpDescription::new(Code::LDA, AddressMode::ZeroPageX, Byte(2), Cycle(4), "LDA", PageBound::No);
        l[0xad] = OpDescription::new(Code::LDA, AddressMode::Absolute, Byte(3), Cycle(4), "LDA", PageBound::No);
        l[0xbd] = OpDescription::new(Code::LDA, AddressMode::AbsoluteX, Byte(3), Cycle(4), "LDA", PageBound::Yes);
        l[0xb9] = OpDescription::new(Code::LDA, AddressMode::AbsoluteY, Byte(3), Cycle(4), "LDA", PageBound::Yes);
        l[0xa1] = OpDescription::new(Code::LDA, AddressMode::IndirectX, Byte(2), Cycle(6), "LDA", PageBound::No);
        l[0xb1] = OpDescription::new(Code::LDA, AddressMode::IndirectY, Byte(2), Cycle(5), "LDA", PageBound::Yes);

        l[0xa2] = OpDescription::new(Code::LDX, AddressMode::ImmediateAddress, Byte(2), Cycle(2), "LDX", PageBound::No);
        l[0xa6] = OpDescription::new(Code::LDX, AddressMode::ZeroPage, Byte(2), Cycle(3), "LDX", PageBound::No);
        l[0xb6] = OpDescription::new(Code::LDX, AddressMode::ZeroPageY, Byte(2), Cycle(4), "LDX", PageBound::No);
        l[0xae] = OpDescription::new(Code::LDX, AddressMode::Absolute, Byte(3), Cycle(4), "LDX", PageBound::No);
        l[0xbe] = OpDescription::new(Code::LDX, AddressMode::AbsoluteY, Byte(3), Cycle(4), "LDX", PageBound::Yes);

        l[0xa0] = OpDescription::new(Code::LDY, AddressMode::ImmediateAddress, Byte(2), Cycle(2), "LDY", PageBound::No);
        l[0xa4] = OpDescription::new(Code::LDY, AddressMode::ZeroPage, Byte(2), Cycle(3), "LDY", PageBound::No);
        l[0xb4] = OpDescription::new(Code::LDY, AddressMode::ZeroPageX, Byte(2), Cycle(4), "LDY", PageBound::No);
        l[0xac] = OpDescription::new(Code::LDY, AddressMode::Absolute, Byte(3), Cycle(4), "LDY", PageBound::No);
        l[0xbc] = OpDescription::new(Code::LDY, AddressMode::AbsoluteX, Byte(3), Cycle(4), "LDY", PageBound::Yes);

        l[0xea] = OpDescription::new(Code::NOP, AddressMode::Implied, Byte(1), Cycle(2), "NOP", PageBound::No);

        l
    };
}
