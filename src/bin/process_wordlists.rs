use std::io;
use std::path::Path;
use wordle::wordlist::{Word, stream::{BoxedWordStream, WordStream, from_csv_zst_file, from_unsorted_zst_file}};

struct OutputConfig {
    output_path: &'static str,
    inputs: &'static [&'static str],
}

const OUTPUTS: &[OutputConfig] = &[
    OutputConfig {
        output_path: "wordlists/processed/de.txt.zst",
        inputs: &[
            "wordlists/original/de/davidak.txt.zst",
            "wordlists/original/de/dwds_lemmata_2026-01-01.csv.zst",
        ],
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
/// filter to 5-char words, filter non-alphabetic, lowercase, dedup.
fn process_input_file(path: &str) -> io::Result<BoxedWordStream> {
    let processed = if path.contains(".csv") {
        process_input_stream(from_csv_zst_file(path)?)
    } else {
        process_input_stream(from_unsorted_zst_file(path)?)
    };

    Ok(processed)
}

fn process_input_stream(stream: WordStream<impl Iterator<Item=io::Result<Word>> + 'static>) -> BoxedWordStream {
        return stream
            .filter(|w| w.chars().count() == 5)
            .filter_non_alphabetic()
            .to_lowercase()
            .dedup()
            .boxed()
    }

fn process_output(config: &OutputConfig) -> io::Result<()> {
    // Ensure output directory exists
    if let Some(parent) = Path::new(config.output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Process first input
    let mut stream = process_input_file(config.inputs[0])?;

    // Merge additional inputs
    for input in &config.inputs[1..] {
        stream = stream.merge(process_input_file(input)?);
    }

    stream = stream.dedup();

    // Write merged result
    stream.write_to_zst_file(config.output_path)?;

    println!("Processed: {}", config.output_path);
    Ok(())
}
