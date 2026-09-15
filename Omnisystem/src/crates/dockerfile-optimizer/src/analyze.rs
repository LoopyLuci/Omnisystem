//! Real anti-pattern detection over a parsed instruction list.

use crate::types::{Instruction, Suggestion};

const BUILD_TOOL_MARKERS: &[&str] =
    &["gcc ", "cc ", "make ", "cargo build", "go build", "mvn ", "npm run build", "yarn build", "g++ "];

/// Analyze a parsed Dockerfile and return every anti-pattern found.
pub fn analyze(instructions: &[Instruction]) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    suggestions.extend(combine_consecutive_run(instructions));
    suggestions.extend(unpinned_base_images(instructions));
    suggestions.extend(missing_multi_stage_opportunity(instructions));
    suggestions.extend(prefer_copy_over_add(instructions));

    suggestions
}

fn combine_consecutive_run(instructions: &[Instruction]) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();
    let mut run_lines: Vec<usize> = Vec::new();

    let flush = |run_lines: &mut Vec<usize>, out: &mut Vec<Suggestion>| {
        if run_lines.len() >= 2 {
            out.push(Suggestion::CombineConsecutiveRun { lines: run_lines.clone() });
        }
        run_lines.clear();
    };

    for instr in instructions {
        if instr.keyword == "RUN" {
            run_lines.push(instr.line);
        } else {
            flush(&mut run_lines, &mut suggestions);
        }
    }
    flush(&mut run_lines, &mut suggestions);

    suggestions
}

fn unpinned_base_images(instructions: &[Instruction]) -> Vec<Suggestion> {
    let mut stage_names: Vec<String> = Vec::new();
    let mut suggestions = Vec::new();

    for instr in instructions {
        if instr.keyword != "FROM" {
            continue;
        }
        let args = instr.args.trim();

        // `FROM builder AS stage` / `FROM ... as stage` -- record the alias
        // so a later `FROM stage` (a real multi-stage reference) isn't
        // flagged as an external image.
        let mut parts = args.split_whitespace();
        let image = match parts.next() {
            Some(i) => i.to_string(),
            None => continue,
        };
        if let (Some(as_kw), Some(alias)) = (parts.next(), parts.next()) {
            if as_kw.eq_ignore_ascii_case("as") {
                stage_names.push(alias.to_string());
            }
        }

        if image == "scratch" || stage_names.iter().any(|s| s == &image) {
            continue;
        }

        let is_unpinned = match image.rsplit_once(':') {
            // `localhost:5000/img` has a ':' that's a registry port, not a
            // tag -- only treat it as a tag if there's no '/' after the ':'.
            Some((_, tag)) if !tag.contains('/') => tag == "latest",
            _ => true, // no ':' at all -> implicit `:latest`.
        };

        if is_unpinned {
            suggestions.push(Suggestion::UnpinnedBaseImage { line: instr.line, image });
        }
    }

    suggestions
}

fn missing_multi_stage_opportunity(instructions: &[Instruction]) -> Vec<Suggestion> {
    let from_count = instructions.iter().filter(|i| i.keyword == "FROM").count();
    if from_count != 1 {
        return Vec::new();
    }

    instructions
        .iter()
        .find(|i| i.keyword == "RUN" && BUILD_TOOL_MARKERS.iter().any(|m| i.args.contains(m)))
        .map(|i| vec![Suggestion::MissingMultiStageOpportunity { line: i.line }])
        .unwrap_or_default()
}

fn prefer_copy_over_add(instructions: &[Instruction]) -> Vec<Suggestion> {
    instructions
        .iter()
        .filter(|i| i.keyword == "ADD")
        .map(|i| Suggestion::PreferCopyOverAdd { line: i.line })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    #[test]
    fn detects_combinable_runs() {
        let instrs = parse("FROM ubuntu:22.04\nRUN apt-get update\nRUN apt-get install -y curl\nCMD [\"true\"]\n").unwrap();
        let suggestions = analyze(&instrs);
        assert!(suggestions.iter().any(|s| matches!(s, Suggestion::CombineConsecutiveRun { lines } if lines.len() == 2)));
    }

    #[test]
    fn single_run_is_not_flagged() {
        let instrs = parse("FROM ubuntu:22.04\nRUN apt-get update\nCMD [\"true\"]\n").unwrap();
        let suggestions = analyze(&instrs);
        assert!(!suggestions.iter().any(|s| matches!(s, Suggestion::CombineConsecutiveRun { .. })));
    }

    #[test]
    fn detects_unpinned_latest_tag() {
        let instrs = parse("FROM ubuntu:latest\n").unwrap();
        let suggestions = analyze(&instrs);
        assert!(suggestions.iter().any(|s| matches!(s, Suggestion::UnpinnedBaseImage { .. })));
    }

    #[test]
    fn detects_missing_tag_as_unpinned() {
        let instrs = parse("FROM ubuntu\n").unwrap();
        let suggestions = analyze(&instrs);
        assert!(suggestions.iter().any(|s| matches!(s, Suggestion::UnpinnedBaseImage { .. })));
    }

    #[test]
    fn pinned_tag_is_not_flagged() {
        let instrs = parse("FROM ubuntu:22.04\n").unwrap();
        let suggestions = analyze(&instrs);
        assert!(!suggestions.iter().any(|s| matches!(s, Suggestion::UnpinnedBaseImage { .. })));
    }

    #[test]
    fn scratch_base_is_never_flagged() {
        let instrs = parse("FROM scratch\n").unwrap();
        assert!(analyze(&instrs).is_empty());
    }

    #[test]
    fn multi_stage_reference_to_prior_alias_is_not_flagged_as_unpinned() {
        let src = "FROM golang:1.22 AS builder\nRUN go build -o app .\nFROM builder\nCMD [\"./app\"]\n";
        let instrs = parse(src).unwrap();
        let suggestions = analyze(&instrs);
        // Only the first FROM (golang:1.22, which IS pinned) is a real base image;
        // `FROM builder` refers to the prior stage and must not be flagged.
        assert!(!suggestions.iter().any(|s| matches!(s, Suggestion::UnpinnedBaseImage { .. })));
    }

    #[test]
    fn registry_port_is_not_mistaken_for_unpinned_tag_check() {
        let instrs = parse("FROM localhost:5000/myapp:1.0\n").unwrap();
        let suggestions = analyze(&instrs);
        assert!(!suggestions.iter().any(|s| matches!(s, Suggestion::UnpinnedBaseImage { .. })));
    }

    #[test]
    fn single_stage_build_with_compiler_flags_multi_stage_opportunity() {
        let src = "FROM golang:1.22\nRUN go build -o app .\nCMD [\"./app\"]\n";
        let instrs = parse(src).unwrap();
        let suggestions = analyze(&instrs);
        assert!(suggestions.iter().any(|s| matches!(s, Suggestion::MissingMultiStageOpportunity { .. })));
    }

    #[test]
    fn multi_stage_build_is_not_flagged_for_missing_multi_stage() {
        let src = "FROM golang:1.22 AS builder\nRUN go build -o app .\nFROM alpine:3.19\nCOPY --from=builder /app /app\n";
        let instrs = parse(src).unwrap();
        let suggestions = analyze(&instrs);
        assert!(!suggestions.iter().any(|s| matches!(s, Suggestion::MissingMultiStageOpportunity { .. })));
    }

    #[test]
    fn add_is_flagged_prefer_copy() {
        let instrs = parse("FROM ubuntu:22.04\nADD app.tar.gz /app\n").unwrap();
        let suggestions = analyze(&instrs);
        assert!(suggestions.iter().any(|s| matches!(s, Suggestion::PreferCopyOverAdd { .. })));
    }

    #[test]
    fn copy_is_never_flagged() {
        let instrs = parse("FROM ubuntu:22.04\nCOPY . /app\n").unwrap();
        let suggestions = analyze(&instrs);
        assert!(!suggestions.iter().any(|s| matches!(s, Suggestion::PreferCopyOverAdd { .. })));
    }
}
