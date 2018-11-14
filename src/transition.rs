use constants::*;
use word::Word;
use phone::Phone;

#[derive(Debug)]
pub struct Dest {
    word: usize,
    phone: usize,
    state: usize,
    is_next_word: bool,
}

#[derive(Debug)]
pub struct Transition {
    dest: Dest,
    log_prob: f64,
}

#[derive(Debug)]
pub struct Transitions {
    from_start: Vec<Transition>,
    from_state: Vec<Vec<Vec<Vec<Transition>>>>,
}

fn find_phone<'a>(name: &str, phones: &'a [Phone]) -> &'a Phone {
    let find_result = phones.iter()
        .find(|phone| phone.name == name);

    match find_result {
        Some(phone) => phone,
        None => panic!("No phone name: {}", name)
    }
}

pub fn wire(phones: &[Phone], words: &[Word]) -> Transitions {
    let mut from_start: Vec<Transition> = Vec::new();
    for w in 0..words.len() {
        let word = &words[w];
        let phone = find_phone(&word.phones[0], phones);
        for s in 0..phone.states.len() {
            let prob = word.head_prob * phone.in_prob[s];
            if prob > 0f64 {
                from_start.push(
                    Transition {
                        dest: Dest {
                            word: w,
                            phone: 0,
                            state: s,
                            is_next_word: true
                        },
                        log_prob: prob.ln() - WORD_PENALTY
                    }
                )
            }
        }
    }

    // initialize from_state
    let mut from_state: Vec<Vec<Vec<Vec<Transition>>>> = Vec::new();
    for w in 0..words.len() {
        let word = &words[w];
        from_state.push(Vec::new());
        for p in 0..words[w].phones.len() {
            let phone = find_phone(&word.phones[p], phones);
            from_state[w].push(Vec::new());
            for _ in 0..phone.states.len() {
                from_state[w][p].push(Vec::new());
            }
        }
    }

    for w in 0..words.len() {
        let word = &words[w];
        for p in 0..word.phones.len() {
            let phone = find_phone(&word.phones[p], phones);

            // transitions in each phone's hmm
            for s in 0..phone.states.len() {
                for d in 0..phone.states.len() {
                    let prob = phone.trans_prob[s][d];
                    if prob > 0f64 {
                        from_state[w][p][s].push(
                            Transition {
                                dest: Dest {
                                    word: w,
                                    phone: p,
                                    state: d,
                                    is_next_word: false
                                },
                                log_prob: prob.ln()
                            }
                        )
                    }
                }
            }

            if p < word.phones.len() - 1 {
                // transitions between current phone & next phone
                let next_phone = find_phone(&word.phones[p+1], phones);
                for s in 0..phone.states.len() {
                    for d in 0..next_phone.states.len() {
                        let prob = phone.out_prob[s] * next_phone.in_prob[d];
                        if prob > 0f64 {
                            from_state[w][p][s].push(
                                Transition {
                                    dest: Dest { 
                                        word: w,
                                        phone: p + 1,
                                        state: d,
                                        is_next_word: false
                                    },
                                    log_prob: prob.ln()
                                }
                            )
                        }
                    }
                }
            }
        }
    }

    // transitions to next word
    for w in 0..words.len() {
        let word = &words[w];
        let p = word.phones.len() - 1;
        let phone = find_phone(&word.phones[p], phones);
        let is_phone_sp = phone.name == "sp";
        for next_w in 0..words.len() {
            let next_word = &words[next_w];
            let next_phone = find_phone(&next_word.phones[0], phones);
            for d in 0..next_phone.states.len() {
                for s in 0..phone.states.len() {
                    let prob = phone.out_prob[s] * word.next_word_prob[next_w] * next_phone.in_prob[d];
                    if prob > 0f64 {
                        from_state[w][p][s].push(
                            Transition {
                                dest: Dest {
                                    word: next_w,
                                    phone: 0,
                                    state: d,
                                    is_next_word: true
                                },
                                log_prob: prob.ln() - WORD_PENALTY
                            }
                        )
                    }
                }

                if is_phone_sp && p > 0 {
                    // if the phone is "sp", then we can skip it
                    let prev_phone = find_phone(&word.phones[p - 1], phones);
                    for s in 0..prev_phone.states.len() {
                        let prob = prev_phone.out_prob[s] * phone.skip_prob * word.next_word_prob[next_w] * next_phone.in_prob[d];
                        if prob > 0f64 {
                            from_state[w][p - 1][s].push(
                                Transition {
                                    dest: Dest {
                                        word: next_w,
                                        phone: 0,
                                        state: d,
                                        is_next_word: true
                                    },
                                    log_prob: prob.ln() - WORD_PENALTY
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