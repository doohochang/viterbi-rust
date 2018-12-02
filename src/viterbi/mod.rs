mod transition;

use constants::*;
use phone::Phone;
use word::Word;
use dnn::Dnn;
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

pub fn run<'w>(
    spectrogram: &[[f64; N_DIMENSION]],
    phones: &[Phone],
    words: &'w [Word<'w>],
    transitions: &Transitions,
    dnn: &mut Dnn,
) -> Vec<&'w Word<'w>> {
    let mut table = init_table(spectrogram.len(), words);

    for t in transitions.from_start.iter() {
        let dest_value = &mut table[0][t.dest.word][t.dest.phone][t.dest.state];
        consider_and_apply(
            Value { log_prob: t.log_prob, prev: None, word_changed: false },
            dest_value
        )
    }

    for t in 0..spectrogram.len()-1 {
        let spectrum_window = make_spectrum_window(spectrogram, t + 1, dnn.spectrum_window_range);
        let observation_prob = dnn.compute_observation_prob(&spectrum_window, phones);

        for (w, word) in words.iter().enumerate() {
            for (p, phone) in word.phones.iter().enumerate() {
                for s in 0..phone.n_states {
                    let table_value = table[t][w][p][s];
                    match table_value {
                        Some(prev_value) => {
                            for tr in transitions.from_state[w][p][s].iter() {
                                let next_p_index = words[tr.dest.word].phones[tr.dest.phone].index;
                                let log_prob = prev_value.log_prob + tr.log_prob + (observation_prob[next_p_index][tr.dest.state] as f64).ln();
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

fn make_spectrum_window(spectrogram: &[[f64; N_DIMENSION]], index: usize, range: (i32, i32)) -> Vec<f32> {
    let (start, end) = range;
    let mut spectrum_window = Vec::new();
    for delta in start..end {
        let t = index as i32 + delta;
        let spectrum =
            if t < 0 {
                &spectrogram[0]
            }
            else if t >= spectrogram.len() as i32 {
                &spectrogram[spectrogram.len() - 1]
            } else {
                &spectrogram[t as usize]
            };

        for value in spectrum.iter() {
            spectrum_window.push(*value as f32);
        }
    }

    spectrum_window
}

// reset and resize multi-demensional vec values
fn init_table(time_length: usize, words: &[Word]) -> Vec<Vec<Vec<Vec<Option<Value>>>>> {
    let mut table = Vec::with_capacity(time_length);
    for t in 0..time_length {
        table.push(Vec::with_capacity(words.len()));
        for (w, word) in words.iter().enumerate() {
            table[t].push(Vec::with_capacity(word.phones.len()));
            for (p, phone) in word.phones.iter().enumerate() {
                table[t][w].push(Vec::with_capacity(phone.n_states));
                for _ in 0..phone.n_states {
                    table[t][w][p].push(None);
                }
            }
        }
    }
    table
}