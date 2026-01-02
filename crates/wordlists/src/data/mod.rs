use std::path::PathBuf;


fn data_path() -> PathBuf {
    std::env::current_exe().unwrap()
    // go out of target dir
    .parent().unwrap()
    .parent().unwrap()
    .parent().unwrap()
    // and into the data dir
    .join("crates/wordlists/src/data")
}

pub mod de;