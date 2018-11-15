use std::ffi::OsStr;

use fileutil;
use word::*;

pub fn read_words(dictionary_path: &str, unigram_path: &str, bigram_path: &str) -> Vec<Word> {
    let mut words = read_dictionary(dictionary_path);
    read_head_probs(unigram_path, &mut words);
    read_next_word_probs(bigram_path, &mut words);

    words
}

fn read_dictionary(path: &str) -> Vec<Word> {
    let lines = fileutil::read_lines(OsStr::new(path));

    let mut words = Vec::new();
    for line in lines.iter() {
        let elements: Vec<&str> = line.split_whitespace().collect();

        if elements.len() == 0 {
            continue;
        }

        let name = elements[0].to_string();
        let phones: Vec<String> = (&elements[1..]).iter()
            .map(|s| s.to_string())
            .collect();

        words.push(
            Word { name, phones, head_prob: 0f64, next_word_prob: Vec::new() }
        );
    }

    let n_words = words.len();
    // set next_word_prob to zeros of n_words.
    for word in words.iter_mut() {
        word.next_word_prob.resize(n_words, 0f64);
    }

    words
}

fn read_head_probs(unigram_path: &str, words: &mut Vec<Word>) {
    let lines = fileutil::read_lines(OsStr::new(unigram_path));

    for line in lines.iter() {
        let mut elements = line.split_whitespace();

        match (elements.next(), elements.next()) {
            (Some(word_name), Some(prob_str)) => {
                let prob: f64 = prob_str.parse().expect("Word Head Prob");
                for word in words.iter_mut() {
                    if word.name == word_name {
                        word.head_prob = prob;
                    }
                }
            },
            _ => panic!("Invalid Unigram Input: {:?}", line)
        }
    }
}

fn read_next_word_probs(bigram_path: &str, words: &mut Vec<Word>) {
    let lines = fileutil::read_lines(OsStr::new(bigram_path));

    for line in lines.iter() {
        let mut elements = line.split_whitespace();

        match (elements.next(), elements.next(), elements.next()) {
            (Some(first), Some(second), Some(prob_str)) => {
                let prob: f64 = prob_str.parse().expect("Word Transition Prob");
                for source in 0..words.len() {
                    if words[source].name != first {
                        continue;
                    }
                    for dest in 0..words.len() {
                        if words[dest].name == second {
                            words[source].next_word_prob[dest] = prob;
                        }
                    }
                }
            },
            _ => panic!("Invalid Bigram Input: {:?}", line)
        }
    }
}
