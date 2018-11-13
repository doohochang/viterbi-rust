use fileutil;
use constants::*;
use phone::*;

#[derive(Debug)]
enum InputType {
    PhoneName(String),
    BeginHmm,
    EndHmm,
    NumStates(u32),
    CurrentState(u32),
    Mean(Vec<f64>),
    Var(Vec<f64>),
    TransProb(Vec<Vec<f64>>),
    NumMixes(u32),
    CurrentMixture(u32, f64),
}

use self::InputType::*;

fn split_whitespace(line: &String) -> Vec<&str> {
    line.split_whitespace()
        .collect()
}

fn get_float_values(line: &String) -> Vec<f64> {
    split_whitespace(line).into_iter()
        .map(|s| s.parse().expect("Float Value"))
        .collect()
}

// reads input from lines and return InputType
// it consumes read lines.
fn read_one_input(lines: &mut &[String]) -> InputType {
    let values = split_whitespace(&lines[0]);

    match values[0] {
        "~h" => {
            let phone_name = values[1][1..values[1].len() - 1].to_string();
            *lines = &lines[1..];
            PhoneName(phone_name)
        },
        "<BEGINHMM>" => {
            *lines = &lines[1..];
            BeginHmm
        },
        "<ENDHMM>" => {
            *lines = &lines[1..];
            EndHmm 
        },
        "<NUMSTATES>" => {
            let number: u32 = values[1].parse().expect("Num State");
            *lines = &lines[1..];
            NumStates(number)
        },
        "<STATE>" => {
            let number: u32 = values[1].parse().expect("State");
            *lines = &lines[1..];
            CurrentState(number)
        },
        "<MEAN>" => {
            let means = get_float_values(&lines[1]);
            *lines = &lines[2..];
            Mean(means)
        },
        "<VARIANCE>" => {
            let vars = get_float_values(&lines[1]);
            *lines = &lines[2..];
            Var(vars)
        },
        "<TRANSP>" => {
            let n: usize = values[1].parse().expect("TP Size");
            let mut tp: Vec<Vec<f64>> = Vec::new();
            for i in 1..=n {
                tp.push(get_float_values(&lines[i]));
            }
            *lines = &lines[n + 1..];
            TransProb(tp)
        },
        "<NUMMIXES>" => {
            let n: u32 = values[1].parse().expect("TP Size");
            *lines = &lines[1..];
            NumMixes(n)
        },
        "<MIXTURE>" => {
            let n: u32 = values[1].parse().expect("Mixture Index");
            let weight: f64 = values[2].parse().expect("Mixture Weight");
            *lines = &lines[1..];
            CurrentMixture(n, weight)
        },
        _ => panic!("Unexpected Input: {:?}", lines[0]),
    }
}

pub fn read_pdf(lines: &mut &[String]) -> Pdf {
    // read <MIXTURE>
    let weight: f64 = match read_one_input(lines) {
        CurrentMixture(_, weight) => weight,
        input @ _ => panic!("Unexpected HMM Input: {:?}", input)
    };

    // read <MEAN>, <VARIANCE>
    let mut mean = [0f64; N_DIMENSION];
    let mut var = [0f64; N_DIMENSION];
    for _ in 0..2 {
        match read_one_input(lines) {
            Mean(values) => for i in 0..N_DIMENSION {
                mean[i] = values[i];
            },
            Var(values) => for i in 0..N_DIMENSION {
                var[i] = values[i];
            },
            input @ _ => panic!("Unexpected HMM Input: {:?}", input)
        }
    }
    
    Pdf { weight, mean, var }
}

pub fn read_state(lines: &mut &[String]) -> State {
    read_one_input(lines); // read <STATE>
    
    // read <NUMMIXES>
    let n_mixes: u32 = match read_one_input(lines) {
        NumMixes(num) => num,
        input @ _ => panic!("Unexpected HMM Input: {:?}", input)
    };

    let mut pdfs = Vec::new();
    for _ in 0..n_mixes {
        pdfs.push(read_pdf(lines));
    }

    State { pdfs }
}

pub fn read_phone(lines: &mut &[String]) -> Phone {
    let mut name: String = String::new();
    let mut n_state: usize = 0;

    // read phone name, <NUMSTATES>, and <BEGINHMM>
    for _ in 0..3 {
        match read_one_input(lines) {
            PhoneName(_name) => name = _name,
            NumStates(num) => n_state = (num - 2) as usize,
            _ => ()
        }
    }

    // read states
    let mut states: Vec<State> = Vec::new();
    for _ in 0..n_state {
        states.push(read_state(lines));
    }

    // read transition probability matrix
    let mut in_prob = Vec::new();
    let mut trans_prob = Vec::new();
    let mut out_prob = Vec::new();
    match read_one_input(lines) {
        TransProb(_tp) => {
            in_prob = _tp[0][1..=n_state].to_vec();

            for i in 1..=n_state {
                trans_prob.push(_tp[i][1..=n_state].to_vec());
            }

            for i in 1..=n_state {
                out_prob.push(_tp[i][n_state + 1]);
            }
        },
        _ => ()
    }
    
    read_one_input(lines); // read <ENDHMM>

    Phone {
        name,
        states,
        in_prob, trans_prob, out_prob
    }
}

pub fn read_phones(hmm_file_path: &str) -> Vec<Phone> {
    let all_lines = fileutil::read_lines(hmm_file_path);

    let mut phones: Vec<Phone> = Vec::new();

    let mut remaining_lines = &all_lines[..];

    while remaining_lines.len() > 0 {
        phones.push(read_phone(&mut remaining_lines));
    }

    phones
}