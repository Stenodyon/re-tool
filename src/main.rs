use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use pancurses::{Input, Window};

mod disassembler;
mod gb;
use disassembler::*;
use gb::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ByteType {
    Unknown,
    Data,
    Code,
}

struct ByteStore {
    pub bytes: Vec<u8>,
    pub types: Vec<ByteType>,
}

enum ResolvedAddress {
    Physical(usize),
    UnknownBank(u16),
    System(u16),
}

impl ResolvedAddress {
    pub fn get(&self) -> Option<usize> {
        match self {
            &Self::Physical(address) => Some(address),
            _ => None,
        }
    }
}

struct Application {
    running: bool,
    base_address: usize,
    selected_address: usize,

    window: Window,
    byte_store: ByteStore,
    type_changes: Vec<(ByteType, usize)>,
    labels: HashMap<usize, String>,
    banks: HashMap<usize, usize>,

    /// Contains the addresses from which a follow command was issued, used to rewind follows
    follow_stack: Vec<usize>,
    follow_stack_top: usize,
}

impl Application {
    pub fn new(rom_data: Vec<u8>) -> Application {
        let rom_data_length = rom_data.len();
        let window = pancurses::initscr();
        pancurses::noecho();
        pancurses::curs_set(0);
        Application {
            running: false,
            base_address: 0,
            selected_address: 0,

            window,
            byte_store: ByteStore {
                bytes: rom_data,
                types: vec![ByteType::Unknown; rom_data_length],
            },
            type_changes: Vec::new(),
            labels: HashMap::new(),
            banks: HashMap::new(),

            follow_stack: Vec::new(),
            follow_stack_top: 0,
        }
    }

    pub fn run(&mut self) {
        self.running = true;
        while self.running {
            self.window.clear();
            self.window.mv(0, 0);
            self.draw_header();
            self.window.mv(3, 0);
            self.draw_hline();
            self.draw_byte_store();
            self.handle_input();
            self.handle_type_changes();
        }
    }

    fn handle_input(&mut self) {
        let input = self.window.getch();
        self.window.mv(2, 0);
        self.window.addstr(format!("{:?}", input));
        match input {
            None => {}
            Some(Input::Character('\u{1b}')) | Some(Input::Character('q')) => self.running = false,
            Some(Input::Character('j')) => {
                self.selected_address = self.next_valid_address(self.selected_address)
            }
            Some(Input::Character('k')) if self.selected_address > 0 => {
                self.selected_address = self.snap_to_valid_address(self.selected_address - 1);
            }
            Some(Input::Character('c')) => {
                self.type_changes
                    .push((ByteType::Code, self.selected_address));
            }
            Some(Input::Character('d')) => {
                self.type_changes
                    .push((ByteType::Data, self.selected_address));
            }
            Some(Input::Character('G')) => {
                if let Ok(address) = usize::from_str_radix(&self.read_line("Go to address: "), 16) {
                    self.push_follow(self.selected_address);
                    self.base_address = address;
                    self.selected_address = address;
                }
            }
            Some(Input::Character('f')) => {
                if self.byte_store.types[self.selected_address] == ByteType::Code {
                    if let Some(address) = self
                        .instruction_at(self.selected_address)
                        .and_then(|instruction| instruction.jump_address())
                        .and_then(|address| {
                            self.resolve_physical_address(self.selected_address, address)
                                .get()
                        })
                    {
                        self.push_follow(self.selected_address);
                        self.base_address = address;
                        self.selected_address = address;
                    }
                }
            }
            Some(Input::Character('o')) => {
                if let Some(address) = self.follow_stack_previous() {
                    self.base_address = address;
                    self.selected_address = address;
                }
            }
            Some(Input::Character('i')) => {
                if let Some(address) = self.follow_stack_next() {
                    self.base_address = address;
                    self.selected_address = address;
                }
            }
            Some(Input::Character('b')) => {
                if let Ok(bank) = usize::from_str_radix(&self.read_line("Bank number: "), 16) {
                    self.banks.insert(self.selected_address, bank);
                }
            }
            Some(Input::Character('l')) => {
                let label = self.read_line("label");
                if !label.is_empty() {
                    self.labels.insert(self.selected_address, label);
                }
            }
            Some(_) => {}
        }
    }

    fn handle_type_changes(&mut self) {
        while !self.type_changes.is_empty() {
            self.handle_type_change();
        }
    }

    fn handle_type_change(&mut self) {
        let (byte_type, mut address) = self.type_changes.pop().unwrap();

        self.byte_store.types[address] = byte_type;

        if byte_type == ByteType::Code {
            while let Some(instruction) =
                GBInstruction::from_bytes(&self.byte_store.bytes[address..])
            {
                if let Some(unmapped_address) = instruction.jump_address() {
                    if let Some(physical_address) = self
                        .resolve_physical_address(address, unmapped_address)
                        .get()
                    {
                        if self.byte_store.types[physical_address as usize] == ByteType::Unknown {
                            self.type_changes.push((ByteType::Code, physical_address));
                        }

                        if !self.labels.contains_key(&physical_address) {
                            self.labels
                                .insert(physical_address, format!("LOC_{:06X}", physical_address));
                        }
                    }
                }
                if !instruction.falls_through() {
                    break;
                }
                address += instruction.size();
                if self.byte_store.types[address] != ByteType::Unknown {
                    break;
                }
                self.byte_store.types[address] = ByteType::Code;
            }
        }
    }

    fn read_line(&self, prompt: &str) -> String {
        self.window.mvaddstr(1, 0, prompt);
        pancurses::echo();
        pancurses::nocbreak();
        pancurses::curs_set(2);
        let mut string = String::new();
        let string = loop {
            match self.window.getch() {
                Some(Input::Character('\n')) => {
                    break string;
                }
                Some(Input::Character(c)) => {
                    string.push(c);
                }
                _ => {}
            }
        };
        pancurses::noecho();
        pancurses::cbreak();
        pancurses::curs_set(0);

        self.window.mv(1, 0);
        self.clear_line();

        string
    }

    fn instruction_at(&self, address: usize) -> Option<GBInstruction> {
        GBInstruction::from_bytes(&self.byte_store.bytes[address..])
    }

    fn draw_header(&self) {
        self.window
            .addstr(format!("Address: {:04x}", self.selected_address));

        if let Some(instruction) = self.instruction_at(self.selected_address) {
            self.window.addstr(format!(" {}", instruction.name()));
            if let Some(first_argument) = instruction.first_argument() {
                self.window.addstr(" ");
                self.draw_argument(self.selected_address, &first_argument);

                if let Some(second_argument) = instruction.second_argument() {
                    self.window.addstr(", ");
                    self.draw_argument(self.selected_address, &second_argument);
                }
            }

            if self.byte_store.types[self.selected_address] != ByteType::Code {
                self.window.addstr(" [c]ode");
            }

            if let Some(address) = instruction.jump_address().and_then(|address| {
                self.resolve_physical_address(self.selected_address, address)
                    .get()
            }) {
                self.window.addstr(format!(" [f]ollow ({:04x})", address));
            }
        }

        if self.byte_store.types[self.selected_address] != ByteType::Data {
            self.window.addstr(" [d]ata");
        }
        self.window.addstr(" [G]oto [b]ank");
    }

    fn draw_hline(&self) {
        let height = self.window.get_cur_y();
        let width = self.window.get_max_x();
        self.window.mv(height, 0);
        for _ in 0..width {
            self.window.addch('-');
        }
    }

    fn clear_line(&self) {
        let height = self.window.get_cur_y();
        let width = self.window.get_max_x();
        self.window.mv(height, 0);
        for _ in 0..width {
            self.window.addch(' ');
        }
    }

    fn draw_byte_store(&mut self) {
        let y0 = self.window.get_cur_y();
        let height = (self.window.get_max_y() - 1 - y0) as usize;
        if self.selected_address < self.base_address {
            self.base_address = self.selected_address;
        }
        if self.selected_address > self.base_address + height {
            self.base_address = self.selected_address - height - 1;
        }

        self.window.mv(y0, 0);
        let mut offset = 0usize;
        loop {
            if self.base_address + offset >= self.byte_store.bytes.len() {
                break;
            }

            let line_address = self.base_address + offset;

            if let Some(label) = self.labels.get(&line_address) {
                self.window.addstr(format!("{}:\n", label));
            }

            if line_address == self.selected_address {
                self.window.attron(pancurses::A_REVERSE);
            } else {
                self.window.attroff(pancurses::A_REVERSE);
            }

            let byte = self.byte_store.bytes[line_address as usize];
            let byte_type = self.byte_store.types[line_address as usize];
            self.window.addstr(format!("{:06x}: ", line_address));

            match byte_type {
                ByteType::Unknown => {
                    self.window.addstr(format!("{:02x}", byte));
                    offset += 1;
                    self.window.mv(self.window.get_cur_y(), 20);
                    self.window.addstr("??");
                }
                ByteType::Data => {
                    self.window.addstr(format!("{:02x}", byte));
                    offset += 1;
                    self.window.mv(self.window.get_cur_y(), 20);
                    self.window.addstr("db");
                }
                ByteType::Code => {
                    if let Some(instr) =
                        GBInstruction::from_bytes(&self.byte_store.bytes[line_address..])
                    {
                        for byte_index in 0..instr.size() {
                            let byte = self.byte_store.bytes[line_address + byte_index];
                            self.window.addstr(format!("{:02x} ", byte));
                        }
                        self.window.mv(self.window.get_cur_y(), 20);
                        self.draw_instruction(line_address, &instr);
                        offset += instr.size();
                    } else {
                        self.window.addstr(format!("{:02x}", byte));
                        offset += 1;
                        self.window.mv(self.window.get_cur_y(), 20);
                        self.window.addstr("Illegal instruction");
                    }
                }
            }

            if line_address == self.selected_address {
                let width = self.window.get_max_x();
                self.window.chgat(width, pancurses::A_REVERSE, 0);
            }

            if self.window.get_cur_y() < self.window.get_max_y() - 1 {
                self.window.mv(self.window.get_cur_y() + 1, 0);
            } else {
                break;
            }
        }
    }

    fn draw_instruction(&self, read_at: usize, instruction: &GBInstruction) {
        let base_x = self.window.get_cur_x();
        self.window.addstr(instruction.name());
        if let Some(first_argument) = instruction.first_argument() {
            self.window.mv(self.window.get_cur_y(), base_x + 6);
            self.draw_argument(read_at, &first_argument);

            if let Some(second_argument) = instruction.second_argument() {
                self.window.addstr(", ");
                self.draw_argument(read_at, &second_argument);
            }
        }
    }

    fn draw_argument(&self, read_at: usize, argument: &Argument) {
        match argument {
            &Argument::Imm8(value) => {
                self.window.addstr(format!("{:02x}", value));
            }
            &Argument::Imm16(value) => {
                self.window.addstr(format!("{:04x}", value));
            }
            &Argument::Rel8(value) => {
                self.window
                    .addstr(format!("({:04x})", read_at.wrapping_add(value as usize)));
            }
            &Argument::Reg8(register) => {
                self.window.addstr(format!("{}", register));
            }
            &Argument::Reg16(register) => {
                self.window.addstr(format!("{}", register));
            }
            &Argument::Address(unmapped_address) => {
                match self.resolve_physical_address(read_at, unmapped_address) {
                    ResolvedAddress::Physical(address) => {
                        if let Some(label) = self.labels.get(&address) {
                            self.window.addstr(label);
                        } else {
                            self.window.addstr(format!("({:06x})", address));
                        }
                    }
                    ResolvedAddress::UnknownBank(offset) => {
                        self.window.addstr(format!("(??:{:04x})", offset));
                    }
                    ResolvedAddress::System(address) => {
                        let name = match address {
                            _ => format!("(SYS:{:04x})", address),
                        };
                        self.window.addstr(name);
                    }
                }
            }
            &Argument::IndirectReg16(register) => {
                self.window.addstr(format!("({})", register));
            }
            &Argument::IndirectHLinc => {
                self.window.addstr("(HL+)");
            }
            &Argument::IndirectHLdec => {
                self.window.addstr("(HL-)");
            }
            &Argument::IndirectC => {
                self.window.addstr("(SYS:ff00 + C)");
            }
            &Argument::ResetVector(reset_vector) => {
                self.window.addstr(format!("{}", reset_vector));
            }
        }
    }

    fn snap_to_valid_address(&self, address: usize) -> usize {
        for backoffset in 1..3.min(address + 1) {
            let offset_address = address - backoffset;
            if self.byte_store.types[offset_address] == ByteType::Code {
                if let Some(instruction) = self.instruction_at(offset_address) {
                    if instruction.size() > backoffset {
                        return offset_address;
                    }
                }
            }
        }
        return address;
    }

    fn next_valid_address(&self, address: usize) -> usize {
        if self.byte_store.types[address] == ByteType::Code {
            if let Some(instruction) = self.instruction_at(address) {
                return address + instruction.size();
            }
        }
        return address + 1;
    }

    fn resolve_physical_address(
        &self,
        read_at: usize,
        address: UnmappedAddress,
    ) -> ResolvedAddress {
        if address.0 < 0x4000 {
            return ResolvedAddress::Physical(address.0 as usize);
        } else if address.0 < 0x8000 {
            let offset = (address.0 & 0x3fff) as usize;
            if read_at >= 0x4000 && read_at < 0x8000 {
                // We're already in the bank, so we know its number
                let bank = read_at / 0x4000;
                return ResolvedAddress::Physical(bank * 0x4000 + offset);
            }
            if let Some(bank) = self.banks.get(&read_at) {
                return ResolvedAddress::Physical(bank * 0x4000 + offset);
            } else {
                return ResolvedAddress::UnknownBank(address.0 & 0x3fff);
            }
        } else {
            return ResolvedAddress::System(address.0);
        }
    }

    fn push_follow(&mut self, address: usize) {
        if self.follow_stack_top == self.follow_stack.len() {
            self.follow_stack.push(address);
        } else {
            self.follow_stack[self.follow_stack_top] = address;
        }
        self.follow_stack_top += 1;
    }

    fn follow_stack_previous(&mut self) -> Option<usize> {
        if self.follow_stack_top == 0 {
            return None;
        }

        self.follow_stack_top -= 1;
        Some(self.follow_stack[self.follow_stack_top])
    }

    fn follow_stack_next(&mut self) -> Option<usize> {
        if self.follow_stack_top == self.follow_stack.len() {
            return None;
        }

        let address = self.follow_stack[self.follow_stack_top];
        self.follow_stack_top += 1;
        Some(address)
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        pancurses::endwin();
    }
}

fn main() {
    let matches = clap::App::new("gbretools")
        .arg(clap::Arg::with_name("rom_file").required(true))
        .get_matches();

    let filename = matches.value_of("rom_file").unwrap();
    let mut rom_file = File::open(filename).expect(&format!("Unable to open file {}", filename));
    let mut rom_data = Vec::new();
    rom_file.read_to_end(&mut rom_data).unwrap();

    let mut application = Application::new(rom_data);
    application.run();
}
