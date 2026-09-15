//! Real Dockerfile parsing: comments/blanks stripped, line continuations
//! (`\`) folded, each instruction split into keyword + args.

use crate::error::{Error, Result};
use crate::types::Instruction;

/// Parse Dockerfile source text into a sequence of instructions.
pub fn parse(source: &str) -> Result<Vec<Instruction>> {
    let mut instructions = Vec::new();
    let mut pending: Option<(String, usize)> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim_end();

        let (accumulated_start, mut buffer) = match pending.take() {
            Some((buf, start)) => (start, buf),
            None => {
                let trimmed = line.trim_start();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                (line_no, String::new())
            }
        };

        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_suffix('\\') {
            buffer.push_str(stripped.trim_end());
            buffer.push(' ');
            pending = Some((buffer, accumulated_start));
            continue;
        }

        buffer.push_str(trimmed);
        let full = buffer.trim().to_string();
        if full.is_empty() {
            continue;
        }

        let (keyword, args) = match full.split_once(char::is_whitespace) {
            Some((kw, rest)) => (kw.to_uppercase(), rest.trim().to_string()),
            None => (full.to_uppercase(), String::new()),
        };

        instructions.push(Instruction { keyword, args, line: accumulated_start });
    }

    if let Some((_, start)) = pending {
        return Err(Error::DanglingContinuation(start));
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_instructions() {
        let src = "FROM ubuntu:22.04\nRUN apt-get update\nCMD [\"echo\", \"hi\"]\n";
        let instrs = parse(src).unwrap();
        assert_eq!(instrs.len(), 3);
        assert_eq!(instrs[0].keyword, "FROM");
        assert_eq!(instrs[0].args, "ubuntu:22.04");
        assert_eq!(instrs[1].keyword, "RUN");
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        let src = "# a comment\nFROM ubuntu\n\nRUN echo hi\n";
        let instrs = parse(src).unwrap();
        assert_eq!(instrs.len(), 2);
    }

    #[test]
    fn folds_line_continuations_into_one_instruction() {
        let src = "RUN apt-get update && \\\n    apt-get install -y curl\n";
        let instrs = parse(src).unwrap();
        assert_eq!(instrs.len(), 1);
        assert!(instrs[0].args.contains("apt-get install -y curl"));
        assert_eq!(instrs[0].line, 1);
    }

    #[test]
    fn dangling_continuation_at_eof_errors() {
        let src = "RUN echo hi && \\\n";
        assert!(parse(src).is_err());
    }

    #[test]
    fn keyword_is_uppercased() {
        let src = "from ubuntu:22.04\n";
        let instrs = parse(src).unwrap();
        assert_eq!(instrs[0].keyword, "FROM");
    }
}
