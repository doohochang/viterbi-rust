pub mod read;

pub use self::read::read_words as read_all;

use phone::Phone;

#[derive(Debug)]
pub struct Word<'p> {
    pub name: String,
    pub phones: Vec<&'p Phone>,
    pub head_prob: f64,
    pub next_word_prob: Vec<f64>,
}
