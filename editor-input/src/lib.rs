use std::{
    env::var,
    fs::File,
    io::{Read, Result},
    process::Command,
};
use tempfile::NamedTempFile;

pub fn input_from_editor() -> Result<String> {
    let editor = var("EDITOR").unwrap_or("vi".to_string());

    let tmpfile = NamedTempFile::new()?;
    let file_path = tmpfile.into_temp_path();

    let status = Command::new(editor)
        .arg(&file_path)
        .status()?;
    assert!(status.success(), "editor gave bad exit code");

    let mut editable = String::new();
    File::open(file_path)?
        .read_to_string(&mut editable)?;

    Ok(editable)
}
