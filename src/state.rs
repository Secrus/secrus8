use crate::consts::{FONT_DATA, INITIAL_PC, TOTAL_RAM_SIZE};

pub struct State {
    pub ram: [u8; TOTAL_RAM_SIZE as usize],
    pub stack: Vec<u16>,
    pub pc: u16,
    pub registers: [u8; 16],
    pub index_register: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        let mut ram = [0; TOTAL_RAM_SIZE as usize];
        ram[0x50..=0x9F].copy_from_slice(&FONT_DATA);
        State {
            ram,
            stack: Vec::new(),
            pc: INITIAL_PC,
            registers: [0; 16],
            index_register: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}
