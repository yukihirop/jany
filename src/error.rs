use thiserror::Error;

#[derive(Debug, Error)]
pub enum JanyError {
    #[error("{0}")]
    Usage(String),
    #[error("no such command: {0} (looked in {1})")]
    NoCommand(String, String),
    #[error("schema: {0}")]
    Schema(String),
    #[error("could not interpret: {0}")]
    Unresolved(String),
    #[error("assemble: {0}")]
    Assemble(String),
    #[error("jev: {0}")]
    Jev(String),
    #[error("interpretation rejected (confidence {0:.2} < {1:.2}); rerun with --explain to see why")]
    LowConfidence(f32, f32),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("config: {0}")]
    Config(String),
}

impl JanyError {
    pub fn exit_code(&self) -> i32 {
        match self {
            JanyError::Usage(_) | JanyError::Config(_) | JanyError::NoCommand(..) | JanyError::Schema(_) => 64,
            _ => 2,
        }
    }
}
