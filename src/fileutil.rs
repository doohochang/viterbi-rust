use std::path::Path;
use std::fs::{self, File};
use std::io::{self, Read};
use std::ffi::OsString;

pub fn read_lines(file_path: &str) -> Vec<String> {
    let path = Path::new(file_path);
    let file = File::open(path).expect("Can't open file");
    read_lines_from_file(file)
}

pub fn list_test_files(directory_path: &str) -> Vec<OsString> {
    list_files(Path::new(directory_path)).expect("Can't list test files")
}

fn read_lines_from_file(mut file: File) -> Vec<String> {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).expect("Can't read");

    buffer.lines()
        .map(|s: &str| String::from(s))
        .collect()
}

fn list_files(path: &Path) -> io::Result<Vec<OsString>> {
    let mut files = Vec::new();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let child_path = entry?.path();
            if child_path.is_dir() {
                let child_files = list_files(&child_path)?;
                files.extend(child_files);
            } else {
                files.push(child_path.as_os_str().to_os_string());
            }
        }

        Ok(files)
    } else {
        files.push(path.as_os_str().to_os_string());
        Ok(files)
    }
}
