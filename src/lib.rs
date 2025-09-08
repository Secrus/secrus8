pub mod consts;
pub mod display;
pub mod interpreter;
pub mod state;

pub struct Chip8Error {}

pub type Result<T> = std::result::Result<T, Chip8Error>;
