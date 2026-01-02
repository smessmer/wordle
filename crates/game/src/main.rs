fn main() {
    print_words();
}

fn print_words() {
    let loaded =
        wordle_wordlists_processing::stream::from_txt_zstd(wordle_game::wordlists::DE).unwrap();
    for word in loaded {
        println!("{}", word.unwrap());
    }
}
