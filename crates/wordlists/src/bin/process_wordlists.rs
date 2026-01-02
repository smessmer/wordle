use std::io;
use std::path::{Path, PathBuf};

use wordle_wordlists::wordlist::{Word, stream::{BoxedWordStream, WordStream, from_csv_zst_file, from_unsorted_zst_file}};

struct OutputConfig {
    output_path: &'static str,
    inputs: &'static [&'static str],
}

impl OutputConfig {
    fn output_full_path(&self) -> PathBuf {
        data_path().join(self.output_path)
    }

    fn input_full_paths(&self) -> Vec<PathBuf> {
        self.inputs.iter().map(|p| data_path().join(p)).collect()
    }
}

const OUTPUTS: &[OutputConfig] = &[
    OutputConfig {
        output_path: "processed/de.txt.zst",
        inputs: &[
            "original/de/davidak.txt.zst",
            "original/de/dwds_lemmata_2026-01-01.csv.zst",
        ],
    },
    // Add more outputs here later
];

fn data_path() -> PathBuf {
    std::env::current_exe().unwrap()
    // go out of target dir
    .parent().unwrap()
    .parent().unwrap()
    .parent().unwrap()
    // and into the data dir
    .join("crates/wordlists/data")
}

/// Loads a single input file and applies the standard processing pipeline:
/// filter to 5-char words, filter non-alphabetic, lowercase, dedup.
fn process_input_file(path: &Path) -> io::Result<BoxedWordStream> {
    let processed = if path.to_str().unwrap().contains(".csv") {
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
    let input_paths = config.input_full_paths();
    let output_path = config.output_full_path();

    println!("Processing: {}", output_path.display());

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Process first input
    let mut stream = process_input_file(&input_paths[0])?;

    // Merge additional inputs
    for input in &input_paths[1..] {
        stream = stream.merge(process_input_file(&input)?);
    }

    stream = stream.dedup();

    // Write merged result
    stream.write_to_zst_file(&output_path)?;

    println!("Processed: {}", output_path.display());
    Ok(())
}

fn main() -> io::Result<()> {
    for config in OUTPUTS {
        process_output(config)?;
    }
    Ok(())
}
