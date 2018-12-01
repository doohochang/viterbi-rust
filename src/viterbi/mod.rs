mod observation;
mod transition;

use constants::*;
use phone::{self, Phone};
use word::Word;
pub use self::transition::{Transitions, wire as wire_transitions};

#[derive(Clone, Copy, Debug)]
pub struct StateRef {
    pub word: usize,
    pub phone: usize,
    pub state: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Value {
    log_prob: f64,
    prev: Option<StateRef>,
    word_changed: bool,
}

fn consider_and_apply(new_value: Value, old_value: &mut Option<Value>) {
    let is_new_value_better = 
        if let Some(value) = old_value {
            value.log_prob < new_value.log_prob
        } else {
            true
        };

    if is_new_value_better {
        *old_value = Some(new_value);
    }
}

pub fn run<'w>(spectrogram: &[[f64; N_DIMENSION]], phones: &[Phone], words: &'w [Word], transitions: &Transitions) -> Vec<&'w Word> {
    let mut table = init_table(spectrogram.len(), phones, words);
    let phone_index = compute_phone_index(phones, words);

    for t in transitions.from_start.iter() {
        let dest_value = &mut table[0][t.dest.word][t.dest.phone][t.dest.state];
        consider_and_apply(
            Value { log_prob: t.log_prob, prev: None, word_changed: false },
            dest_value
        )
    }

    for (t, spectrum) in spectrogram[1..].iter().enumerate() {
        let observation_prob = compute_observation_prob(spectrum, phones);
        for (w, word) in words.iter().enumerate() {
            for p in 0..word.phones.len() {
                let p_index = phone_index[w][p];
                let n_state = phones[p_index].states.len();
                for s in 0..n_state {
                    let table_value = table[t][w][p][s];
                    match table_value {
                        Some(prev_value) => {
                            for tr in transitions.from_state[w][p][s].iter() {
                                let next_p_index = phone_index[tr.dest.word][tr.dest.phone];
                                let log_prob = prev_value.log_prob + tr.log_prob + observation_prob[next_p_index][tr.dest.state];
                                consider_and_apply(
                                    Value { 
                                        log_prob,
                                        prev: Some(StateRef { word: w, phone: p, state: s }),
                                        word_changed: tr.to_next_word
                                    },
                                    &mut table[t+1][tr.dest.word][tr.dest.phone][tr.dest.state]
                                )
                            }
                        },
                        None => ()
                    }
                }
            }
        }
    }
    let max_ref = get_max(&table[spectrogram.len() - 1]);
    let word_index_seq = backtrace(spectrogram.len() - 1, max_ref.word, max_ref.phone, max_ref.state, &table);

    word_index_seq.into_iter()
        .map(|index| &words[index])
        .collect()
}

// backtrace and return word index sequence
fn backtrace(time: usize, word: usize, phone: usize, state: usize, table: &Vec<Vec<Vec<Vec<Option<Value>>>>>) -> Vec<usize> {
    let value = &table[time][word][phone][state].expect("No Table Value");

    match value.prev {
        Some(StateRef { word: prev_word, phone: prev_phone, state: prev_state }) => {
            let mut seq = backtrace(time - 1, prev_word, prev_phone, prev_state, table);
            if value.word_changed {
                seq.push(prev_word);
            }
            seq
        },
        None => Vec::new()
    }
}

fn get_max(last_values: &Vec<Vec<Vec<Option<Value>>>>) -> StateRef {
    let mut max = None;
    for w in 0..last_values.len() {
        for p in 0..last_values[w].len() {
            for s in 0..last_values[w][p].len() {
                match &last_values[w][p][s] {
                    Some(value) => {
                        consider_and_apply(
                            Value {
                                log_prob: value.log_prob,
                                prev: Some(StateRef { word: w, phone: p, state: s }),
                                word_changed: false,
                            },
                            &mut max,
                        )
                    },
                    None => ()
                }
            }
        }
    }

    max.expect("Max Value").prev.expect("Max StateRef")
}

// pre-compute observation probabilities
fn compute_observation_prob(spectrum: &[f64; N_DIMENSION], phones: &[Phone]) -> Vec<Vec<f64>> {
    let mut prob = vec![Vec::new(); phones.len()];
    for (p, _phone) in phones.iter().enumerate() {
        for state in _phone.states.iter() {
            prob[p].push(observation::prob(spectrum, state));
        }
    }
    prob
}

fn compute_phone_index(phones: &[Phone], words: &[Word]) -> Vec<Vec<usize>> {
    let mut phone_index = Vec::with_capacity(words.len());
    for (w, word) in words.iter().enumerate() {
        phone_index.push(Vec::with_capacity(word.phones.len()));
        for phone_name in word.phones.iter() {
            phone_index[w].push(phone::find_index(phone_name, phones));
        }
    }
    phone_index
}

// reset and resize multi-demensional vec values
fn init_table(time_length: usize, phones: &[Phone], words: &[Word]) -> Vec<Vec<Vec<Vec<Option<Value>>>>> {
    let mut table = Vec::with_capacity(time_length);
    for t in 0..time_length {
        table.push(Vec::with_capacity(words.len()));
        for (w, word) in words.iter().enumerate() {
            table[t].push(Vec::with_capacity(word.phones.len()));
            for p in 0..word.phones.len() {
                let _phone = phone::find(&word.phones[p], phones);
                table[t][w].push(Vec::with_capacity(_phone.states.len()));
                for _ in 0.._phone.states.len() {
                    table[t][w][p].push(None);
                }
            }
        }
    }
    table
}