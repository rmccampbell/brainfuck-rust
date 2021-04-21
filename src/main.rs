use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    /// A brainfuck source file or - for stdin
    #[structopt(parse(from_os_str), default_value = "-")]
    file: PathBuf,
    /// An inline brainfuck program
    #[structopt(short, long, conflicts_with = "file")]
    command: Option<String>,
    /// Enable debug prints
    #[structopt(short, long)]
    debug: bool,
}

#[derive(Debug, Copy, Clone)]
enum BfOp {
    Gt,
    Lt,
    Plus,
    Minus,
    Dot,
    Comma,
    LBracket(usize),
    RBracket(usize),
}

#[derive(Debug, Copy, Clone)]
enum ParseError {
    UnmatchedLeftBracket,
    UnmatchedRightBracket,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid brainfuck syntax: {:?}", self)
    }
}

impl Error for ParseError {}

use BfOp::*;
use ParseError::*;

fn parse(code: &str) -> Result<Vec<BfOp>, ParseError> {
    let mut instrs: Vec<_> = code
        .bytes()
        .filter_map(|c| match c {
            b'>' => Some(Gt),
            b'<' => Some(Lt),
            b'+' => Some(Plus),
            b'-' => Some(Minus),
            b'.' => Some(Dot),
            b',' => Some(Comma),
            b'[' => Some(LBracket(0)),
            b']' => Some(RBracket(0)),
            _ => None,
        })
        .collect();
    let mut brackets = Vec::new();
    for i in 0..instrs.len() {
        match instrs[i] {
            LBracket(_) => brackets.push(i),
            RBracket(_) => {
                let j = brackets.pop().ok_or(UnmatchedRightBracket)?;
                instrs[j] = LBracket(i);
                instrs[i] = RBracket(j);
            }
            _ => (),
        }
    }
    if !brackets.is_empty() {
        return Err(UnmatchedLeftBracket);
    }
    Ok(instrs)
}

fn run(code: &str, opts: &Opt) -> Result<(), Box<dyn Error>> {
    let instrs = parse(code)?;
    if opts.debug {
        println!("{:?}", instrs);
    }
    let stdout = io::stdout();
    let stdin = io::stdin();
    let mut stdout = stdout.lock();
    let mut stdin = stdin.lock();
    let mut tape = [0u8; 1 << 16];
    let mut pc = 0;
    let mut ptr = 0;
    while pc < instrs.len() {
        match instrs[pc] {
            Gt => ptr += 1,
            Lt => ptr -= 1,
            Plus => tape[ptr] = tape[ptr].wrapping_add(1),
            Minus => tape[ptr] = tape[ptr].wrapping_sub(1),
            Dot => {
                stdout.write(&tape[ptr..=ptr])?;
            }
            Comma => {
                tape[ptr] = 0;
                stdin.read(&mut tape[ptr..=ptr])?;
            }
            LBracket(i) => {
                if tape[ptr] == 0 {
                    pc = i
                }
            }
            RBracket(i) => {
                if tape[ptr] != 0 {
                    pc = i
                }
            }
        }
        pc += 1;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let mut code = String::new();
    if let Some(cmd) = &opt.command {
        code = cmd.clone();
    } else {
        let mut reader: Box<dyn Read> = match &opt.file {
            p if p == Path::new("-") => Box::new(io::stdin()),
            path => Box::new(File::open(path)?),
        };
        reader.read_to_string(&mut code)?;
    }
    run(&code, &opt)?;
    Ok(())
}

// fn main() {
//     main_().unwrap_or_else(|e| eprintln!("Error: {}", e));
// }
