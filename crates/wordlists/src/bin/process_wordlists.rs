use std::io;
use std::path::{PathBuf};

use wordle_wordlists::wordlist::{Word, stream::{BoxedWordStream, WordStream}};

struct OutputConfig {
    output_path: &'static str,
    inputs: Vec<BoxedWordStream>,
}

impl OutputConfig {
    fn output_full_path(&self) -> PathBuf {
        data_path().join(self.output_path)
    }

    fn into_inputs(self) -> Vec<BoxedWordStream> {
        self.inputs
    }
}

fn outputs() -> [OutputConfig; 1] {
    [
        OutputConfig {
            output_path: "processed/de.txt.zst",
            inputs: vec![
                process_input_stream(wordle_wordlists::data::de::davidak::load().unwrap()),
                process_input_stream(wordle_wordlists::data::de::dwds_lemmata::load().unwrap()),
            ],
        },
        // Add more outputs here later
    ]
}

fn data_path() -> PathBuf {
    std::env::current_exe().unwrap()
    // go out of target dir
    .parent().unwrap()
    .parent().unwrap()
    .parent().unwrap()
    // and into the data dir
    .join("crates/wordlists/data")
}

fn process_input_stream(stream: WordStream<impl Iterator<Item=io::Result<Word>> + 'static>) -> BoxedWordStream {
        return stream
            .filter(|w| w.chars().count() == 5)
            .filter_non_alphabetic()
            .to_lowercase()
            .dedup()
            .boxed()
    }

fn process_output(config: OutputConfig) -> io::Result<()> {
    let output_path = config.output_full_path();
    let mut inputs = config.into_inputs().into_iter();

    println!("Processing: {}", output_path.display());

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Process first input
    let mut stream = inputs.next().expect("At least one input required");

    // Merge additional inputs
    for input in inputs {
        stream = stream.merge(input);
    }

    stream = stream.dedup();

    // Write merged result
    stream.write_to_zst_file(&output_path)?;

    println!("Processed: {}", output_path.display());
    Ok(())
}

fn main() -> io::Result<()> {
    for config in outputs() {
        process_output(config)?;
    }
    Ok(())
}
