use std::path::Path;
use std::fs::File;
use std::io::Read;

pub fn read_lines(file_path: &str) -> Vec<String> {
    let path = Path::new(file_path);
    let file = File::open(path).expect("Can't open file");
    read_lines_from_file(file)
}

fn read_lines_from_file(mut file: File) -> Vec<String> {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).expect("Can't read");

    buffer.lines()
        .map(|s: &str| String::from(s))
        .collect()
}
