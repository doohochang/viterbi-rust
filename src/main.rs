mod fileutil;
mod constants;
mod phone;
mod word;
mod observation;
mod transition;

fn main() {
    let phones = phone::read_all("hmm.txt");
    let words = word::read_all("dictionary.txt", "unigram.txt", "bigram.txt");
    let transitions = transition::wire(&phones, &words);
}
