use std::fmt::Display;
pub mod client;
pub mod traits;
pub mod paged;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum RestSvcError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("other error: {0}")]
    OtherError(String),
}

#[derive(Error, Debug)]
pub enum RestSvcExpandedError {
    SerdeExpandedError {
        source: RestSvcError,
        extra: String,
    },
    OtherError(String),
}

impl Display for RestSvcExpandedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RestSvcExpandedError::SerdeExpandedError { source, extra } => {
                write!(f, "SerdeExpandedError: {} \n {}", source, extra)
            }
            RestSvcExpandedError::OtherError(extra) => {
                write!(f, "OtherError: {}", extra)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
