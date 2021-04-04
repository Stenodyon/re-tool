use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::iter::FromIterator;

use pancurses::{Input, Window};

mod gb;
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

struct Application {
    running: bool,
    base_address: usize,
    selected_address: usize,

    window: Window,
    byte_store: ByteStore,
    type_changes: Vec<(ByteType, usize)>,
    labels: HashMap<usize, String>,

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
                    self.base_address = address;
                    self.selected_address = address;
                }
            }
            Some(Input::Character('f')) => {
                if self.byte_store.types[self.selected_address] == ByteType::Code {
                    if let Some(address) = self
                        .instruction_at(self.selected_address)
                        .and_then(|instruction| instruction.jump_address())
                        .and_then(|address| address.physical_address())
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
            while let Some(instruction) = Instruction::from_bytes(&self.byte_store.bytes[address..])
            {
                if let Some(address) = instruction.jump_address() {
                    if let Some(physical_address) = address.physical_address() {
                        if self.byte_store.types[physical_address] == ByteType::Unknown {
                            self.type_changes.push((ByteType::Code, physical_address));
                        }

                        if !self.labels.contains_key(&physical_address) {
                            self.labels
                                .insert(physical_address, format!("LOC_{:02X}", physical_address));
                        }
                    }
                }
                if !instruction.fall_through() {
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

    fn instruction_at(&self, address: usize) -> Option<Instruction> {
        Instruction::from_bytes(&self.byte_store.bytes[address..])
    }

    fn draw_header(&self) {
        self.window.addstr(format!(
            "Address: {:04x} | folow_stack = {:?} | follow_stack_top = {}",
            self.selected_address, self.follow_stack, self.follow_stack_top
        ));
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

            let byte = self.byte_store.bytes[line_address];
            let byte_type = self.byte_store.types[line_address];
            self.window.addstr(format!("{:04x}: ", line_address));

            match byte_type {
                ByteType::Unknown => {
                    self.window.addstr(format!("{:02x} ??", byte));
                    offset += 1;
                }
                ByteType::Data => {
                    self.window.addstr(format!("db {:02x}", byte));
                    offset += 1;
                }
                ByteType::Code => {
                    if let Some(instr) =
                        Instruction::from_bytes(&self.byte_store.bytes[line_address..])
                    {
                        for byte_index in 0..instr.size() {
                            let byte = self.byte_store.bytes[line_address + byte_index];
                            self.window.addstr(format!("{:02x} ", byte));
                        }
                        self.window.addstr(format!("{}", instr));
                        offset += instr.size();
                    } else {
                        self.window
                            .addstr(format!("{:02x} Illegal instruction", byte));
                        offset += 1;
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
