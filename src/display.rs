use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};

use std::io::{self, Write};

pub struct CLIDisplay {
    screen: [[u8; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize],
}

impl Default for CLIDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl CLIDisplay {
    pub fn new() -> Self {
        CLIDisplay {
            screen: [[0; _]; _],
        }
    }

    pub fn clear(&mut self) {
        self.screen = [[0; _]; _];
    }

    pub fn show(&self) {
        // Improve the screen display code for more interactive terminal

        print!("\x1B[2J\x1B[H");

        io::stdout().flush().unwrap();

        let mut res = String::new();

        for row in &self.screen {
            for &pixel in row {
                res.push(if pixel == 0 { '░' } else { '█' });
            }

            res.push('\n');
        }

        print!("{}", res);

        io::stdout().flush().unwrap();
    }

    pub fn draw(&mut self, reg_x: u8, reg_y: u8, sprite: &[u8]) -> bool {
        let x = reg_x % SCREEN_WIDTH;
        let y = reg_y % SCREEN_HEIGHT;
        let mut did_switch: bool = false;

        for (yo, data) in sprite.iter().enumerate() {
            let row = y as usize + yo;
            if row >= SCREEN_HEIGHT as usize {
                break;
            }

            for (xo, bit) in byte_to_bits(*data).iter().enumerate() {
                let col = x as usize + xo;
                if col >= SCREEN_WIDTH as usize {
                    break;
                }

                if *bit == 1 {
                    if self.screen[row][col] == 1 {
                        self.screen[row][col] = 0;
                        did_switch = true;
                    } else {
                        self.screen[row][col] = 1;
                    }
                }
            }
        }
        did_switch
    }
}

/// Bits as 0 or 1 u8 from the most to least significant
fn byte_to_bits(b: u8) -> [u8; 8] {
    std::array::from_fn(|i| (b >> (7 - i)) & 1)
}
