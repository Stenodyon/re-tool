use std::convert::TryInto;
use std::fmt;

pub enum Instruction {
    NOP,                         // 00
    LDBCd16(u16),                // 01
    DECBC,                       // 0B
    LDDd8(u8),                   // 16
    ADDHLDE,                     // 19
    JRNZr8(i8),                  // 20
    LDHLd16(u16),                // 21
    LDHLincA,                    // 22
    INCHL,                       // 23
    LDSPd16(u16),                // 31
    LDAd8(u8),                   // 3E
    LDDHL,                       // 56
    LDDA,                        // 57
    LDEHL,                       // 5E
    LDEA,                        // 5F
    LDAB,                        // 78
    LDAD,                        // 7A
    XORA,                        // AF
    ORC,                         // B1
    JPa16(Address),              // C3
    RET,                         // C9
    Special(SpecialInstruction), // CB xx
    CALLa16(Address),            // CD
    PUSHDE,                      // D5
    POPHL,                       // E1
    LDHa8A(u8),                  // E0
    ANDd8(u8),                   // E6
    JPHL,                        // E9
    LDa16A(Address),             // EA
    LDHAa8(u8),                  // F0
    CPd8(u8),                    // FE
}

pub enum SpecialInstruction {
    RLD,   // 12
    SLAE,  // 23
    RES0A, // 87
}

impl SpecialInstruction {
    pub fn from_byte(byte: u8) -> Option<SpecialInstruction> {
        match byte {
            0x12 => Some(SpecialInstruction::RLD),
            0x23 => Some(SpecialInstruction::SLAE),
            0x87 => Some(SpecialInstruction::RES0A),
            _ => None,
        }
    }
}

impl fmt::Display for SpecialInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecialInstruction::RLD => write!(f, "RL D"),
            SpecialInstruction::SLAE => write!(f, "SLA E"),
            SpecialInstruction::RES0A => write!(f, "RES 0, A"),
        }
    }
}

impl Instruction {
    pub fn from_bytes(bytes: &[u8]) -> Option<Instruction> {
        match bytes[0] {
            0x00 => Some(Instruction::NOP),
            0x01 => {
                let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::LDBCd16(value))
            }
            0x0b => Some(Instruction::DECBC),
            0x16 => Some(Instruction::LDDd8(bytes[1])),
            0x19 => Some(Instruction::ADDHLDE),
            0x20 => Some(Instruction::JRNZr8(bytes[1] as i8)),
            0x21 => {
                let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::LDHLd16(value))
            }
            0x22 => Some(Instruction::LDHLincA),
            0x23 => Some(Instruction::INCHL),
            0x31 => {
                let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                Some(Instruction::LDSPd16(value))
            }
            0x3e => Some(Instruction::LDAd8(bytes[1])),
            0x56 => Some(Instruction::LDDHL),
            0x57 => Some(Instruction::LDDA),
            0x5e => Some(Instruction::LDEHL),
            0x5f => Some(Instruction::LDEA),
            0x78 => Some(Instruction::LDAB),
            0x7a => Some(Instruction::LDAD),
            0xaf => Some(Instruction::XORA),
            0xb1 => Some(Instruction::ORC),
            0xc3 => {
                let offset = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                let address = Address::from_physical_address(offset);
                Some(Instruction::JPa16(address))
            }
            0xc9 => Some((Instruction::RET)),
            0xcb => SpecialInstruction::from_byte(bytes[1]).map(Instruction::Special),
            0xcd => {
                let offset = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                let address = Address::from_physical_address(offset);
                Some(Instruction::CALLa16(address))
            }
            0xd5 => Some(Instruction::PUSHDE),
            0xe1 => Some(Instruction::POPHL),
            0xe0 => Some(Instruction::LDHa8A(bytes[1])),
            0xe6 => Some(Instruction::ANDd8(bytes[1])),
            0xe9 => Some(Instruction::JPHL),
            0xea => {
                let offset = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                let address = Address::from_physical_address(offset);
                Some(Instruction::LDa16A(address))
            }
            0xf0 => Some(Instruction::LDHAa8(bytes[1])),
            0xfe => Some(Instruction::CPd8(bytes[1])),
            _ => None,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Instruction::NOP => 1,
            Instruction::LDBCd16(_) => 3,
            Instruction::DECBC => 1,
            Instruction::LDDd8(_) => 2,
            Instruction::ADDHLDE => 1,
            Instruction::JRNZr8(_) => 2,
            Instruction::LDHLd16(_) => 3,
            Instruction::LDHLincA => 1,
            Instruction::INCHL => 1,
            Instruction::LDSPd16(_) => 3,
            Instruction::LDAd8(_) => 2,
            Instruction::LDDHL => 1,
            Instruction::LDDA => 1,
            Instruction::LDEHL => 1,
            Instruction::LDEA => 1,
            Instruction::LDAB => 1,
            Instruction::LDAD => 1,
            Instruction::XORA => 1,
            Instruction::ORC => 1,
            Instruction::JPa16(_) => 3,
            Instruction::RET => 1,
            Instruction::Special(_) => 2,
            Instruction::CALLa16(_) => 3,
            Instruction::PUSHDE => 1,
            Instruction::POPHL => 1,
            Instruction::LDHa8A(_) => 2,
            Instruction::ANDd8(_) => 2,
            Instruction::JPHL => 1,
            Instruction::LDa16A(_) => 3,
            Instruction::LDHAa8(_) => 2,
            Instruction::CPd8(_) => 2,
        }
    }

    /// Returns `true` if the execution can continue past this instruction. This would be false for unconditional jumps for example.
    pub fn fall_through(&self) -> bool {
        match self {
            Instruction::JPa16(_) | Instruction::JPHL | Instruction::RET => false,
            _ => true,
        }
    }

    /// Returns the jump address if this instruction contains one.
    pub fn jump_address(&self) -> Option<Address> {
        match self {
            &Instruction::JPa16(address) => Some(address),
            &Instruction::CALLa16(address) => Some(address),
            _ => None,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::NOP => write!(f, "NOP"),
            Instruction::LDBCd16(value) => write!(f, "LD BC, {:04x}", value),
            Instruction::DECBC => write!(f, "DEC BC"),
            Instruction::LDDd8(value) => write!(f, "LD D, {:02x}", value),
            Instruction::ADDHLDE => write!(f, "ADD HL, DE"),
            Instruction::JRNZr8(value) => write!(f, "JR NZ, {}", value),
            Instruction::LDHLd16(value) => write!(f, "LD HL, {:04x}", value),
            Instruction::LDHLincA => write!(f, "LD (HL+), A"),
            Instruction::INCHL => write!(f, "INC HL"),
            Instruction::LDSPd16(value) => write!(f, "LD SP, {:04x}", value),
            Instruction::LDAd8(value) => write!(f, "LD A, {:02x}", value),
            Instruction::LDDHL => write!(f, "LD D, (HL)"),
            Instruction::LDDA => write!(f, "LD E, A"),
            Instruction::LDEHL => write!(f, "LD E, (HL)"),
            Instruction::LDEA => write!(f, "LD E, A"),
            Instruction::LDAB => write!(f, "LD A, B"),
            Instruction::LDAD => write!(f, "LD A, D"),
            Instruction::XORA => write!(f, "XOR A"),
            Instruction::ORC => write!(f, "OR C"),
            Instruction::JPa16(address) => write!(f, "JP {}", address),
            Instruction::RET => write!(f, "RET"),
            Instruction::Special(instr) => instr.fmt(f),
            Instruction::CALLa16(address) => write!(f, "CALL {}", address),
            Instruction::PUSHDE => write!(f, "PUSH DE"),
            Instruction::POPHL => write!(f, "POP HL"),
            Instruction::LDHa8A(value) => write!(f, "LDH (ff{:02x}), A", value),
            Instruction::ANDd8(value) => write!(f, "AND {:02x}", value),
            Instruction::JPHL => write!(f, "JP (HL)"),
            Instruction::LDa16A(address) => write!(f, "LD ({}), A", address),
            Instruction::LDHAa8(value) => write!(f, "LDH A, (ff{:02x})", value),
            Instruction::CPd8(value) => write!(f, "CP {:02x}", value),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Address {
    Bank0(usize),
    UnknownBank(usize),
    Bank(usize, usize),
    System(usize),
}

impl Address {
    pub fn from_physical_address(physical_address: u16) -> Address {
        if physical_address < 0x4000 {
            Address::Bank0(physical_address as usize)
        } else if physical_address < 0x8000 {
            Address::UnknownBank((physical_address & 0x3fff) as usize)
        } else {
            Address::System(physical_address as usize)
        }
    }

    pub fn offset(&self) -> usize {
        match self {
            &Address::Bank0(offset) => offset,
            &Address::UnknownBank(offset) => offset,
            &Address::Bank(_, offset) => offset,
            &Address::System(offset) => offset,
        }
    }

    pub fn bank(&self) -> Option<usize> {
        match self {
            &Address::Bank0(_) => Some(0),
            &Address::UnknownBank(_) => None,
            &Address::Bank(bank, _) => Some(bank),
            &Address::System(_) => None,
        }
    }

    pub fn physical_address(&self) -> Option<usize> {
        match self {
            &Address::Bank0(offset) => Some(offset),
            &Address::UnknownBank(offset) => None,
            &Address::Bank(bank, offset) => Some(bank * 0x4000 + offset),
            &Address::System(_) => None,
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &Address::Bank0(offset) => write!(f, "{:02x}:{:04x}", 0, offset),
            &Address::UnknownBank(offset) => write!(f, "??:{:04x}", offset),
            &Address::Bank(bank, offset) => write!(f, "{:02x}:{:04x}", bank, offset),
            &Address::System(offset) => write!(f, "SYS{:04x}", offset),
        }
    }
}
