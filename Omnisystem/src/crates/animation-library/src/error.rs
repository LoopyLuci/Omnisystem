//! Error types for animation timeline evaluation.

/// Errors that can occur while building or sampling an animation timeline.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A keyframe list was empty.
    NoKeyframes,
    /// Keyframes were not given in non-decreasing time order.
    UnorderedKeyframes(f64, f64),
    /// A duration was zero or negative.
    InvalidDuration(f64),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoKeyframes => write!(f, "animation has no keyframes"),
            Error::UnorderedKeyframes(prev, next) => {
                write!(f, "keyframe at t={} precedes keyframe at t={}", next, prev)
            }
            Error::InvalidDuration(d) => write!(f, "invalid duration: {}", d),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for animation operations.
pub type Result<T> = std::result::Result<T, Error>;
