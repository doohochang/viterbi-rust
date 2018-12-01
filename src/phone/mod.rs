use std::fmt;
use constants::*;

pub mod read;
pub use self::read::read_phones as read_all;

#[derive(Debug)]
pub struct Phone {
    pub index: usize,
    pub name: String,
    pub states: Vec<State>,
    pub in_prob: Vec<f64>,
    pub trans_prob: Vec<Vec<f64>>,
    pub out_prob: Vec<f64>,
    pub skip_prob: f64, // transition prob between entry and exit
}

#[derive(Debug)]
pub struct State {
    pub pdfs: Vec<Pdf>,
}

pub struct Pdf {
    pub weight: f64,
    pub mean: [f64; N_DIMENSION],
    pub var: [f64; N_DIMENSION],
}

impl fmt::Debug for Pdf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pdf {{ weight: {}, mean: {:?}.., var: {:?}.. }}", self.weight, &self.mean[0..6], &self.var[0..6])
    }
}

pub fn find<'a>(name: &str, phones: &'a [Phone]) -> &'a Phone {
    let find_result = phones.iter()
        .find(|phone| phone.name == name);

    match find_result {
        Some(phone) => phone,
        None => panic!("No phone name: {}", name)
    }
}
