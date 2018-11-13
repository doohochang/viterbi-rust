pub mod read;

pub use self::read::read_words as read_all;

#[derive(Debug)]
pub struct Word {
    pub name: String,
    pub phones: Vec<String>,
    pub head_prob: f64,
    pub next_word_prob: Vec<f64>,
}
