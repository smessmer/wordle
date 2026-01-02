use crate::{data::data_path, wordlist::stream::{UnsortedWords, WordStream, from_unsorted_zst_file}};


pub fn load() -> Result<WordStream<UnsortedWords>, std::io::Error>{
    from_unsorted_zst_file(data_path().join("de/davidak/davidak.txt.zst"))
}
