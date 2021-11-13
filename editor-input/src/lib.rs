use std::{
    env::var,
    fs::File,
    io::{Read, Result, Write},
    process::Command,
};
use tempfile::NamedTempFile;

pub fn input_from_editor(placeholder: &str) -> Result<String> {
    let editor = var("EDITOR").unwrap_or("vi".to_string());

    let mut tmpfile = NamedTempFile::new()?;
    tmpfile.write_all(placeholder.as_bytes())?;
    let file_path = tmpfile.into_temp_path();

    let status = Command::new(editor).arg(&file_path).status()?;
    assert!(status.success(), "editor gave bad exit code");

    let mut editable = String::new();
    File::open(file_path)?.read_to_string(&mut editable)?;

    Ok(editable)
}
