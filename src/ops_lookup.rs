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
    AND,
    EOR,
    ORA,
    BIT,
    CLC,
    CLD,
    CLI,
    CLV,
    SEC,
    SED,
    SEI,
    BCC,
    BCS,
    BEQ,
    BMI,
    BNE,
    BPL,
    BVC,
    BVS,
    ASL,
    LSR,
    ROL,
    ROR,
    TSX,
    TXS,
    PHA,
    PHP,
    PLA,
    PLP,
    JSR,
    RTS,
    CMP,
    CPX,
    CPY,
    JMP,
    ADC,
    SBC,
    RTI,
    BRK,
    NOP,
}

#[derive(Clone, Copy, PartialEq)]
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
    Relative,
    Accumulator,
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

        l[0x29] = declare_op!(Code::AND, AddressMode::Immediate, Byte(2), Cycle(2));
        l[0x25] = declare_op!(Code::AND, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0x35] = declare_op!(Code::AND, AddressMode::ZeroPageX, Byte(2), Cycle(4));
        l[0x2d] = declare_op!(Code::AND, AddressMode::Absolute, Byte(3), Cycle(4));
        l[0x3d] = declare_op!(Code::AND, AddressMode::AbsoluteX, Byte(3), Cycle(4), PageBound::Yes);
        l[0x39] = declare_op!(Code::AND, AddressMode::AbsoluteY, Byte(3), Cycle(4), PageBound::Yes);
        l[0x21] = declare_op!(Code::AND, AddressMode::IndirectX, Byte(2), Cycle(6));
        l[0x31] = declare_op!(Code::AND, AddressMode::IndirectY, Byte(2), Cycle(5), PageBound::Yes);

        l[0x49] = declare_op!(Code::EOR, AddressMode::Immediate, Byte(2), Cycle(2));
        l[0x45] = declare_op!(Code::EOR, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0x55] = declare_op!(Code::EOR, AddressMode::ZeroPageX, Byte(2), Cycle(4));
        l[0x4d] = declare_op!(Code::EOR, AddressMode::Absolute, Byte(3), Cycle(4));
        l[0x5d] = declare_op!(Code::EOR, AddressMode::AbsoluteX, Byte(3), Cycle(4), PageBound::Yes);
        l[0x59] = declare_op!(Code::EOR, AddressMode::AbsoluteY, Byte(3), Cycle(4), PageBound::Yes);
        l[0x41] = declare_op!(Code::EOR, AddressMode::IndirectX, Byte(2), Cycle(6));
        l[0x51] = declare_op!(Code::EOR, AddressMode::IndirectY, Byte(2), Cycle(5), PageBound::Yes);

        l[0x09] = declare_op!(Code::ORA, AddressMode::Immediate, Byte(2), Cycle(2));
        l[0x05] = declare_op!(Code::ORA, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0x15] = declare_op!(Code::ORA, AddressMode::ZeroPageX, Byte(2), Cycle(4));
        l[0x0d] = declare_op!(Code::ORA, AddressMode::Absolute, Byte(3), Cycle(4));
        l[0x1d] = declare_op!(Code::ORA, AddressMode::AbsoluteX, Byte(3), Cycle(4), PageBound::Yes);
        l[0x19] = declare_op!(Code::ORA, AddressMode::AbsoluteY, Byte(3), Cycle(4), PageBound::Yes);
        l[0x01] = declare_op!(Code::ORA, AddressMode::IndirectX, Byte(2), Cycle(6));
        l[0x11] = declare_op!(Code::ORA, AddressMode::IndirectY, Byte(2), Cycle(5), PageBound::Yes);
        
        l[0x24] = declare_op!(Code::BIT, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0x2c] = declare_op!(Code::BIT, AddressMode::Absolute, Byte(3), Cycle(4));

        l[0x18] = declare_op!(Code::CLC, AddressMode::Implied, Byte(1), Cycle(2));
        l[0xd8] = declare_op!(Code::CLD, AddressMode::Implied, Byte(1), Cycle(2));
        l[0x58] = declare_op!(Code::CLI, AddressMode::Implied, Byte(1), Cycle(2));
        l[0xb8] = declare_op!(Code::CLV, AddressMode::Implied, Byte(1), Cycle(2));
        l[0x38] = declare_op!(Code::SEC, AddressMode::Implied, Byte(1), Cycle(2));
        l[0xf8] = declare_op!(Code::SED, AddressMode::Implied, Byte(1), Cycle(2));
        l[0x78] = declare_op!(Code::SEI, AddressMode::Implied, Byte(1), Cycle(2));

        l[0x90] = declare_op!(Code::BCC, AddressMode::Relative, Byte(2), Cycle(2), PageBound::Yes);
        l[0xb0] = declare_op!(Code::BCS, AddressMode::Relative, Byte(2), Cycle(2), PageBound::Yes);
        l[0xf0] = declare_op!(Code::BEQ, AddressMode::Relative, Byte(2), Cycle(2), PageBound::Yes);
        l[0x30] = declare_op!(Code::BMI, AddressMode::Relative, Byte(2), Cycle(2), PageBound::Yes);
        l[0xd0] = declare_op!(Code::BNE, AddressMode::Relative, Byte(2), Cycle(2), PageBound::Yes);
        l[0x10] = declare_op!(Code::BPL, AddressMode::Relative, Byte(2), Cycle(2), PageBound::Yes);
        l[0x50] = declare_op!(Code::BVC, AddressMode::Relative, Byte(2), Cycle(2), PageBound::Yes);
        l[0x70] = declare_op!(Code::BVS, AddressMode::Relative, Byte(2), Cycle(2), PageBound::Yes);

        l[0x0a] = declare_op!(Code::ASL, AddressMode::Accumulator, Byte(1), Cycle(2));
        l[0x06] = declare_op!(Code::ASL, AddressMode::ZeroPage, Byte(2), Cycle(5));
        l[0x16] = declare_op!(Code::ASL, AddressMode::ZeroPageX, Byte(2), Cycle(6));
        l[0x0e] = declare_op!(Code::ASL, AddressMode::Absolute, Byte(3), Cycle(6));
        l[0x1e] = declare_op!(Code::ASL, AddressMode::AbsoluteX, Byte(3), Cycle(7));

        l[0x4a] = declare_op!(Code::LSR, AddressMode::Accumulator, Byte(1), Cycle(2));
        l[0x46] = declare_op!(Code::LSR, AddressMode::ZeroPage, Byte(2), Cycle(5));
        l[0x56] = declare_op!(Code::LSR, AddressMode::ZeroPageX, Byte(2), Cycle(6));
        l[0x4e] = declare_op!(Code::LSR, AddressMode::Absolute, Byte(3), Cycle(6));
        l[0x5e] = declare_op!(Code::LSR, AddressMode::AbsoluteX, Byte(3), Cycle(7));

        l[0x2a] = declare_op!(Code::ROL, AddressMode::Accumulator, Byte(1), Cycle(2));
        l[0x26] = declare_op!(Code::ROL, AddressMode::ZeroPage, Byte(2), Cycle(5));
        l[0x36] = declare_op!(Code::ROL, AddressMode::ZeroPageX, Byte(2), Cycle(6));
        l[0x2e] = declare_op!(Code::ROL, AddressMode::Absolute, Byte(3), Cycle(6));
        l[0x3e] = declare_op!(Code::ROL, AddressMode::AbsoluteX, Byte(3), Cycle(7));

        l[0x6a] = declare_op!(Code::ROR, AddressMode::Accumulator, Byte(1), Cycle(2));
        l[0x66] = declare_op!(Code::ROR, AddressMode::ZeroPage, Byte(2), Cycle(5));
        l[0x76] = declare_op!(Code::ROR, AddressMode::ZeroPageX, Byte(2), Cycle(6));
        l[0x6e] = declare_op!(Code::ROR, AddressMode::Absolute, Byte(3), Cycle(6));
        l[0x7e] = declare_op!(Code::ROR, AddressMode::AbsoluteX, Byte(3), Cycle(7));

        l[0xba] = declare_op!(Code::TSX, AddressMode::Implied, Byte(1), Cycle(2));
        l[0x9a] = declare_op!(Code::TXS, AddressMode::Implied, Byte(1), Cycle(2));
        l[0x48] = declare_op!(Code::PHA, AddressMode::Implied, Byte(1), Cycle(3));
        l[0x68] = declare_op!(Code::PLA, AddressMode::Implied, Byte(1), Cycle(4));
        l[0x08] = declare_op!(Code::PHP, AddressMode::Implied, Byte(1), Cycle(3));
        l[0x28] = declare_op!(Code::PLP, AddressMode::Implied, Byte(1), Cycle(4));

        l[0x20] = declare_op!(Code::JSR, AddressMode::Absolute, Byte(3), Cycle(6));
        l[0x60] = declare_op!(Code::RTS, AddressMode::Implied, Byte(1), Cycle(6));

        l[0xc9] = declare_op!(Code::CMP, AddressMode::Immediate, Byte(2), Cycle(2));
        l[0xc5] = declare_op!(Code::CMP, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0xd5] = declare_op!(Code::CMP, AddressMode::ZeroPageX, Byte(2), Cycle(4));
        l[0xcd] = declare_op!(Code::CMP, AddressMode::Absolute, Byte(3), Cycle(4));
        l[0xdd] = declare_op!(Code::CMP, AddressMode::AbsoluteX, Byte(3), Cycle(4), PageBound::Yes);
        l[0xd9] = declare_op!(Code::CMP, AddressMode::AbsoluteY, Byte(3), Cycle(4), PageBound::Yes);
        l[0xc1] = declare_op!(Code::CMP, AddressMode::IndirectX, Byte(2), Cycle(6));
        l[0xd1] = declare_op!(Code::CMP, AddressMode::IndirectY, Byte(2), Cycle(5), PageBound::Yes);

        l[0xe0] = declare_op!(Code::CPX, AddressMode::Immediate, Byte(2), Cycle(2));
        l[0xe4] = declare_op!(Code::CPX, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0xec] = declare_op!(Code::CPX, AddressMode::Absolute, Byte(3), Cycle(4));

        l[0xc0] = declare_op!(Code::CPY, AddressMode::Immediate, Byte(2), Cycle(2));
        l[0xc4] = declare_op!(Code::CPY, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0xcc] = declare_op!(Code::CPY, AddressMode::Absolute, Byte(3), Cycle(4));

        l[0x4c] = declare_op!(Code::JMP, AddressMode::Absolute, Byte(3), Cycle(3));
        l[0x6c] = declare_op!(Code::JMP, AddressMode::Indirect, Byte(3), Cycle(5));

        l[0x69] = declare_op!(Code::ADC, AddressMode::Immediate, Byte(2), Cycle(2));
        l[0x65] = declare_op!(Code::ADC, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0x75] = declare_op!(Code::ADC, AddressMode::ZeroPageX, Byte(2), Cycle(4));
        l[0x6d] = declare_op!(Code::ADC, AddressMode::Absolute, Byte(3), Cycle(4));
        l[0x7d] = declare_op!(Code::ADC, AddressMode::AbsoluteX, Byte(3), Cycle(4), PageBound::Yes);
        l[0x79] = declare_op!(Code::ADC, AddressMode::AbsoluteY, Byte(3), Cycle(4), PageBound::Yes);
        l[0x61] = declare_op!(Code::ADC, AddressMode::IndirectX, Byte(2), Cycle(6));
        l[0x71] = declare_op!(Code::ADC, AddressMode::IndirectY, Byte(2), Cycle(5), PageBound::Yes);

        l[0xe9] = declare_op!(Code::SBC, AddressMode::Immediate, Byte(2), Cycle(2));
        l[0xe5] = declare_op!(Code::SBC, AddressMode::ZeroPage, Byte(2), Cycle(3));
        l[0xf5] = declare_op!(Code::SBC, AddressMode::ZeroPageX, Byte(2), Cycle(4));
        l[0xed] = declare_op!(Code::SBC, AddressMode::Absolute, Byte(3), Cycle(4));
        l[0xfd] = declare_op!(Code::SBC, AddressMode::AbsoluteX, Byte(3), Cycle(4), PageBound::Yes);
        l[0xf9] = declare_op!(Code::SBC, AddressMode::AbsoluteY, Byte(3), Cycle(4), PageBound::Yes);
        l[0xe1] = declare_op!(Code::SBC, AddressMode::IndirectX, Byte(2), Cycle(6));
        l[0xf1] = declare_op!(Code::SBC, AddressMode::IndirectY, Byte(2), Cycle(5), PageBound::Yes);

        l[0x40] = declare_op!(Code::RTI, AddressMode::Implied, Byte(1), Cycle(6));
        l[0x00] = declare_op!(Code::BRK, AddressMode::Implied, Byte(1), Cycle(6));

        l[0xea] = declare_op!(Code::NOP, AddressMode::Implied, Byte(1), Cycle(2));

        l
    };
}
