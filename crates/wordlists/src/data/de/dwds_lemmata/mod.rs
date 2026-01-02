use crate::{data::data_path, wordlist::stream::{UnsortedWords, WordStream, from_csv_zst_file}};

pub fn load() -> Result<WordStream<UnsortedWords>, std::io::Error>{
    from_csv_zst_file(data_path().join("de/dwds_lemmata/dwds_lemmata_2026-01-01.csv.zst"))
}
