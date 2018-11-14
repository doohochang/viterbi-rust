mod fileutil;
mod constants;
mod phone;
mod word;
mod viterbi;

fn main() {
    let phones = phone::read_all("hmm.txt");
    let words = word::read_all("dictionary.txt", "unigram.txt", "bigram.txt");
    let transitions = viterbi::wire_transitions(&phones, &words);
}
