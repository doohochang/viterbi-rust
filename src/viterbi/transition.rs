use constants::*;
use word::Word;
use viterbi::StateRef;

#[derive(Debug)]
pub struct Transition {
    pub log_prob: f64,
    pub dest: StateRef,
    pub to_next_word: bool,
}

#[derive(Debug)]
pub struct Transitions {
    pub from_start: Vec<Transition>,
    pub from_state: Vec<Vec<Vec<Vec<Transition>>>>, // from_state[word][phone][state] has own transitions
}

pub fn wire(words: &[Word]) -> Transitions {
    let mut from_start: Vec<Transition> = Vec::new();
    for (w, word) in words.iter().enumerate() {
        let phone = word.phones[0];
        for s in 0..phone.n_states {
            let prob = word.head_prob * phone.in_prob[s];
            if prob > 0f64 {
                from_start.push(
                    Transition {
                        log_prob: prob.ln(),
                        dest: StateRef {
                            word: w,
                            phone: 0,
                            state: s,
                        },
                        to_next_word: false
                    }
                )
            }
        }
    }

    // initialize from_state
    let mut from_state: Vec<Vec<Vec<Vec<Transition>>>> = Vec::new();
    for (w, word) in words.iter().enumerate() {
        from_state.push(Vec::with_capacity(word.phones.len()));
        for (p, phone) in word.phones.iter().enumerate() {
            from_state[w].push(Vec::with_capacity(phone.n_states));
            for _ in 0..phone.n_states {
                from_state[w][p].push(Vec::new());
            }
        }
    }

    for (w, word) in words.iter().enumerate() {
        for (p, phone) in word.phones.iter().enumerate() {
            // transitions in each phone's hmm
            for s in 0..phone.n_states {
                for d in 0..phone.n_states {
                    let prob = phone.trans_prob[s][d];
                    if prob > 0f64 {
                        from_state[w][p][s].push(
                            Transition {
                                log_prob: prob.ln(),
                                dest: StateRef {
                                    word: w,
                                    phone: p,
                                    state: d,
                                },
                                to_next_word: false
                            }
                        )
                    }
                }
            }

            if p < word.phones.len() - 1 {
                // transitions between current phone & next phone
                let next_phone = word.phones[p+1];
                for s in 0..phone.n_states {
                    for d in 0..next_phone.n_states {
                        let prob = phone.out_prob[s] * next_phone.in_prob[d];
                        if prob > 0f64 {
                            from_state[w][p][s].push(
                                Transition {
                                    log_prob: prob.ln(),
                                    dest: StateRef { 
                                        word: w,
                                        phone: p + 1,
                                        state: d,
                                    },
                                    to_next_word: false,
                                }
                            )
                        }
                    }
                }
            }
        }
    }

    // transitions to next word
    for (w, word) in words.iter().enumerate() {
        let p = word.phones.len() - 1;
        let phone = word.phones[p];
        let is_phone_sp = phone.name == "sp";
        for (next_w, next_word) in words.iter().enumerate() {
            let next_phone = next_word.phones[0];
            for d in 0..next_phone.n_states {
                for s in 0..phone.n_states {
                    let prob = phone.out_prob[s] * word.next_word_prob[next_w] * next_phone.in_prob[d];
                    if prob > 0f64 {
                        from_state[w][p][s].push(
                            Transition {
                                log_prob: prob.ln() - WORD_PENALTY,
                                dest: StateRef {
                                    word: next_w,
                                    phone: 0,
                                    state: d,
                                },
                                to_next_word: true,
                            }
                        )
                    }
                }

                if is_phone_sp && p > 0 {
                    // if the phone is "sp", then we can skip it
                    let prev_phone = word.phones[p - 1];
                    for s in 0..prev_phone.n_states {
                        let prob = prev_phone.out_prob[s] * phone.skip_prob * word.next_word_prob[next_w] * next_phone.in_prob[d];
                        if prob > 0f64 {
                            from_state[w][p - 1][s].push(
                                Transition {
                                    log_prob: prob.ln() - WORD_PENALTY,
                                    dest: StateRef {
                                        word: next_w,
                                        phone: 0,
                                        state: d,
                                    },
                                    to_next_word: true,
                                }
                            )
                        }
                    }
                }
            }
        }
    }

    Transitions {
        from_start,
        from_state
    }
}