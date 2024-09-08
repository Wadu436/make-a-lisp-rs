use thiserror::Error;

#[derive(Debug, Error)]
pub enum MalError {
    #[error("unbalanced brackets")]
    UnbalancedBrackets,
    #[error("unbalanced double quotes")]
    UnbalancedDoubleQuotes,
    #[error("unbalanced hashmap")]
    UnbalancedHashMap,
    #[error("expected EOF, found {found:?}")]
    ExpectedEOF { found: String },
    #[error("invalid escape sequence")]
    InvalidEscapeSequence,
    #[error("invalid token")]
    InvalidToken,
    #[error("invalid hash map key")]
    InvalidHashMapKey,
    #[error("unexpected {found}")]
    Unexpected { found: String },
}
