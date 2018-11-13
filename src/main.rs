mod fileutil;
mod constants;
mod phone;
mod word;

fn main() {
    let phones = phone::read_all("hmm.txt");
    let words = word::read_all("dictionary.txt", "unigram.txt", "bigram.txt");
    println!("{:?}", words);
}
