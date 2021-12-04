use std::{
    env::var,
    error,
    fs::File,
    io::{self, Read, Write},
    process::{Command, ExitStatus},
};
use tempfile::NamedTempFile;

#[derive(Debug)]
pub enum EditorInputError {
    ExitStatus(ExitStatus),
    IO(io::Error),
}

impl std::fmt::Display for EditorInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorInputError::ExitStatus(ex) => {
                write!(f, "Editor exited with unsuccessful exit status {}", ex)
            }
            EditorInputError::IO(io) => io.fmt(f),
        }
    }
}

impl error::Error for EditorInputError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            EditorInputError::ExitStatus(_) => None,
            EditorInputError::IO(ref e) => Some(e),
        }
    }
}

impl From<io::Error> for EditorInputError {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

/// Opens the editor specified by the $EDITOR environment variable (fallback `vi`)
/// and returns the saved text when the editor is closed.
///
/// # Arguments
///
///  * `placeholder` - Text that will be present in the temporary file being edited
///
pub fn input_from_editor(placeholder: &str) -> Result<String, EditorInputError> {
    let editor = var("EDITOR").unwrap_or("vi".to_string());

    let mut tmpfile = NamedTempFile::new()?;
    tmpfile.write_all(placeholder.as_bytes())?;
    let file_path = tmpfile.into_temp_path();

    let status = Command::new(editor).arg(&file_path).status()?;
    if !status.success() {
        return Err(EditorInputError::ExitStatus(status));
    }

    let mut editable = String::new();
    File::open(file_path)?.read_to_string(&mut editable)?;

    Ok(editable)
}
