use std::marker::PhantomData;

pub struct DisassemblerState<'a> {
    rom: &'a [u8],
    byte_type: Vec<ByteType>,
}

impl<'a> DisassemblerState<'a> {
    pub fn new(rom: &[u8]) -> DisassemblerState {
        let size = rom.len();
        DisassemblerState {
            rom,
            byte_type: vec![ByteType::Unknown; size],
        }
    }
}

pub struct Disassembler<'a, Architecture> {
    state: DisassemblerState<'a>,

    phantom: PhantomData<Architecture>,
}

// TODO events
// TODO display
impl<'a, Arch: Architecture> Disassembler<'a, Arch> {
    pub fn new(rom: &[u8]) -> Disassembler<Arch> {
        let size = rom.len();
        Disassembler {
            state: DisassemblerState::new(rom),

            phantom: PhantomData,
        }
    }

    pub fn mark_data(&mut self, address: usize) {
        self.state.byte_type[address] = ByteType::Data;
    }

    pub fn mark_unknown(&mut self, address: usize) {
        self.state.byte_type[address] = ByteType::Unknown;
    }

    pub fn mark_code(&mut self, mut address: usize) {
        let mut branches = vec![address];
        while let Some(mut address) = branches.pop() {
            while let Some(instruction) = Arch::disassemble(&self.state.rom[address..]) {
                if self.state.byte_type[address] != ByteType::Code {
                    break;
                }
                self.state.byte_type[address] = ByteType::Code;
                if let Some(branch_address) =
                    instruction.branch_address().and_then(|branch_address| {
                        Arch::resolve_address(branch_address, address, &self.state)
                    })
                {
                    branches.push(branch_address);
                }
                if !instruction.falls_through() {
                    break;
                }
                address += instruction.size();
            }
        }
    }

    pub fn resolve_branch_address(&self, location: usize) -> Option<usize> {
        Arch::disassemble(&self.state.rom[location..]).and_then(|instruction| {
            instruction.branch_address().and_then(|unresolved_address| {
                Arch::resolve_address(unresolved_address, location, &self.state)
            })
        })
    }

    fn align_address_to_valid_location(&self, address: usize) -> usize {
        match self.state.byte_type[address] {
            ByteType::Data | ByteType::Code => address,
            ByteType::Unknown => {
                for back_offset in 1..3.min(address + 1) {
                    if let Some(_) = Arch::disassemble(&self.state.rom[address - back_offset..]) {
                        return address - back_offset;
                    }
                }
                address
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ByteType {
    Unknown,
    Code,
    Data,
}

pub enum AddressRepr<'a> {
    Unknown {
        address: usize,
        byte: u8,
    },
    Data {
        address: usize,
        byte: u8,
    },
    Code {
        address: usize,
        bytes: &'a [u8],
        instruction_name: &'a str,
        arguments: &'a str,
    },
    Label {
        name: &'a str,
    },
}

pub trait Architecture {
    type Instruction: self::Instruction;

    fn disassemble(bytes: &[u8]) -> Option<Self::Instruction>;

    fn resolve_address(
        address: LogicalAddress,
        location: usize,
        state: &DisassemblerState,
    ) -> Option<usize>;
}

pub trait Instruction {
    /// The size of the instruction in bytes
    fn size(&self) -> usize;

    /// Whether execution can continue after this instruction
    fn falls_through(&self) -> bool;

    /// The logical address that execution can jump to from this address, None if
    /// this isn't a branch instruction
    fn branch_address(&self) -> Option<LogicalAddress>;
}

pub enum LogicalAddress {
    Absolute(usize),
    Relative(isize),
}
