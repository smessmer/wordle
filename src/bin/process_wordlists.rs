use std::io;
use std::path::Path;
use wordle::wordlist::stream::{from_unsorted_zst_file, BoxedWordStream};

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

/// Loads a single input file and applies the standard processing pipeline:
/// filter to 5-char words, lowercase, dedup.
fn process_input(path: &str) -> io::Result<BoxedWordStream> {
    Ok(from_unsorted_zst_file(path)?
        .filter(|w| w.chars().count() == 5)
        .to_lowercase()
        .dedup()
        .boxed())
}

fn process_output(config: &OutputConfig) -> io::Result<()> {
    // Ensure output directory exists
    if let Some(parent) = Path::new(config.output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Process first input
    let mut stream = process_input(config.inputs[0])?;

    // Merge additional inputs
    for input in &config.inputs[1..] {
        stream = stream.merge(process_input(input)?);
    }

    // Write merged result
    stream.write_to_zst_file(config.output_path)?;

    println!("Processed: {}", config.output_path);
    Ok(())
}
