use std::fmt;

#[derive(Debug, Default)]
pub struct ReactorError(pub String);

impl fmt::Display for ReactorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.0.as_str())
    }
}

impl std::error::Error for ReactorError {}
