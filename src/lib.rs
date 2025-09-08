pub mod chip_core;
pub mod consts;
pub mod display;
pub mod state;

pub struct Chip8Error {}

pub type Result<T> = std::result::Result<T, Chip8Error>;
