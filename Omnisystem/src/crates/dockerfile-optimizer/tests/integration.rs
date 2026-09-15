use dockerfile_optimizer::{analyze, parse, Suggestion};

#[test]
fn realistic_dockerfile_flags_expected_issues() {
    let source = "\
FROM golang:latest
RUN apt-get update
RUN apt-get install -y git
RUN go build -o app .
ADD config.tar.gz /etc/app
CMD [\"./app\"]
";
    let instructions = parse(source).unwrap();
    let suggestions = analyze(&instructions);

    assert!(suggestions.iter().any(|s| matches!(s, Suggestion::UnpinnedBaseImage { image, .. } if image == "golang:latest")));
    assert!(suggestions.iter().any(|s| matches!(s, Suggestion::CombineConsecutiveRun { lines } if lines.len() == 3)));
    assert!(suggestions.iter().any(|s| matches!(s, Suggestion::MissingMultiStageOpportunity { .. })));
    assert!(suggestions.iter().any(|s| matches!(s, Suggestion::PreferCopyOverAdd { .. })));
}

#[test]
fn well_written_multi_stage_dockerfile_has_no_suggestions() {
    let source = "\
FROM golang:1.22.4 AS builder
WORKDIR /src
COPY . .
RUN go build -o /out/app .

FROM alpine:3.19.1
COPY --from=builder /out/app /usr/local/bin/app
CMD [\"/usr/local/bin/app\"]
";
    let instructions = parse(source).unwrap();
    let suggestions = analyze(&instructions);
    assert!(suggestions.is_empty(), "expected no suggestions, got: {:?}", suggestions);
}

#[test]
fn line_continuation_preserves_the_starting_line_number_in_suggestions() {
    let source = "FROM ubuntu:22.04\nRUN apt-get update && \\\n    apt-get install -y curl\n";
    let instructions = parse(source).unwrap();
    assert_eq!(instructions[1].line, 2);
}
