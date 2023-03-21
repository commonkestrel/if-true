use std::{fs, path::PathBuf};
use super::{IO_ERROR, COMPILE_ERROR};

const INSRUCTIONS: [&'static str; 25] = ["ldr", "lda", "ldi", "st", "add", "adc", "sub", "or", "and", "xor", "not", "lsl", "lsr", "cmp", "breq", "brne", "brgt", "brlt", "brge", "brle", "jmp", "out", "in", "nop", "halt"];

#[derive(Debug, Clone)]
struct Macro {
    name: String,
    arguments: Vec<String>,
    lines: Vec<String>,
}

#[derive(Debug, Clone)]
struct Label {
    name: String,
    byte: u16,
}

pub fn run(path: String) {
    let mut file = fs::read_to_string(path.to_owned()).expect("Failed to read file");
    let labels = parse_labels(file.to_owned());
    file = expand_macros(file, path.to_owned());
    println!("{file}");

    let mut file_path = PathBuf::from(path);
    file_path.set_extension("o");

    fs::write(file_path, file).expect("Unable to write to output file");
}

fn parse_labels(file: String) -> Vec<Label> {
    let mut line_number = 0;
    let mut labels: Vec<Label> = Vec::new();

    for line in file.lines().map(|s| s.trim().to_lowercase()) {
        if line.starts_with("//") || line.starts_with("end") || line.is_empty() {
            continue;
        }

        if line.starts_with(".") {
            let label = Label {
                name: line[1..].to_string(),
                byte: line_number*4,
            };
            labels.push(label);
            continue;
        }

        line_number += 1;
    }

    labels
}

fn parse_macros(file: String, path: String) -> (Vec<Macro>, String) {
    let mut line_number: usize = 1;
    let mut cutout = String::new();
    let mut macros = Vec::new();
    let mut names = Vec::new();
    let mut current: Option<(Macro, usize, String)> = None;

    for line in file.lines().map(|s| s.trim().to_lowercase()) {
        if line.starts_with("@") {
            if let Some(macro_line) = current {
                println!("error: macro inside another macro
\n\
                        --> {path}:{line_number} \n\
                        {0:1$}| \n\
                        {2:<1$}| {3} \n\
                        {0:1$}| ^ first macro defined here \n\
                        {line_number:<1$}| {line} \n\
                        {0:1$}| \n", "",
                    line_number.to_string().len().max(macro_line.1.to_string().len()),
                    macro_line.1, macro_line.2,
                );
                std::process::exit(COMPILE_ERROR);
            }

            let (name, arguments): (String, Vec<String>) = match line[1..].split_once(" ") {
                Some((n, a)) => (n.to_owned(), a.split(",").map(|s| s.trim().to_owned()).collect()),
                None => (line[1..].to_owned(), Vec::new()),
            };

            if INSRUCTIONS.contains(&name.as_str()) {
                println!("error: macros cannot share a name with a cpu instruction \n\
                        \n\
                        {0:>1$} \n\
                        {line_number}| {line} \n\
                        {0:>1$} ^ attempted redeclaration of the '{name}' instruction\n", "|", line_number.to_string().len()+1
                );
                std::process::exit(COMPILE_ERROR);
            }

            if names.contains(&name) {
                println!("error: macro '{name}' already declared
                        \n\
                        {0:>1$} \n\
                        {line_number}| {line} \n\
                        {0:>1$} ^ macro '{name}' redeclared here \n", "|", line_number.to_string().len()+1
                );
                std::process::exit(COMPILE_ERROR);
            }

            for arg in arguments.iter() {
                if !arg.starts_with("%") {
                    let position = line.find(arg).unwrap();
                    println!("error: macro arguments must begin with '%' \n\
                            \n\
                            --> {path}:{line_number} \n\
                            {0:>2$} \n\
                            {line_number}| {line} \n\
                            {0:>2$} {1:position$}^ argument declared here \n", "|", "", line_number.to_string().len()+1,
                    );
                    std::process::exit(COMPILE_ERROR);
                }
            }

            names.push(name.to_owned());
            current = Some((
                Macro {
                    name,
                    arguments,
                    lines: Vec::new(),
                },
                line_number, 
                line.to_owned(),
            ));
        } else if line == "end" {
            if let Some((asm_macro, _, _)) = current {
                macros.push(asm_macro);
                current = None;
            } else {
                println!(
                        "error: 'end' keyword with no macro to close \n\
                        \n\
                        --> {path}:{line_number} \n\
                        {line_number}| {line} \n\
                        {:width$}  ^^^\n",
                    "",
                    width = line_number.to_string().len()
                );
                std::process::exit(COMPILE_ERROR);
            }
        } else if !line.starts_with("//") && !line.starts_with(".") {
            if let Some(ref mut cur) = current {
                cur.0.lines.push(line.to_owned());
            }
        }

        if current.is_none() && line != "end" {
            if !cutout.is_empty() {
                cutout += "\n";
            }
            cutout += &line;
        }

        line_number += 1;
    }

    if let Some(cur) = current {
        println!(
"error: macro '{}' not ended

--> {path}:{1}
{1}| {2}
  {3:4$}^ macro defined here
",
            cur.0.name,
            cur.1,
            cur.2,
            "",
            cur.1.to_string().len()
        )
    }

    (macros, cutout)
}

fn expand_macros(file: String, path: String) -> String {
    let (macros, file) = parse_macros(file.to_owned(), path.to_owned());
    let mut expanded = String::new();
    let mut line_number = 1;

    for line in file.lines().map(|s| s.trim().to_lowercase()) {
        if line.starts_with("//") || line.starts_with(".") || line.starts_with("@") || (line == "end") || line.is_empty() {
            line_number += 1;
            continue;
        }

        let (instruction, args) = match line.split_once(" ") {
            Some((name, args)) => (name.to_owned(), args.to_owned()),
            None => (line.to_owned(), String::new()),
        };

        if INSRUCTIONS.contains(&instruction.as_str()) {
            if !expanded.is_empty() {
                expanded += "\n";
            }
            expanded += &line;
            continue;
        }

        let arguments: Vec<String> = args.split(",").map(|s| s.trim().to_owned()).collect();

        for asm_macro in macros.iter() {
            if asm_macro.name == instruction {
                if asm_macro.arguments.len() != arguments.len() {
                    println!("error: incorrect number of arguments
                            \n\
                            --> {path}:{line_number} \n\
                            {0:>1$} \n\
                            {line_number}| {line} \n\
                            {0:>1$} expected {2} arguments, found {3} \n\
                            {0:>1$}\n", "|", line_number.to_string().len()+1, asm_macro.arguments.len(), arguments.len()
                    );
                    std::process::exit(COMPILE_ERROR);
                }

                let mut replaced = asm_macro.lines.join("\n");
                for i in 0..arguments.len() {
                    replaced = replaced.replace(&asm_macro.arguments[i], &arguments[i]);
                }
                if !expanded.is_empty() {
                    expanded += "\n";
                }
                expanded += &replaced;

                break;
            } 
        }

        line_number += 1;
    }

    expanded
}
