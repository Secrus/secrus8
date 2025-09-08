use crate::consts::INITIAL_PC;
use crate::display::CLIDisplay;
use crate::state::State;
use crate::{Chip8Error, Result};
use rand::Rng;
use std::io::{self, Write};
use std::thread::sleep;

enum StepResult {
    Continue,
    Halt,
}

pub struct Core {
    state: State,
    display: CLIDisplay,
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

impl Core {
    pub fn new() -> Self {
        Core {
            state: State::new(),
            display: CLIDisplay::new(),
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        let start = INITIAL_PC as usize;
        let end = start + rom.len();
        self.state.ram[start..end].copy_from_slice(&rom);
    }

    pub fn run(&mut self) {
        // --- Timing Configuration ---
        const TARGET_FPS: u32 = 60;
        const TARGET_IPS: u32 = 700;
        const INSTRUCTIONS_PER_FRAME: u32 = TARGET_IPS / TARGET_FPS;
        let frame_duration = std::time::Duration::from_secs_f32(1.0 / TARGET_FPS as f32);

        // --- Main Emulator Loop ---
        'main_loop: loop {
            let frame_start = std::time::Instant::now();

            for _ in 0..INSTRUCTIONS_PER_FRAME {
                match self.step() {
                    Ok(StepResult::Continue) => {}
                    Ok(StepResult::Halt) => {
                        println!("\nProgram finished. Exiting.");
                        break 'main_loop;
                    }
                    Err(_) => {
                        eprintln!("\nExecution error. Exiting.");
                        break 'main_loop;
                    }
                }
            }

            self.update_timers();

            let elapsed = frame_start.elapsed();
            if let Some(sleep_time) = frame_duration.checked_sub(elapsed) {
                sleep(sleep_time);
            }
        }
    }

    fn step(&mut self) -> Result<StepResult> {
        let instruction_address = self.state.pc;

        let b1 = self.state.ram[self.state.pc as usize];
        let b2 = self.state.ram[(self.state.pc + 1) as usize];

        // Increment program counter by 2

        self.state.pc += 2;

        // Split into 4 nibbles
        let n1 = b1 >> 4;
        let n2 = b1 & 0x0F;
        let n3 = b2 >> 4;
        let n4 = b2 & 0x0F;

        match (n1, n2, n3, n4) {
            // 00E0 - clear screen
            (0, 0, 0xE, 0) => {
                self.display.clear();
            }
            // 00EE - return from subroutine
            (0, 0, 0xE, 0xE) => {
                self.state.pc = self.state.stack.pop().expect("empty stack");
            }
            // 1NNN - jump to NNN
            (1, _, _, _) => {
                let target_address = ((n2 as u16) << 8) | b2 as u16;
                if instruction_address == target_address {
                    return Ok(StepResult::Halt);
                }
            }
            // 2NNN - call subroutine at NNN
            (2, _, _, _) => {
                self.state.stack.push(self.state.pc);
                self.state.pc = ((n2 as u16) << 8) | b2 as u16;
            }
            // 3XNN - skip next if VX equals NN
            (3, _, _, _) => {
                if self.state.registers[n2 as usize] == b2 {
                    self.state.pc += 2
                }
            }
            // 4XNN - skip next if VX does not equal NN
            (4, _, _, _) => {
                if self.state.registers[n2 as usize] != b2 {
                    self.state.pc += 2
                }
            }
            // 5XY0 - skip next if VX equals VY
            (5, _, _, 0) => {
                if self.state.registers[n2 as usize] == self.state.registers[n3 as usize] {
                    self.state.pc += 2
                }
            }
            // 6XNN - set VX to NN
            (6, _, _, _) => {
                self.state.registers[n2 as usize] = b2;
            }
            // 7XNN - add NN to VX
            (7, _, _, _) => {
                let (sum, _) = self.state.registers[n2 as usize].overflowing_add(b2);
                self.state.registers[n2 as usize] = sum;
            }
            // 8XY0 - set VX to value of VY
            (8, _, _, 0) => {
                self.state.registers[n2 as usize] = self.state.registers[n3 as usize];
            }
            // 8XY1 - set VX | VY
            (8, _, _, 1) => {
                self.state.registers[n2 as usize] |= self.state.registers[n3 as usize];
            }
            // 8XY2 - set VX & VY
            (8, _, _, 2) => {
                self.state.registers[n2 as usize] &= self.state.registers[n3 as usize];
            }
            // 8XY3 - set VX ^ VY
            (8, _, _, 3) => {
                self.state.registers[n2 as usize] ^= self.state.registers[n3 as usize];
            }
            // 8XY4 - add VY to VX (with VF as overflow control)
            (8, n2, n3, 4) => {
                let vx = self.state.registers[n2 as usize];
                let vy = self.state.registers[n3 as usize];

                let (sum, overflow) = vx.overflowing_add(vy);
                self.state.registers[n2 as usize] = sum;
                self.state.registers[0xF] = if overflow { 1 } else { 0 };
            }
            // 8XY5 - VX = VX - VY (with VF as overflow control)
            (8, _, _, 5) => {
                let vx = self.state.registers[n2 as usize];
                let vy = self.state.registers[n3 as usize];

                let (diff, overflow) = vx.overflowing_sub(vy);
                self.state.registers[n2 as usize] = diff;
                self.state.registers[0xF] = if overflow { 0 } else { 1 };
            }
            // 8XY6 - VX >>= 1, LSB stored in VF
            (8, _, _, 6) => {
                self.state.registers[0xF] = self.state.registers[n2 as usize] & 1;
                self.state.registers[n2 as usize] >>= 1;
            }
            // 8XY7 - VX = VY - VX (with VF as overflow control)
            (8, n2, n3, 7) => {
                let vx = self.state.registers[n2 as usize];
                let vy = self.state.registers[n3 as usize];

                let (diff, overflow) = vy.overflowing_sub(vx);
                self.state.registers[n2 as usize] = diff;
                // Set VF to 1 if there was NO borrow (overflow is false)
                self.state.registers[0xF] = if overflow { 0 } else { 1 };
            }
            // 8XYE - VX <<= 1 (with VF as overflow control)
            (8, _, _, 0xE) => {
                self.state.registers[0xF] = self.state.registers[n2 as usize] >> 7;
                self.state.registers[n2 as usize] <<= 1;
            }
            // 9XY0 - skip next if VX does not equal VY
            (9, _, _, 0) => {
                if self.state.registers[n2 as usize] != self.state.registers[n3 as usize] {
                    self.state.pc += 2;
                }
            }
            // ANNN - set I to NNN
            (0xA, _, _, _) => {
                self.state.index_register = ((n2 as u16) << 8) | b2 as u16;
            }
            // BNNN - jump to V0 + NNN
            (0xB, _, _, _) => {
                self.state.pc = self.state.registers[0] as u16 + ((n2 as u16) << 8 | b2 as u16);
            }
            // CXNN - set VX to rand(0, 255) & NN
            (0xC, _, _, _) => {
                let mut rng = rand::rng();
                let n: u8 = rng.random_range(0..=255);
                self.state.registers[n2 as usize] = n & b2;
            }
            // DXYN - draw a sprite
            (0xD, vx, vy, n) => {
                let x: u8 = self.state.registers[vx as usize];
                let y: u8 = self.state.registers[vy as usize];
                let start = self.state.index_register as usize;
                let end = (self.state.index_register + n as u16) as usize;
                self.state.registers[0xF] = {
                    if self.display.draw(x, y, &self.state.ram[start..end]) {
                        1
                    } else {
                        0
                    }
                };
                self.display.show();
            }
            // EX9E - skip next if key == VX
            (0xE, _, 9, 0xE) => {
                unimplemented!();
            }
            // EXA1 - skip next if key != VX
            (0xE, _, 0xA, 1) => {
                unimplemented!();
            }
            // FX07 - set VX to delay timer value
            (0xF, _, 0, 7) => {
                self.update_timers();
                self.state.registers[n2 as usize] = self.state.delay_timer;
            }
            // FX15 - set delay timer to VX
            (0xF, _, 1, 5) => {
                self.state.delay_timer = self.state.registers[n2 as usize];
            }
            // FX18 - set sound timer to VX
            (0xF, _, 1, 8) => {
                self.state.sound_timer = self.state.registers[n2 as usize];
            }
            // FX29 - set I to location of sprite for character in VX
            (0xF, n2, 2, 9) => {
                let character = self.state.registers[n2 as usize];
                self.state.index_register = 0x50 + (character as u16 * 5);
            }
            // FX33 - store binary coded decimal at memory under I(I+1)(I+2)
            (0xF, n2, 3, 3) => {
                let num = self.state.registers[n2 as usize];
                let i = self.state.index_register as usize;
                // Hundreds digit
                self.state.ram[i] = num / 100;
                // Tens digit
                self.state.ram[i + 1] = (num / 10) % 10;
                // Ones digit
                self.state.ram[i + 2] = num % 10;
            }
            // FX1E - add VX to I (don't consider overflow)
            (0xF, _, 1, 0xE) => {
                self.state.index_register += self.state.registers[n2 as usize] as u16;
            }
            // FX55 - dump registers V0 to VX in memory, starting from I
            (0xF, _, 5, 5) => {
                for ri in 0..=n2 {
                    self.state.ram[(self.state.index_register + ri as u16) as usize] =
                        self.state.registers[ri as usize];
                }
            }
            // FX65 - load memory starting from I into V0 to VX
            (0xF, _, 6, 5) => {
                for ri in 0..=n2 {
                    self.state.registers[ri as usize] =
                        self.state.ram[(self.state.index_register + ri as u16) as usize];
                }
            }
            // unknown opcode
            _ => return Err(Chip8Error {}),
        }

        Ok(StepResult::Continue)
    }

    fn update_timers(&mut self) {
        if self.state.delay_timer > 0 {
            self.state.delay_timer -= 1;
        }

        if self.state.sound_timer > 0 {
            // A simple terminal beep for sound feedback
            if self.state.sound_timer == 1 {
                print!("\x07");
                io::stdout().flush().unwrap();
            }
            self.state.sound_timer -= 1;
        }
    }
}
