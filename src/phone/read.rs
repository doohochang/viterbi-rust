use fileutil;
use phone::*;

#[derive(Debug)]
enum InputType {
    PhoneName(String),
    BeginHmm,
    EndHmm,
    NumStates(u32),
    TransProb(Vec<Vec<f64>>),
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
        "<TRANSP>" => {
            let n: usize = values[1].parse().expect("TP Size");
            let mut tp: Vec<Vec<f64>> = Vec::new();
            for i in 1..=n {
                tp.push(get_float_values(&lines[i]));
            }
            *lines = &lines[n + 1..];
            TransProb(tp)
        },
        _ => panic!("Unexpected Input: {:?}", lines[0]),
    }
}

pub fn read_phone(lines: &mut &[String], index: usize) -> Phone {
    let mut name: String = String::new();
    let mut n_states: usize = 0;

    // read phone name, <NUMSTATES>, and <BEGINHMM>
    for _ in 0..3 {
        match read_one_input(lines) {
            PhoneName(_name) => name = _name,
            NumStates(num) => n_states = (num - 2) as usize,
            _ => ()
        }
    }

    // read transition probability matrix
    let mut in_prob = Vec::new();
    let mut trans_prob = Vec::new();
    let mut out_prob = Vec::new();
    let mut skip_prob = 0f64;
    match read_one_input(lines) {
        TransProb(_tp) => {
            in_prob = _tp[0][1..=n_states].to_vec();

            for i in 1..=n_states {
                trans_prob.push(_tp[i][1..=n_states].to_vec());
            }

            for i in 1..=n_states {
                out_prob.push(_tp[i][n_states + 1]);
            }

            skip_prob = _tp[0][n_states + 1];
        },
        _ => ()
    }
    
    read_one_input(lines); // read <ENDHMM>

    Phone {
        index,
        name, n_states,
        in_prob, trans_prob, out_prob, skip_prob
    }
}

pub fn read_phones(hmm_file_path: &str) -> Vec<Phone> {
    use std::ffi::OsStr;
    let all_lines = fileutil::read_lines(OsStr::new(hmm_file_path));

    let mut phones: Vec<Phone> = Vec::new();

    let mut remaining_lines = &all_lines[..];

    let mut index = 0;
    while remaining_lines.len() > 0 {
        phones.push(read_phone(&mut remaining_lines, index));
        index = index + 1;
    }

    phones
}