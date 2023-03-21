mod assembler;

use std::env;

pub const IO_ERROR: i32 = 1;
pub const COMPILE_ERROR: i32 = 2;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("\nExpected more arguments!\nCommand should be called like this:\n\tif-true <command> <input file>\n\nPossible commands are:\n\tassemble: assembles an input .asm file to machine code\n\temulate: emulates an input if-true .o file\n");
        std::process::exit(1);
    }

    let command = args[1].to_owned();
    let file_name = args[2].to_owned();
    match command.as_str() {
        "assemble" => {
            assembler::run(file_name);
        },
        "emulate" => {
            println!("emulate");
        },
        _ => {
            println!("\n{command} not recognized as a command\n\nPossible commands are:\n\tassemble: assembles an input .asm file to machine code\n\temulate: emulates an input if-true .o file\n");
        },
    }
}
