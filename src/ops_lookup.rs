use std::option::Option;

#[derive(Clone, Copy)]
#[allow(non_snake_case)]
pub enum Code {
    LDA,
    LDX,
    LDY,
    STA,
    STX,
    STY,
    TAX,
    TXA,
    TAY,
    TYA,
    INC,
    INX,
    INY,
    DEC,
    DEX,
    DEY,
    NOP,
}

#[derive(Clone, Copy)]
pub enum AddressMode {
    Immediate,
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
    _No, // don't actually need this after macro?
    Yes,
}

macro_rules! declare_op {
    ($code: expr, $mode:expr, $bytes:expr, $cycles:expr, $check_bound:expr) => {{
        Some(OpDescription {
            code: $code,
            mode: $mode,
            instruction_bytes: $bytes.0,
            cycles: $cycles.0,
            name: stringify!(code),
            page_boundary_cycle: $check_bound == PageBound::Yes,
        })
    }};
    ($code: expr, $mode:expr, $bytes:expr, $cycles:expr) => {{
        Some(OpDescription {
            code: $code,
            mode: $mode,
            instruction_bytes: $bytes.0,
            cycles: $cycles.0,
            name: stringify!(code),
            page_boundary_cycle: false,
        })
    }};
}

#[rustfmt::skip]
lazy_static! {
    pub static ref OPCODE_TABLE: [Option<OpDescription>; 256] = {
        let mut l = [None; 256];
        l[0xa9] = declare_op!(Code::LDA, AddressMode::Immediate, Byte(2), Cycle(2));
        l[0xa5] = declare_op!(Code::LDA, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0xb5] = declare_op!(Code::LDA, AddressMode::ZeroPageX, Byte(2), Cycle(4));
        l[0xad] = declare_op!(Code::LDA, AddressMode::Absolute, Byte(3), Cycle(4));
        l[0xbd] = declare_op!(Code::LDA, AddressMode::AbsoluteX, Byte(3), Cycle(4), PageBound::Yes);
        l[0xb9] = declare_op!(Code::LDA, AddressMode::AbsoluteY, Byte(3), Cycle(4), PageBound::Yes);
        l[0xa1] = declare_op!(Code::LDA, AddressMode::IndirectX, Byte(2), Cycle(6));
        l[0xb1] = declare_op!(Code::LDA, AddressMode::IndirectY, Byte(2), Cycle(5), PageBound::Yes);

        l[0xa2] = declare_op!(Code::LDX, AddressMode::Immediate, Byte(2), Cycle(2));
        l[0xa6] = declare_op!(Code::LDX, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0xb6] = declare_op!(Code::LDX, AddressMode::ZeroPageY, Byte(2), Cycle(4));
        l[0xae] = declare_op!(Code::LDX, AddressMode::Absolute, Byte(3), Cycle(4));
        l[0xbe] = declare_op!(Code::LDX, AddressMode::AbsoluteY, Byte(3), Cycle(4), PageBound::Yes);

        l[0xa0] = declare_op!(Code::LDY, AddressMode::Immediate, Byte(2), Cycle(2));
        l[0xa4] = declare_op!(Code::LDY, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0xb4] = declare_op!(Code::LDY, AddressMode::ZeroPageX, Byte(2), Cycle(4));
        l[0xac] = declare_op!(Code::LDY, AddressMode::Absolute, Byte(3), Cycle(4));
        l[0xbc] = declare_op!(Code::LDY, AddressMode::AbsoluteX, Byte(3), Cycle(4), PageBound::Yes);

        l[0x85] = declare_op!(Code::STA, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0x95] = declare_op!(Code::STA, AddressMode::ZeroPageX, Byte(2), Cycle(4));
        l[0x8d] = declare_op!(Code::STA, AddressMode::Absolute, Byte(3), Cycle(4));
        l[0x9d] = declare_op!(Code::STA, AddressMode::AbsoluteX, Byte(3), Cycle(5));
        l[0x99] = declare_op!(Code::STA, AddressMode::AbsoluteY, Byte(3), Cycle(5));
        l[0x81] = declare_op!(Code::STA, AddressMode::IndirectX, Byte(2), Cycle(6));
        l[0x91] = declare_op!(Code::STA, AddressMode::IndirectY, Byte(2), Cycle(6));

        l[0x86] = declare_op!(Code::STX, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0x96] = declare_op!(Code::STX, AddressMode::ZeroPageY, Byte(2), Cycle(4));
        l[0x8e] = declare_op!(Code::STX, AddressMode::Absolute, Byte(3), Cycle(4));

        l[0x84] = declare_op!(Code::STY, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0x94] = declare_op!(Code::STY, AddressMode::ZeroPageX, Byte(2), Cycle(4));
        l[0x8c] = declare_op!(Code::STY, AddressMode::Absolute, Byte(3), Cycle(4));

        l[0xaa] = declare_op!(Code::TAX, AddressMode::Implied, Byte(1), Cycle(2));
        l[0x8a] = declare_op!(Code::TXA, AddressMode::Implied, Byte(1), Cycle(2));
        l[0xa8] = declare_op!(Code::TAY, AddressMode::Implied, Byte(1), Cycle(2));
        l[0x98] = declare_op!(Code::TYA, AddressMode::Implied, Byte(1), Cycle(2));

        l[0xe6] = declare_op!(Code::INC, AddressMode::ZeroPage, Byte(2), Cycle(5));
        l[0xf6] = declare_op!(Code::INC, AddressMode::ZeroPageX, Byte(2), Cycle(6));
        l[0xee] = declare_op!(Code::INC, AddressMode::Absolute, Byte(3), Cycle(6));
        l[0xfe] = declare_op!(Code::INC, AddressMode::AbsoluteX, Byte(3), Cycle(7));

        l[0xc6] = declare_op!(Code::DEC, AddressMode::ZeroPage, Byte(2), Cycle(5));
        l[0xd6] = declare_op!(Code::DEC, AddressMode::ZeroPageX, Byte(2), Cycle(6));
        l[0xce] = declare_op!(Code::DEC, AddressMode::Absolute, Byte(3), Cycle(6));
        l[0xde] = declare_op!(Code::DEC, AddressMode::AbsoluteX, Byte(3), Cycle(7));

        l[0xe8] = declare_op!(Code::INX, AddressMode::Implied, Byte(1), Cycle(2));
        l[0xc8] = declare_op!(Code::INY, AddressMode::Implied, Byte(1), Cycle(2));
        l[0xca] = declare_op!(Code::DEX, AddressMode::Implied, Byte(1), Cycle(2));
        l[0x88] = declare_op!(Code::DEY, AddressMode::Implied, Byte(1), Cycle(2));

        l[0xea] = declare_op!(Code::NOP, AddressMode::Implied, Byte(1), Cycle(2));

        l
    };
}
