use std::io::Cursor;

use wordle_wordlists_processing::stream::{from_txt_zstd, UnsortedWords, WordStream};

const DATA: &[u8] = include_bytes!("davidak.txt.zst");

pub fn load() -> Result<WordStream<UnsortedWords>, std::io::Error> {
    from_txt_zstd(Cursor::new(DATA))
}
