use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct Label {
    name: String,
    byte: u16,
}

pub fn run(path: PathBuf) {
    let file = fs::read_to_string(path).expect("Failed to read file");
    let labels = parse_labels(file.to_owned());
}

fn parse_labels(file: String) -> Vec<Label> {
    let mut line_number = 0;
    let mut labels: Vec<Label> = Vec::new();

    for line in file.lines().map(|s| s.trim()) {
        if line.starts_with("//") || line.is_empty() {
            continue
        }

        if line.starts_with(".") {
            let label = Label {
                name: line[1..].to_string(),
                byte: line_number*4,
            };
            println!("{label:?}");
            labels.push(label);
            continue;
        }

        line_number += 1;
    }

    labels
}
