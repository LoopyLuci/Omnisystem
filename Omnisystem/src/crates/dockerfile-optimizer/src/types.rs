//! Data types
use serde::{Deserialize, Serialize};

/// A single parsed Dockerfile instruction line (comments and blank lines
/// are dropped; a line continuation with a trailing `\` is folded into one
/// instruction).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instruction {
    /// Instruction keyword, uppercased (`FROM`, `RUN`, `COPY`, ...).
    pub keyword: String,
    /// Everything after the keyword, trimmed.
    pub args: String,
    /// 1-indexed source line number the instruction started on.
    pub line: usize,
}

/// A category of Dockerfile anti-pattern this optimizer can detect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Suggestion {
    /// Multiple consecutive `RUN` instructions that could be combined with
    /// `&&` into one layer.
    CombineConsecutiveRun {
        /// 1-indexed lines of the RUN instructions that could be combined.
        lines: Vec<usize>,
    },
    /// A base image tag that isn't pinned to a specific version (`latest`
    /// or no tag at all), which makes builds non-reproducible.
    UnpinnedBaseImage {
        /// Line of the offending `FROM`.
        line: usize,
        /// The image reference as written.
        image: String,
    },
    /// The Dockerfile only has one build stage but installs build-time
    /// tooling (e.g. compilers) that bloats the final image -- a multi-stage
    /// build would let the final stage `COPY --from=` just the artifacts.
    MissingMultiStageOpportunity {
        /// Line of the `RUN` instruction that looks like a build step.
        line: usize,
    },
    /// `ADD` was used where `COPY` would do (ADD's remote-URL/auto-extract
    /// behavior is rarely intended and considered a footgun).
    PreferCopyOverAdd {
        /// Line of the offending `ADD`.
        line: usize,
    },
}
