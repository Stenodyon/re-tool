use std::convert::TryInto;
use std::fmt;

use crate::disassembler::{Architecture, Instruction, LogicalAddress};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnmappedAddress(pub u16);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GBInstruction {
    NOP,                         // 00
    LDd16(Reg16, u16),           // 01 11 21 31
    LDi16A(Reg16),               // 02 12
    INC16(Reg16),                // 03 13 23 33
    DEC16(Reg16),                // 0B 1B 2B 3B
    INC8(Reg8),                  // 04 0C 14 1C 24 2C 34 4C
    DEC8(Reg8),                  // 05 0D 15 1D 25 2D 35 3D
    LDd8(Reg8, u8),              // 06 0E 16 1E 26 2E 36 3E
    ADDHL(Reg16),                // 09 19 29 39
    JRr8(i8),                    // 18
    JRNZr8(i8),                  // 20
    LDHLincA,                    // 22
    JRZr8(i8),                   // 28
    LDAHLdec,                    // 2A
    LDHLdecA,                    // 32
    LD(Reg8, Reg8),              // 40 to 7F except 76
    SUB(Reg8),                   // 90 to 97
    AND(Reg8),                   // A0 to A7
    XOR(Reg8),                   // A8 to AF
    OR(Reg8),                    // B0 to B7
    CP(Reg8),                    // B8 to BF
    RETNZ,                       // C0
    JPNZa16(UnmappedAddress),    // C2
    JPa16(UnmappedAddress),      // C3
    RST(ResetVector),            // C7 CF D7 DF E7 EF F7 FF
    RET,                         // C9
    JPZa16(UnmappedAddress),     // CA
    Special(SpecialInstruction), // CB xx
    CALLa16(UnmappedAddress),    // CD
    PUSH(Reg16),                 // C5 D5 E5 F5
    POP(Reg16),                  // C1 D1 E1 F1
    LDHa8A(u8),                  // E0
    LDCA,                        // E2
    ANDd8(u8),                   // E6
    JPHL,                        // E9
    LDa16A(UnmappedAddress),     // EA
    LDHAa8(u8),                  // F0
    DI,                          // F3
    LDAa16(UnmappedAddress),     // FA
    EI,                          // FB
    CPd8(u8),                    // FE
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SpecialInstruction {
    RL(Reg8),   // 12
    SLA(Reg8),  // 23
    RES0(Reg8), // 87
}

impl SpecialInstruction {
    pub fn from_byte(byte: u8) -> Option<SpecialInstruction> {
        match byte {
            0x12 => Some(SpecialInstruction::RL(Reg8::D)),
            0x23 => Some(SpecialInstruction::SLA(Reg8::E)),
            0x87 => Some(SpecialInstruction::RES0(Reg8::A)),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SpecialInstruction::RL(_) => "RL",
            SpecialInstruction::SLA(_) => "SLA",
            SpecialInstruction::RES0(_) => "RES 0",
        }
    }

    pub fn first_argument(&self) -> Option<Argument> {
        match self {
            &SpecialInstruction::RL(reg)
            | &SpecialInstruction::SLA(reg)
            | &SpecialInstruction::RES0(reg) => Some(Argument::Reg8(reg)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    IndirectHL,
}

impl fmt::Display for Reg8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Reg8::A => "A",
                Reg8::B => "B",
                Reg8::C => "C",
                Reg8::D => "D",
                Reg8::E => "E",
                Reg8::H => "H",
                Reg8::L => "L",
                Reg8::IndirectHL => "(HL)",
            }
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

impl fmt::Display for Reg16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Reg16::AF => "AF",
                Reg16::BC => "BC",
                Reg16::DE => "DE",
                Reg16::HL => "HL",
                Reg16::SP => "SP",
            }
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ResetVector {
    H00,
    H08,
    H10,
    H18,
    H20,
    H28,
    H30,
    H38,
}

impl ResetVector {
    pub fn address(&self) -> UnmappedAddress {
        match self {
            Self::H00 => UnmappedAddress(0x00),
            Self::H08 => UnmappedAddress(0x08),
            Self::H10 => UnmappedAddress(0x10),
            Self::H18 => UnmappedAddress(0x18),
            Self::H20 => UnmappedAddress(0x20),
            Self::H28 => UnmappedAddress(0x28),
            Self::H30 => UnmappedAddress(0x30),
            Self::H38 => UnmappedAddress(0x38),
        }
    }
}

impl fmt::Display for ResetVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ResetVector::H00 => "00H",
                ResetVector::H08 => "08H",
                ResetVector::H10 => "10H",
                ResetVector::H18 => "18H",
                ResetVector::H20 => "20H",
                ResetVector::H28 => "28H",
                ResetVector::H30 => "30H",
                ResetVector::H38 => "38H",
            }
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Argument {
    Imm8(u8),
    Imm16(u16),
    Rel8(i8),
    Reg8(Reg8),
    Reg16(Reg16),
    IndirectReg16(Reg16),
    Address(UnmappedAddress),
    IndirectHLinc,
    IndirectHLdec,
    IndirectC,
    ResetVector(ResetVector),
}

impl GBInstruction {
    pub fn from_bytes(bytes: &[u8]) -> Option<GBInstruction> {
        match bytes[0] {
            0x00 => Some(GBInstruction::NOP),
            0x01 => {
                let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(GBInstruction::LDd16(Reg16::BC, value))
            }
            0x02 => Some(GBInstruction::LDi16A(Reg16::BC)),
            0x03 => Some(GBInstruction::INC16(Reg16::BC)),
            0x04 => Some(GBInstruction::INC8(Reg8::B)),
            0x05 => Some(GBInstruction::DEC8(Reg8::B)),
            0x06 => Some(GBInstruction::LDd8(Reg8::B, bytes[1])),
            0x0b => Some(GBInstruction::DEC16(Reg16::BC)),
            0x0c => Some(GBInstruction::INC8(Reg8::C)),
            0x0d => Some(GBInstruction::DEC8(Reg8::C)),
            0x0e => Some(GBInstruction::LDd8(Reg8::C, bytes[1])),
            0x11 => {
                let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(GBInstruction::LDd16(Reg16::DE, value))
            }
            0x12 => Some(GBInstruction::LDi16A(Reg16::DE)),
            0x13 => Some(GBInstruction::INC16(Reg16::DE)),
            0x14 => Some(GBInstruction::INC8(Reg8::D)),
            0x15 => Some(GBInstruction::DEC8(Reg8::D)),
            0x16 => Some(GBInstruction::LDd8(Reg8::D, bytes[1])),
            0x19 => Some(GBInstruction::ADDHL(Reg16::DE)),
            0x1b => Some(GBInstruction::DEC16(Reg16::DE)),
            0x1c => Some(GBInstruction::INC8(Reg8::E)),
            0x1d => Some(GBInstruction::DEC8(Reg8::E)),
            0x1e => Some(GBInstruction::LDd8(Reg8::E, bytes[1])),
            0x18 => Some(GBInstruction::JRr8(bytes[1] as i8)),
            0x20 => Some(GBInstruction::JRNZr8(bytes[1] as i8)),
            0x21 => {
                let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(GBInstruction::LDd16(Reg16::HL, value))
            }
            0x22 => Some(GBInstruction::LDHLincA),
            0x23 => Some(GBInstruction::INC16(Reg16::HL)),
            0x24 => Some(GBInstruction::INC8(Reg8::H)),
            0x25 => Some(GBInstruction::DEC8(Reg8::H)),
            0x26 => Some(GBInstruction::LDd8(Reg8::H, bytes[1])),
            0x28 => Some(GBInstruction::JRZr8(bytes[1] as i8)),
            0x2a => Some(GBInstruction::LDAHLdec),
            0x2b => Some(GBInstruction::DEC16(Reg16::HL)),
            0x2c => Some(GBInstruction::INC8(Reg8::L)),
            0x2d => Some(GBInstruction::DEC8(Reg8::L)),
            0x2e => Some(GBInstruction::LDd8(Reg8::L, bytes[1])),
            0x31 => {
                let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(GBInstruction::LDd16(Reg16::SP, value))
            }
            0x32 => Some(GBInstruction::LDHLdecA),
            0x33 => Some(GBInstruction::INC16(Reg16::SP)),
            0x34 => Some(GBInstruction::INC8(Reg8::IndirectHL)),
            0x35 => Some(GBInstruction::DEC8(Reg8::IndirectHL)),
            0x36 => Some(GBInstruction::LDd8(Reg8::IndirectHL, bytes[1])),
            0x3b => Some(GBInstruction::DEC16(Reg16::SP)),
            0x3c => Some(GBInstruction::INC8(Reg8::A)),
            0x3d => Some(GBInstruction::DEC8(Reg8::A)),
            0x3e => Some(GBInstruction::LDd8(Reg8::A, bytes[1])),
            0x40..=0x7f => {
                if bytes[0] == 0x76 {
                    // TODO
                    return None;
                }
                let registers = [
                    Reg8::B,
                    Reg8::C,
                    Reg8::D,
                    Reg8::E,
                    Reg8::H,
                    Reg8::L,
                    Reg8::IndirectHL,
                    Reg8::A,
                ];
                let dest_register_index = (bytes[0] & 0x38) >> 3;
                let source_register_index = bytes[0] & 0x07;
                let dest_register = registers[dest_register_index as usize];
                let source_register = registers[source_register_index as usize];
                Some(GBInstruction::LD(dest_register, source_register))
            }
            0x90 => Some(GBInstruction::SUB(Reg8::B)),
            0x91 => Some(GBInstruction::SUB(Reg8::C)),
            0x92 => Some(GBInstruction::SUB(Reg8::D)),
            0x93 => Some(GBInstruction::SUB(Reg8::E)),
            0x94 => Some(GBInstruction::SUB(Reg8::H)),
            0x95 => Some(GBInstruction::SUB(Reg8::L)),
            0x96 => Some(GBInstruction::SUB(Reg8::IndirectHL)),
            0x97 => Some(GBInstruction::SUB(Reg8::A)),
            0xa0 => Some(GBInstruction::AND(Reg8::B)),
            0xa1 => Some(GBInstruction::AND(Reg8::C)),
            0xa2 => Some(GBInstruction::AND(Reg8::D)),
            0xa3 => Some(GBInstruction::AND(Reg8::E)),
            0xa4 => Some(GBInstruction::AND(Reg8::H)),
            0xa5 => Some(GBInstruction::AND(Reg8::L)),
            0xa6 => Some(GBInstruction::AND(Reg8::IndirectHL)),
            0xa7 => Some(GBInstruction::AND(Reg8::A)),
            0xa8 => Some(GBInstruction::XOR(Reg8::B)),
            0xa9 => Some(GBInstruction::XOR(Reg8::C)),
            0xaa => Some(GBInstruction::XOR(Reg8::D)),
            0xab => Some(GBInstruction::XOR(Reg8::E)),
            0xac => Some(GBInstruction::XOR(Reg8::H)),
            0xad => Some(GBInstruction::XOR(Reg8::L)),
            0xae => Some(GBInstruction::XOR(Reg8::IndirectHL)),
            0xaf => Some(GBInstruction::XOR(Reg8::A)),
            0xb0 => Some(GBInstruction::OR(Reg8::B)),
            0xb1 => Some(GBInstruction::OR(Reg8::C)),
            0xb2 => Some(GBInstruction::OR(Reg8::D)),
            0xb3 => Some(GBInstruction::OR(Reg8::E)),
            0xb4 => Some(GBInstruction::OR(Reg8::H)),
            0xb5 => Some(GBInstruction::OR(Reg8::L)),
            0xb6 => Some(GBInstruction::OR(Reg8::IndirectHL)),
            0xb7 => Some(GBInstruction::OR(Reg8::A)),
            0xb8 => Some(GBInstruction::CP(Reg8::B)),
            0xb9 => Some(GBInstruction::CP(Reg8::C)),
            0xba => Some(GBInstruction::CP(Reg8::D)),
            0xbb => Some(GBInstruction::CP(Reg8::E)),
            0xbc => Some(GBInstruction::CP(Reg8::H)),
            0xbd => Some(GBInstruction::CP(Reg8::L)),
            0xbe => Some(GBInstruction::CP(Reg8::IndirectHL)),
            0xbf => Some(GBInstruction::CP(Reg8::A)),
            0xc0 => Some(GBInstruction::RETNZ),
            0xc1 => Some(GBInstruction::POP(Reg16::BC)),
            0xc2 => {
                let address = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(GBInstruction::JPNZa16(UnmappedAddress(address)))
            }
            0xc3 => {
                let address = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(GBInstruction::JPa16(UnmappedAddress(address)))
            }
            0xc5 => Some(GBInstruction::PUSH(Reg16::BC)),
            0xc7 => Some(GBInstruction::RST(ResetVector::H00)),
            0xc9 => Some(GBInstruction::RET),
            0xca => {
                let address = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(GBInstruction::JPZa16(UnmappedAddress(address)))
            }
            0xcb => SpecialInstruction::from_byte(bytes[1]).map(GBInstruction::Special),
            0xcd => {
                let address = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(GBInstruction::CALLa16(UnmappedAddress(address)))
            }
            0xcf => Some(GBInstruction::RST(ResetVector::H08)),
            0xd1 => Some(GBInstruction::POP(Reg16::DE)),
            0xd5 => Some(GBInstruction::PUSH(Reg16::DE)),
            0xd7 => Some(GBInstruction::RST(ResetVector::H10)),
            0xdf => Some(GBInstruction::RST(ResetVector::H18)),
            0xe0 => Some(GBInstruction::LDHa8A(bytes[1])),
            0xe1 => Some(GBInstruction::POP(Reg16::HL)),
            0xe7 => Some(GBInstruction::RST(ResetVector::H20)),
            0xe2 => Some(GBInstruction::LDCA),
            0xe5 => Some(GBInstruction::PUSH(Reg16::HL)),
            0xe6 => Some(GBInstruction::ANDd8(bytes[1])),
            0xe9 => Some(GBInstruction::JPHL),
            0xea => {
                let address = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(GBInstruction::LDa16A(UnmappedAddress(address)))
            }
            0xef => Some(GBInstruction::RST(ResetVector::H28)),
            0xf0 => Some(GBInstruction::LDHAa8(bytes[1])),
            0xf1 => Some(GBInstruction::POP(Reg16::AF)),
            0xf3 => Some(GBInstruction::DI),
            0xf5 => Some(GBInstruction::PUSH(Reg16::AF)),
            0xf7 => Some(GBInstruction::RST(ResetVector::H30)),
            0xfa => {
                let address = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(GBInstruction::LDAa16(UnmappedAddress(address)))
            }
            0xfb => Some(GBInstruction::EI),
            0xfe => Some(GBInstruction::CPd8(bytes[1])),
            0xff => Some(GBInstruction::RST(ResetVector::H38)),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            GBInstruction::NOP => "NOP",
            GBInstruction::LDd16(_, _)
            | GBInstruction::LDd8(_, _)
            | GBInstruction::LDi16A(_)
            | GBInstruction::LDHLincA
            | GBInstruction::LDAHLdec
            | GBInstruction::LDHLdecA
            | GBInstruction::LD(_, _)
            | GBInstruction::LDa16A(_)
            | GBInstruction::LDAa16(_)
            | GBInstruction::LDCA => "LD",
            GBInstruction::LDHAa8(_) | GBInstruction::LDHa8A(_) => "LDH",
            GBInstruction::DEC8(_) | GBInstruction::DEC16(_) => "DEC",
            GBInstruction::ADDHL(_) => "ADD",
            GBInstruction::JRr8(_) => "JR",
            GBInstruction::JRNZr8(_) => "JR NZ",
            GBInstruction::JRZr8(_) => "JR Z",
            GBInstruction::INC8(_) | GBInstruction::INC16(_) => "INC",
            GBInstruction::XOR(_) => "XOR",
            GBInstruction::OR(_) => "OR",
            GBInstruction::CP(_) => "CP",
            GBInstruction::RETNZ => "RET NZ",
            GBInstruction::JPNZa16(_) => "JP NZ",
            GBInstruction::JPa16(_) | GBInstruction::JPHL => "JP",
            GBInstruction::RET => "RET",
            GBInstruction::JPZa16(_) => "JP Z",
            GBInstruction::Special(special_instruction) => special_instruction.name(),
            GBInstruction::CALLa16(_) => "CALL",
            GBInstruction::PUSH(_) => "PUSH",
            GBInstruction::POP(_) => "POP",
            GBInstruction::SUB(_) => "SUB",
            GBInstruction::AND(_) | GBInstruction::ANDd8(_) => "AND",
            GBInstruction::DI => "DI",
            GBInstruction::EI => "EI",
            GBInstruction::CPd8(_) => "CP",
            GBInstruction::RST(_) => "RST",
        }
    }

    pub fn first_argument(&self) -> Option<Argument> {
        match self {
            GBInstruction::NOP | GBInstruction::DI | GBInstruction::EI => None,
            GBInstruction::LDd16(reg, _)
            | GBInstruction::INC16(reg)
            | GBInstruction::DEC16(reg)
            | GBInstruction::PUSH(reg)
            | GBInstruction::POP(reg) => Some(Argument::Reg16(*reg)),
            GBInstruction::ADDHL(_) => Some(Argument::Reg16(Reg16::HL)),
            GBInstruction::JRr8(value)
            | GBInstruction::JRNZr8(value)
            | GBInstruction::JRZr8(value) => Some(Argument::Rel8(*value)),
            GBInstruction::LDHLincA => Some(Argument::IndirectHLinc),
            GBInstruction::LDAHLdec => Some(Argument::Reg8(Reg8::A)),
            GBInstruction::LDHLdecA => Some(Argument::IndirectHLdec),
            GBInstruction::LDd8(reg, _)
            | GBInstruction::LD(reg, _)
            | GBInstruction::INC8(reg)
            | GBInstruction::DEC8(reg)
            | GBInstruction::SUB(reg)
            | GBInstruction::AND(reg)
            | GBInstruction::XOR(reg)
            | GBInstruction::OR(reg)
            | GBInstruction::CP(reg) => Some(Argument::Reg8(*reg)),
            GBInstruction::LDa16A(address)
            | GBInstruction::JPa16(address)
            | GBInstruction::JPZa16(address)
            | GBInstruction::JPNZa16(address)
            | GBInstruction::CALLa16(address) => Some(Argument::Address(*address)),
            GBInstruction::RET | GBInstruction::RETNZ => None,
            GBInstruction::Special(special_instruction) => special_instruction.first_argument(),
            GBInstruction::LDHa8A(value) => {
                Some(Argument::Address(UnmappedAddress(0xff00 | (*value as u16))))
            }
            GBInstruction::CPd8(value) | GBInstruction::ANDd8(value) => {
                Some(Argument::Imm8(*value))
            }
            GBInstruction::JPHL => Some(Argument::IndirectReg16(Reg16::HL)),
            GBInstruction::LDHAa8(_) | GBInstruction::LDAa16(_) => Some(Argument::Reg8(Reg8::A)),
            GBInstruction::LDCA => Some(Argument::IndirectC),
            GBInstruction::LDi16A(reg) => Some(Argument::IndirectReg16(*reg)),
            GBInstruction::RST(reset_vector) => Some(Argument::ResetVector(*reset_vector)),
        }
    }

    pub fn second_argument(&self) -> Option<Argument> {
        match self {
            GBInstruction::INC8(_)
            | GBInstruction::DEC8(_)
            | GBInstruction::INC16(_)
            | GBInstruction::DEC16(_)
            | GBInstruction::JRr8(_)
            | GBInstruction::JRZr8(_)
            | GBInstruction::JRNZr8(_)
            | GBInstruction::SUB(_)
            | GBInstruction::AND(_)
            | GBInstruction::XOR(_)
            | GBInstruction::OR(_)
            | GBInstruction::CP(_)
            | GBInstruction::ANDd8(_)
            | GBInstruction::CPd8(_)
            | GBInstruction::JPNZa16(_)
            | GBInstruction::JPa16(_)
            | GBInstruction::JPHL
            | GBInstruction::RET
            | GBInstruction::JPZa16(_)
            | GBInstruction::Special(_)
            | GBInstruction::CALLa16(_)
            | GBInstruction::PUSH(_)
            | GBInstruction::RETNZ
            | GBInstruction::POP(_)
            | GBInstruction::NOP
            | GBInstruction::DI
            | GBInstruction::EI
            | GBInstruction::RST(_) => None,
            GBInstruction::LDd16(_, value) => Some(Argument::Imm16(*value)),
            GBInstruction::LDd8(_, value) => Some(Argument::Imm8(*value)),
            GBInstruction::ADDHL(reg) => Some(Argument::Reg16(*reg)),
            GBInstruction::LDa16A(_)
            | GBInstruction::LDHa8A(_)
            | GBInstruction::LDHLincA
            | GBInstruction::LDHLdecA
            | GBInstruction::LDCA
            | GBInstruction::LDi16A(_) => Some(Argument::Reg8(Reg8::A)),
            GBInstruction::LD(_, reg) => Some(Argument::Reg8(*reg)),
            GBInstruction::LDHAa8(value) => {
                Some(Argument::Address(UnmappedAddress(0xff00 | (*value as u16))))
            }
            GBInstruction::LDAHLdec => Some(Argument::IndirectHLdec),
            GBInstruction::LDAa16(address) => Some(Argument::Address(*address)),
        }
    }

    /// Returns the jump address if this instruction contains one.
    pub fn jump_address(&self) -> Option<UnmappedAddress> {
        match self {
            &GBInstruction::JPa16(address)
            | &GBInstruction::CALLa16(address)
            | &GBInstruction::JPZa16(address)
            | &GBInstruction::JPNZa16(address) => Some(address),
            &GBInstruction::RST(reset_vector) => Some(reset_vector.address()),
            _ => None,
        }
    }
}

pub struct GameBoy;

impl Architecture for GameBoy {
    type Instruction = self::GBInstruction;

    fn disassemble(bytes: &[u8]) -> Option<GBInstruction> {
        GBInstruction::from_bytes(bytes)
    }

    fn resolve_address(
        address: LogicalAddress,
        location: usize,
        _state: &crate::disassembler::DisassemblerState,
    ) -> Option<usize> {
        match address {
            LogicalAddress::Absolute(address) => {
                if address < 0x4000 {
                    Some(address)
                } else if address < 0x8000 {
                    // TODO
                    None
                } else {
                    // TODO
                    None
                }
            }
            LogicalAddress::Relative(offset) => {
                let absolute_address = location.wrapping_add(offset as usize);
                Some(absolute_address)
            }
        }
    }
}

impl Instruction for GBInstruction {
    fn size(&self) -> usize {
        match self {
            GBInstruction::NOP => 1,
            GBInstruction::LDd16(_, _) => 3,
            GBInstruction::LDi16A(_) => 1,
            GBInstruction::INC8(_) => 1,
            GBInstruction::DEC8(_) => 1,
            GBInstruction::INC16(_) => 1,
            GBInstruction::DEC16(_) => 1,
            GBInstruction::LDd8(_, _) => 2,
            GBInstruction::JRZr8(_) => 2,
            GBInstruction::ADDHL(_) => 1,
            GBInstruction::JRr8(_) => 2,
            GBInstruction::JRNZr8(_) => 2,
            GBInstruction::LDHLincA => 1,
            GBInstruction::LDAHLdec => 1,
            GBInstruction::LDHLdecA => 1,
            GBInstruction::LD(_, _) => 1,
            GBInstruction::SUB(_) => 1,
            GBInstruction::AND(_) => 1,
            GBInstruction::XOR(_) => 1,
            GBInstruction::OR(_) => 1,
            GBInstruction::CP(_) => 1,
            GBInstruction::JPNZa16(_) => 3,
            GBInstruction::JPa16(_) => 3,
            GBInstruction::RET => 1,
            GBInstruction::JPZa16(_) => 3,
            GBInstruction::Special(_) => 2,
            GBInstruction::CALLa16(_) => 3,
            GBInstruction::PUSH(_) => 1,
            GBInstruction::RETNZ => 1,
            GBInstruction::POP(_) => 1,
            GBInstruction::LDHa8A(_) => 2,
            GBInstruction::LDCA => 1,
            GBInstruction::ANDd8(_) => 2,
            GBInstruction::JPHL => 1,
            GBInstruction::LDa16A(_) => 3,
            GBInstruction::DI => 1,
            GBInstruction::LDAa16(_) => 3,
            GBInstruction::LDHAa8(_) => 2,
            GBInstruction::EI => 1,
            GBInstruction::CPd8(_) => 2,
            GBInstruction::RST(_) => 1,
        }
    }

    /// Returns `true` if the execution can continue past this instruction. This would be false for
    /// unconditional jumps for example.
    fn falls_through(&self) -> bool {
        match self {
            GBInstruction::JPa16(_)
            | GBInstruction::JPHL
            | GBInstruction::RET
            | GBInstruction::JRr8(_) => false,
            _ => true,
        }
    }

    fn branch_address(&self) -> Option<crate::disassembler::LogicalAddress> {
        match self {
            &GBInstruction::JPa16(address)
            | &GBInstruction::CALLa16(address)
            | &GBInstruction::JPZa16(address)
            | &GBInstruction::JPNZa16(address) => {
                Some(LogicalAddress::Absolute(address.0 as usize))
            }
            &GBInstruction::JRr8(offset)
            | &GBInstruction::JRZr8(offset)
            | &GBInstruction::JRNZr8(offset) => Some(LogicalAddress::Relative(
                offset as isize + self.size() as isize,
            )),
            &GBInstruction::RST(reset_vector) => {
                Some(LogicalAddress::Absolute(reset_vector.address().0 as usize))
            }
            _ => None,
        }
    }
}
