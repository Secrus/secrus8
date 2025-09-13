use crate::Result;
use crate::consts::INITIAL_PC;
use crate::display::CLIDisplay;
use crate::parser::Instruction;
use crate::state::State;
use rand::Rng;
use std::io::{self, Write};
use std::thread::sleep;

enum StepResult {
    Continue,
    Halt,
}

pub struct Interpreter {
    state: State,
    display: CLIDisplay,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
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

        let opcode = (b1 as u16) << 8 | (b2 as u16);

        let instruction = Instruction::from_opcode(opcode)?;

        match instruction {
            Instruction::ClearScreen => {
                self.display.clear();
            }
            Instruction::ReturnFromSubroutine => {
                self.state.pc = self.state.stack.pop().expect("empty stack");
            }
            Instruction::Jump(address) => {
                if instruction_address == address {
                    return Ok(StepResult::Halt);
                }
            }
            Instruction::Call(address) => {
                self.state.stack.push(self.state.pc);
                self.state.pc = address;
            }
            Instruction::SkipIfEqualByte(register, value) => {
                if self.state.registers[register] == value {
                    self.state.pc += 2
                }
            }
            Instruction::SkipIfNotEqualByte(register, value) => {
                if self.state.registers[register] != value {
                    self.state.pc += 2
                }
            }
            Instruction::SkipIfRegistersEqual(register_x, register_y) => {
                if self.state.registers[register_x] == self.state.registers[register_y] {
                    self.state.pc += 2
                }
            }
            Instruction::SetRegisterToValue(register, value) => {
                self.state.registers[register] = value;
            }
            Instruction::AddToRegister(register, value) => {
                let (sum, _) = self.state.registers[register].overflowing_add(value);
                self.state.registers[register] = sum;
            }
            Instruction::SetRegisterToRegisterValue(register_x, register_y) => {
                self.state.registers[register_x] = self.state.registers[register_y];
            }
            Instruction::RegistersBitwiseOr(register_x, register_y) => {
                self.state.registers[register_x] |= self.state.registers[register_y];
            }
            Instruction::RegistersBitwiseAnd(register_x, register_y) => {
                self.state.registers[register_x] &= self.state.registers[register_y];
            }
            Instruction::RegistersBitwiseXor(register_x, register_y) => {
                self.state.registers[register_x] ^= self.state.registers[register_y];
            }
            Instruction::RegistersSumWithOverflow(register_x, register_y) => {
                let vx = self.state.registers[register_x];
                let vy = self.state.registers[register_y];

                let (sum, overflow) = vx.overflowing_add(vy);
                self.state.registers[register_x] = sum;
                self.state.registers[0xF] = if overflow { 1 } else { 0 };
            }
            Instruction::SubtractRegisterFromRegisterValue(register_x, register_y) => {
                let vx = self.state.registers[register_x];
                let vy = self.state.registers[register_y];

                let (diff, overflow) = vx.overflowing_sub(vy);
                self.state.registers[register_x] = diff;
                self.state.registers[0xF] = if overflow { 0 } else { 1 };
            }
            Instruction::ShiftRegisterBitsRight(register) => {
                self.state.registers[0xF] = self.state.registers[register] & 1;
                self.state.registers[register] >>= 1;
            }
            Instruction::SubtractRegisterValueFromRegister(register_x, register_y) => {
                let vx = self.state.registers[register_x];
                let vy = self.state.registers[register_y];

                let (diff, overflow) = vy.overflowing_sub(vx);
                self.state.registers[register_x] = diff;
                // Set VF to 1 if there was NO borrow
                self.state.registers[0xF] = if overflow { 0 } else { 1 };
            }
            Instruction::ShiftRegisterBitsLeft(register) => {
                self.state.registers[0xF] = self.state.registers[register] >> 7;
                self.state.registers[register] <<= 1;
            }
            Instruction::SkipIfRegistersNotEqual(register_x, register_y) => {
                if self.state.registers[register_x] != self.state.registers[register_y] {
                    self.state.pc += 2;
                }
            }
            Instruction::SetIndexRegisterToValue(value) => {
                self.state.index_register = value;
            }
            Instruction::JumpByValue(value) => {
                self.state.pc = self.state.registers[0] as u16 + value;
            }
            Instruction::SetRegisterToRandAndValue(register, value) => {
                let mut rng = rand::rng();
                let n: u8 = rng.random_range(0..=255);
                self.state.registers[register] = n & value;
            }
            Instruction::DrawSprite(register_x, register_y, sprite) => {
                let x: u8 = self.state.registers[register_x];
                let y: u8 = self.state.registers[register_y];
                let start = self.state.index_register as usize;
                let end = (self.state.index_register + sprite as u16) as usize;
                self.state.registers[0xF] = {
                    if self.display.draw(x, y, &self.state.ram[start..end]) {
                        1
                    } else {
                        0
                    }
                };
                self.display.show();
            }
            Instruction::SkipIfKeyEqualsRegister(register) => {
                unimplemented!();
            }
            Instruction::SkipIfKeyNotEqualsRegister(register) => {
                unimplemented!();
            }
            Instruction::SetRegisterToDelayTimerValue(register) => {
                self.update_timers();
                self.state.registers[register] = self.state.delay_timer;
            }
            Instruction::SetDelayTimerToRegisterValue(register) => {
                self.state.delay_timer = self.state.registers[register];
            }
            Instruction::SetSoundTimerToRegisterValue(register) => {
                self.state.sound_timer = self.state.registers[register];
            }
            Instruction::SetIndexRegisterToSpriteForRegister(register) => {
                let character = self.state.registers[register];
                self.state.index_register = 0x50 + (character as u16 * 5);
            }
            Instruction::StoreBinaryCodedDecimalAtIndexRegisterValue(register) => {
                let num = self.state.registers[register];
                let i = self.state.index_register as usize;
                // Hundreds digit
                self.state.ram[i] = num / 100;
                // Tens digit
                self.state.ram[i + 1] = (num / 10) % 10;
                // Ones digit
                self.state.ram[i + 2] = num % 10;
            }
            Instruction::AddRegisterToIndexRegister(register) => {
                self.state.index_register += self.state.registers[register] as u16;
            }
            Instruction::DumpRegistersToMemoryAtIndexRegister(register) => {
                for ri in 0..=register {
                    self.state.ram[(self.state.index_register + ri as u16) as usize] =
                        self.state.registers[ri];
                }
            }
            Instruction::LoadMemoryToRegistersAtIndexRegister(register) => {
                for ri in 0..=register {
                    self.state.registers[ri] =
                        self.state.ram[(self.state.index_register + ri as u16) as usize];
                }
            }
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
