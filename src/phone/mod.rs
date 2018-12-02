pub mod read;

pub use self::read::read_phones as read_all;

#[derive(Debug)]
pub struct Phone {
    pub index: usize,
    pub name: String,
    pub n_states: usize,
    pub in_prob: Vec<f64>,
    pub trans_prob: Vec<Vec<f64>>,
    pub out_prob: Vec<f64>,
    pub skip_prob: f64, // transition prob between entry and exit
}

pub fn find<'a>(name: &str, phones: &'a [Phone]) -> &'a Phone {
    let find_result = phones.iter()
        .find(|phone| phone.name == name);

    match find_result {
        Some(phone) => phone,
        None => panic!("No phone name: {}", name)
    }
}
