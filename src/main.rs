mod fileutil;
mod constants;
mod phone;
mod word;
mod viterbi;

use std::ffi::OsStr;
use constants::PRINT_PERCENT_COUNT;

fn get_rec_name(test_file_path: &OsStr) -> String {
    let test_file_path = test_file_path.to_str().expect("Can't parse test file path");
    let rec_name = str::replace(test_file_path, ".txt", ".rec");

    format!("\"{}\"", &rec_name)
}

fn run_all_tests() {
    let test_file_paths = fileutil::list_test_file_paths("tst");
    let mut recognized_file = fileutil::create_file("recognized.txt");

    let phones = phone::read_all("hmm.txt");
    let words = word::read_all("dictionary.txt", "unigram.txt", "bigram.txt");
    let transitions = viterbi::wire_transitions(&phones, &words);

    use std::io::Write;

    let _ = recognized_file.write("#!MLF!#\n".as_bytes());

    for (count, test_file_path) in test_file_paths.iter().enumerate() {
        if count % PRINT_PERCENT_COUNT == 0 {
            println!("{:.2}%..", count as f64 / test_file_paths.len() as f64 * 100f64);
        }

        let spectrogram = fileutil::read_spectrogram(test_file_path);
        let rec_name = get_rec_name(test_file_path);
        let _ = recognized_file.write_fmt(format_args!("{}\n", &rec_name));
        let word_seq = viterbi::run(&spectrogram, &phones, &words, &transitions);

        for word in word_seq.into_iter() {
            if word.name == "<s>" {
                continue;
            }
            let _ = recognized_file.write_fmt(format_args!("{}\n", &word.name));
        }
        let _ = recognized_file.write(".\n".as_bytes());
    }

    println!("100%");
}

fn main() {
    run_all_tests();
}
