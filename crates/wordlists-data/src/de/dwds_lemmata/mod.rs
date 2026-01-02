use std::io::Cursor;

use wordle_wordlists_processing::stream::{from_csv_zstd, UnsortedWords, WordStream};

const DATA: &[u8] = include_bytes!("dwds_lemmata_2026-01-01.csv.zst");

pub fn load() -> Result<WordStream<UnsortedWords>, std::io::Error> {
    from_csv_zstd(Cursor::new(DATA))
}
