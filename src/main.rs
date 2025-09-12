use std::env;
use std::fs::File;
use std::io;

use secrus8::interpreter::Interpreter;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check that a filename was provided
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    // Open the file in read-only mode
    let file = File::open(filename)?;

    let mut core = Interpreter::new();
    core.load_rom(file);
    core.run();

    Ok(())
}
