use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnmappedAddress(pub u16);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instruction {
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
    LD(Reg8, Reg8),              // 40 to 7F except 76
    AND(Reg8),                   // A0 to A7
    XOR(Reg8),                   // A8 to AF
    OR(Reg8),                    // B0 to B7
    CP(Reg8),                    // B8 to BF
    JPNZa16(UnmappedAddress),    // C2
    JPa16(UnmappedAddress),      // C3
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
    LDAa16(UnmappedAddress),     // FA
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
}

impl Instruction {
    pub fn from_bytes(bytes: &[u8]) -> Option<Instruction> {
        match bytes[0] {
            0x00 => Some(Instruction::NOP),
            0x01 => {
                let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::LDd16(Reg16::BC, value))
            }
            0x02 => Some(Instruction::LDi16A(Reg16::BC)),
            0x03 => Some(Instruction::INC16(Reg16::BC)),
            0x04 => Some(Instruction::INC8(Reg8::B)),
            0x05 => Some(Instruction::DEC8(Reg8::B)),
            0x06 => Some(Instruction::LDd8(Reg8::B, bytes[1])),
            0x0b => Some(Instruction::DEC16(Reg16::BC)),
            0x0c => Some(Instruction::INC8(Reg8::C)),
            0x0d => Some(Instruction::DEC8(Reg8::C)),
            0x0e => Some(Instruction::LDd8(Reg8::C, bytes[1])),
            0x11 => {
                let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::LDd16(Reg16::DE, value))
            }
            0x12 => Some(Instruction::LDi16A(Reg16::DE)),
            0x13 => Some(Instruction::INC16(Reg16::DE)),
            0x14 => Some(Instruction::INC8(Reg8::D)),
            0x15 => Some(Instruction::DEC8(Reg8::D)),
            0x16 => Some(Instruction::LDd8(Reg8::D, bytes[1])),
            0x19 => Some(Instruction::ADDHL(Reg16::DE)),
            0x1b => Some(Instruction::DEC16(Reg16::DE)),
            0x1c => Some(Instruction::INC8(Reg8::E)),
            0x1d => Some(Instruction::DEC8(Reg8::E)),
            0x1e => Some(Instruction::LDd8(Reg8::E, bytes[1])),
            0x18 => Some(Instruction::JRr8(bytes[1] as i8)),
            0x20 => Some(Instruction::JRNZr8(bytes[1] as i8)),
            0x21 => {
                let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::LDd16(Reg16::HL, value))
            }
            0x22 => Some(Instruction::LDHLincA),
            0x23 => Some(Instruction::INC16(Reg16::HL)),
            0x24 => Some(Instruction::INC8(Reg8::H)),
            0x25 => Some(Instruction::DEC8(Reg8::H)),
            0x26 => Some(Instruction::LDd8(Reg8::H, bytes[1])),
            0x28 => Some(Instruction::JRZr8(bytes[1] as i8)),
            0x2a => Some(Instruction::LDAHLdec),
            0x2b => Some(Instruction::DEC16(Reg16::HL)),
            0x2c => Some(Instruction::INC8(Reg8::L)),
            0x2d => Some(Instruction::DEC8(Reg8::L)),
            0x2e => Some(Instruction::LDd8(Reg8::L, bytes[1])),
            0x31 => {
                let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::LDd16(Reg16::SP, value))
            }
            0x33 => Some(Instruction::INC16(Reg16::SP)),
            0x34 => Some(Instruction::INC8(Reg8::IndirectHL)),
            0x35 => Some(Instruction::DEC8(Reg8::IndirectHL)),
            0x36 => Some(Instruction::LDd8(Reg8::IndirectHL, bytes[1])),
            0x3b => Some(Instruction::DEC16(Reg16::SP)),
            0x3c => Some(Instruction::INC8(Reg8::A)),
            0x3d => Some(Instruction::DEC8(Reg8::A)),
            0x3e => Some(Instruction::LDd8(Reg8::A, bytes[1])),
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
                Some(Instruction::LD(dest_register, source_register))
            }
            0xa0 => Some(Instruction::AND(Reg8::B)),
            0xa1 => Some(Instruction::AND(Reg8::C)),
            0xa2 => Some(Instruction::AND(Reg8::D)),
            0xa3 => Some(Instruction::AND(Reg8::E)),
            0xa4 => Some(Instruction::AND(Reg8::H)),
            0xa5 => Some(Instruction::AND(Reg8::L)),
            0xa6 => Some(Instruction::AND(Reg8::IndirectHL)),
            0xa7 => Some(Instruction::AND(Reg8::A)),
            0xa8 => Some(Instruction::XOR(Reg8::B)),
            0xa9 => Some(Instruction::XOR(Reg8::C)),
            0xaa => Some(Instruction::XOR(Reg8::D)),
            0xab => Some(Instruction::XOR(Reg8::E)),
            0xac => Some(Instruction::XOR(Reg8::H)),
            0xad => Some(Instruction::XOR(Reg8::L)),
            0xae => Some(Instruction::XOR(Reg8::IndirectHL)),
            0xaf => Some(Instruction::XOR(Reg8::A)),
            0xb0 => Some(Instruction::OR(Reg8::B)),
            0xb1 => Some(Instruction::OR(Reg8::C)),
            0xb2 => Some(Instruction::OR(Reg8::D)),
            0xb3 => Some(Instruction::OR(Reg8::E)),
            0xb4 => Some(Instruction::OR(Reg8::H)),
            0xb5 => Some(Instruction::OR(Reg8::L)),
            0xb6 => Some(Instruction::OR(Reg8::IndirectHL)),
            0xb7 => Some(Instruction::OR(Reg8::A)),
            0xb8 => Some(Instruction::CP(Reg8::B)),
            0xb9 => Some(Instruction::CP(Reg8::C)),
            0xba => Some(Instruction::CP(Reg8::D)),
            0xbb => Some(Instruction::CP(Reg8::E)),
            0xbc => Some(Instruction::CP(Reg8::H)),
            0xbd => Some(Instruction::CP(Reg8::L)),
            0xbe => Some(Instruction::CP(Reg8::IndirectHL)),
            0xbf => Some(Instruction::CP(Reg8::A)),
            0xc1 => Some(Instruction::POP(Reg16::BC)),
            0xc2 => {
                let address = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::JPNZa16(UnmappedAddress(address)))
            }
            0xc3 => {
                let address = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::JPa16(UnmappedAddress(address)))
            }
            0xc5 => Some(Instruction::PUSH(Reg16::BC)),
            0xc9 => Some(Instruction::RET),
            0xca => {
                let address = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::JPZa16(UnmappedAddress(address)))
            }
            0xcb => SpecialInstruction::from_byte(bytes[1]).map(Instruction::Special),
            0xcd => {
                let address = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::CALLa16(UnmappedAddress(address)))
            }
            0xd1 => Some(Instruction::POP(Reg16::DE)),
            0xd5 => Some(Instruction::PUSH(Reg16::DE)),
            0xe1 => Some(Instruction::POP(Reg16::HL)),
            0xe0 => Some(Instruction::LDHa8A(bytes[1])),
            0xe2 => Some(Instruction::LDCA),
            0xe5 => Some(Instruction::PUSH(Reg16::HL)),
            0xe6 => Some(Instruction::ANDd8(bytes[1])),
            0xe9 => Some(Instruction::JPHL),
            0xea => {
                let address = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::LDa16A(UnmappedAddress(address)))
            }
            0xf1 => Some(Instruction::POP(Reg16::AF)),
            0xf5 => Some(Instruction::PUSH(Reg16::AF)),
            0xfa => {
                let address = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::LDAa16(UnmappedAddress(address)))
            }
            0xf0 => Some(Instruction::LDHAa8(bytes[1])),
            0xfe => Some(Instruction::CPd8(bytes[1])),
            _ => None,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Instruction::NOP => 1,
            Instruction::LDd16(_, _) => 3,
            Instruction::LDi16A(_) => 1,
            Instruction::INC8(_) => 1,
            Instruction::DEC8(_) => 1,
            Instruction::INC16(_) => 1,
            Instruction::DEC16(_) => 1,
            Instruction::LDd8(_, _) => 2,
            Instruction::JRZr8(_) => 2,
            Instruction::ADDHL(_) => 1,
            Instruction::JRr8(_) => 2,
            Instruction::JRNZr8(_) => 2,
            Instruction::LDHLincA => 1,
            Instruction::LDAHLdec => 1,
            Instruction::LD(_, _) => 1,
            Instruction::AND(_) => 1,
            Instruction::XOR(_) => 1,
            Instruction::OR(_) => 1,
            Instruction::CP(_) => 1,
            Instruction::JPNZa16(_) => 3,
            Instruction::JPa16(_) => 3,
            Instruction::RET => 1,
            Instruction::JPZa16(_) => 3,
            Instruction::Special(_) => 2,
            Instruction::CALLa16(_) => 3,
            Instruction::PUSH(_) => 1,
            Instruction::POP(_) => 1,
            Instruction::LDHa8A(_) => 2,
            Instruction::LDCA => 1,
            Instruction::ANDd8(_) => 2,
            Instruction::JPHL => 1,
            Instruction::LDa16A(_) => 3,
            Instruction::LDAa16(_) => 3,
            Instruction::LDHAa8(_) => 2,
            Instruction::CPd8(_) => 2,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Instruction::NOP => "NOP",
            Instruction::LDd16(_, _)
            | Instruction::LDd8(_, _)
            | Instruction::LDi16A(_)
            | Instruction::LDHLincA
            | Instruction::LDAHLdec
            | Instruction::LD(_, _)
            | Instruction::LDa16A(_)
            | Instruction::LDAa16(_)
            | Instruction::LDCA => "LD",
            Instruction::LDHAa8(_) | Instruction::LDHa8A(_) => "LDH",
            Instruction::DEC8(_) | Instruction::DEC16(_) => "DEC",
            Instruction::ADDHL(_) => "ADD",
            Instruction::JRr8(_) => "JR",
            Instruction::JRNZr8(_) => "JR NZ",
            Instruction::JRZr8(_) => "JR Z",
            Instruction::INC8(_) | Instruction::INC16(_) => "INC",
            Instruction::XOR(_) => "XOR",
            Instruction::OR(_) => "OR",
            Instruction::CP(_) => "CP",
            Instruction::JPNZa16(_) => "JP NZ",
            Instruction::JPa16(_) | Instruction::JPHL => "JP",
            Instruction::RET => "RET",
            Instruction::JPZa16(_) => "JP Z",
            Instruction::Special(special_instruction) => special_instruction.name(),
            Instruction::CALLa16(_) => "CALL",
            Instruction::PUSH(_) => "PUSH",
            Instruction::POP(_) => "POP",
            Instruction::AND(_) | Instruction::ANDd8(_) => "AND",
            Instruction::CPd8(_) => "CP",
        }
    }

    pub fn first_argument(&self) -> Option<Argument> {
        match self {
            Instruction::NOP => None,
            Instruction::LDd16(reg, _)
            | Instruction::INC16(reg)
            | Instruction::DEC16(reg)
            | Instruction::PUSH(reg)
            | Instruction::POP(reg) => Some(Argument::Reg16(*reg)),
            Instruction::ADDHL(_) => Some(Argument::Reg16(Reg16::HL)),
            Instruction::JRr8(value) | Instruction::JRNZr8(value) | Instruction::JRZr8(value) => {
                Some(Argument::Rel8(*value))
            }
            Instruction::LDHLincA => Some(Argument::IndirectHLinc),
            Instruction::LDAHLdec => Some(Argument::Reg8(Reg8::A)),
            Instruction::LDd8(reg, _)
            | Instruction::LD(reg, _)
            | Instruction::INC8(reg)
            | Instruction::DEC8(reg)
            | Instruction::AND(reg)
            | Instruction::XOR(reg)
            | Instruction::OR(reg)
            | Instruction::CP(reg) => Some(Argument::Reg8(*reg)),
            Instruction::LDa16A(address)
            | Instruction::JPa16(address)
            | Instruction::JPZa16(address)
            | Instruction::JPNZa16(address)
            | Instruction::CALLa16(address) => Some(Argument::Address(*address)),
            Instruction::RET => None,
            Instruction::Special(special_instruction) => special_instruction.first_argument(),
            Instruction::LDHa8A(value) => {
                Some(Argument::Address(UnmappedAddress(0xff00 | (*value as u16))))
            }
            Instruction::CPd8(value) | Instruction::ANDd8(value) => Some(Argument::Imm8(*value)),
            Instruction::JPHL => Some(Argument::IndirectReg16(Reg16::HL)),
            Instruction::LDHAa8(_) | Instruction::LDAa16(_) => Some(Argument::Reg8(Reg8::A)),
            Instruction::LDCA => Some(Argument::IndirectC),
            Instruction::LDi16A(reg) => Some(Argument::IndirectReg16(*reg)),
        }
    }

    pub fn second_argument(&self) -> Option<Argument> {
        match self {
            Instruction::INC8(_)
            | Instruction::DEC8(_)
            | Instruction::INC16(_)
            | Instruction::DEC16(_)
            | Instruction::JRr8(_)
            | Instruction::JRZr8(_)
            | Instruction::JRNZr8(_)
            | Instruction::AND(_)
            | Instruction::XOR(_)
            | Instruction::OR(_)
            | Instruction::CP(_)
            | Instruction::ANDd8(_)
            | Instruction::CPd8(_)
            | Instruction::JPNZa16(_)
            | Instruction::JPa16(_)
            | Instruction::JPHL
            | Instruction::RET
            | Instruction::JPZa16(_)
            | Instruction::Special(_)
            | Instruction::CALLa16(_)
            | Instruction::PUSH(_)
            | Instruction::POP(_)
            | Instruction::NOP => None,
            Instruction::LDd16(_, value) => Some(Argument::Imm16(*value)),
            Instruction::LDd8(_, value) => Some(Argument::Imm8(*value)),
            Instruction::ADDHL(reg) => Some(Argument::Reg16(*reg)),
            Instruction::LDa16A(_)
            | Instruction::LDHa8A(_)
            | Instruction::LDHLincA
            | Instruction::LDCA
            | Instruction::LDi16A(_) => Some(Argument::Reg8(Reg8::A)),
            Instruction::LD(_, reg) => Some(Argument::Reg8(*reg)),
            Instruction::LDHAa8(value) => {
                Some(Argument::Address(UnmappedAddress(0xff00 | (*value as u16))))
            }
            Instruction::LDAHLdec => Some(Argument::IndirectHLdec),
            Instruction::LDAa16(address) => Some(Argument::Address(*address)),
        }
    }

    /// Returns `true` if the execution can continue past this instruction. This would be false for unconditional jumps for example.
    pub fn fall_through(&self) -> bool {
        match self {
            Instruction::JPa16(_) | Instruction::JPHL | Instruction::RET | Instruction::JRr8(_) => {
                false
            }
            _ => true,
        }
    }

    /// Returns the jump address if this instruction contains one.
    pub fn jump_address(&self) -> Option<UnmappedAddress> {
        match self {
            &Instruction::JPa16(address)
            | &Instruction::CALLa16(address)
            | &Instruction::JPZa16(address)
            | &Instruction::JPNZa16(address) => Some(address),
            _ => None,
        }
    }
}
