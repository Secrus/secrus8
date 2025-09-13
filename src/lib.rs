pub mod consts;
pub mod display;
pub mod interpreter;
mod parser;
pub mod state;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnknownOpcode(u16),
    InsufficientData,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Self::UnknownOpcode(code) => write!(f, "Unknown opcode: {:x}", code),
            Self::InsufficientData => write!(f, "Not enough data"),
        }
    }
}

impl core::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
