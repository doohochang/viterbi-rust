mod fileutil;
mod constants;
mod phone;

fn main() {
    let phones = phone::read_all("hmm.txt");
    println!("{:#?}", phones);
    println!("{}", phones.len());
}
