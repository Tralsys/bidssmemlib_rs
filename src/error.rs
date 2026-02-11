use std::io;

/// Errors that can occur during shared memory operations.
#[derive(Debug, thiserror::Error)]
pub enum SMemError {
    #[error("shared memory not initialized or already disposed")]
    NotInitialized,

    #[error("position {pos} + size {size} exceeds capacity {capacity}")]
    OutOfBounds { pos: u64, size: u64, capacity: u64 },

    #[error("index {index} out of range (length: {length})")]
    IndexOutOfRange { index: usize, length: usize },

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("platform error: {0}")]
    Platform(String),
}

pub type SMemResult<T> = Result<T, SMemError>;
