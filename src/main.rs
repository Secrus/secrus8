use std::env;
use std::fs::File;
use std::io::{self, Read};

use secrus8::chip_core::Core;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check that a filename was provided
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    // Open the file in read-only mode
    let mut file = File::open(filename)?;

    // Read the file contents into a buffer
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // You can now use `buffer` as your binary data
    println!("Read {} bytes from file '{}'", buffer.len(), filename);

    // println!("Read bytes:");
    // for (i, byte) in buffer.iter().take(buffer.len()).enumerate() {
    //     if i % 8 == 0 {
    //         println!();
    //     }
    //     print!("{:02X} ", byte);
    // }
    // println!();

    let mut core = Core::new();
    core.load_rom(buffer);
    core.run();

    Ok(())
}
