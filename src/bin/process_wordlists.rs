use std::io;
use std::path::Path;
use wordle::wordlist::stream::{from_unsorted_zst_file, WordStream};

struct OutputConfig {
    output_path: &'static str,
    inputs: &'static [&'static str],
}

const OUTPUTS: &[OutputConfig] = &[
    OutputConfig {
        output_path: "wordlists/processed/de.txt.zst",
        inputs: &["wordlists/original/de/davidak.txt.zst"],
    },
    // Add more outputs here later
];

fn main() -> io::Result<()> {
    for config in OUTPUTS {
        process_output(config)?;
    }
    Ok(())
}

fn process_output(config: &OutputConfig) -> io::Result<()> {
    // Ensure output directory exists
    if let Some(parent) = Path::new(config.output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Load and merge all inputs into a WordSet, then stream from that
    let mut combined = from_unsorted_zst_file(config.inputs[0])?
        .filter(|w| w.chars().count() == 5)
        .to_lowercase()
        .dedup()
        .collect_to_set()?;

    for input in &config.inputs[1..] {
        let additional = from_unsorted_zst_file(input)?
            .filter(|w| w.chars().count() == 5)
            .to_lowercase()
            .dedup()
            .collect_to_set()?;
        // Merge by chaining iterators and collecting back into a WordSet
        combined = combined
            .into_iter()
            .chain(additional)
            .map(|w| w.0)
            .collect();
    }

    // Write the combined set to output
    WordStream::from_word_set(combined).write_to_zst_file(config.output_path)?;

    println!("Processed: {}", config.output_path);
    Ok(())
}
