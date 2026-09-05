use std::io::{ self, Read, Write };
use std::path::Path;

fn bf(source: &str) {
    let mut cells = [0u8; 32768];
    let mut ptr = 0usize;
    let mut pc = 0usize;

    let bytes = source.as_bytes();

    let mut stack = Vec::new();
    let mut jump = vec![usize::MAX; bytes.len()];

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' => {
                stack.push(i);
            }

            b']' => {
                if let Some(j) = stack.pop() {
                    jump[i] = j;
                    jump[j] = i;
                } else {
                    eprintln!("Unmatched ']' at position {}", i);
                    std::process::exit(1);
                }
            }

            _ => {}
        }
    }

    let mut stdin_buf = io::stdin();
    let mut stdout = io::stdout();

    while pc < bytes.len() {
        match bytes[pc] {
            b'>' => {
                ptr = ptr.wrapping_add(1) % cells.len();
            }

            b'<' => {
                ptr = ptr.wrapping_sub(1) % cells.len();
            }

            b'+' => {
                cells[ptr] = cells[ptr].wrapping_add(1);
            }

            b'-' => {
                cells[ptr] = cells[ptr].wrapping_sub(1);
            }

            b'.' => {
                stdout.write_all(&[cells[ptr]]).unwrap();
                stdout.flush().unwrap();
            }

            b',' => {
                let mut buf = [0u8; 1];
                if stdin_buf.read_exact(&mut buf).is_err() {
                    buf[0] = 0;
                }
                cells[ptr] = buf[0];
            }

            b'[' => {
                if cells[ptr] == 0 {
                    pc = jump[pc];
                }
            }

            b']' => {
                if cells[ptr] != 0 {
                    pc = jump[pc];
                }
            }

            _ => {}
        }
        pc += 1;
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    let input = &args[1];
    let path = Path::new(input);

    if !path.exists() || !path.is_file() {
        eprintln!("Error: File '{}' does not exist or is not a file.", input);
        std::process::exit(1);
    }
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("b" | "bf" | "bfx") => {}

        _ => {
            eprintln!("Error: file must end with .b, .bf, or .bfx");
            std::process::exit(1);
        }
    }

    let source = std::fs::read_to_string(&input)?;
    bf(&source);
    Ok(())
}
