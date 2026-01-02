use std::{collections::HashSet, io::Cursor};

use common_macros::hash_set;
use wordle_wordlists_processing::{Word, stream::{WordStream, from_txt_zstd}};

const DATA: &[u8] = include_bytes!("davidak.txt.zst");

fn remove_words() -> HashSet<&'static str> {
    hash_set! {
        "œuvre",
        "ōsaka",
        "český",
        "česká",
        "české",
    }
}

pub fn load() -> Result<WordStream<impl Iterator<Item = std::io::Result<Word>> + 'static>, std::io::Error> {
    Ok(from_txt_zstd(Cursor::new(DATA))?
        .filter(|w| !remove_words().contains(w.to_lowercase().as_str())))
}
