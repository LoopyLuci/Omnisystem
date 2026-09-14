//! Integration test: real Titan and Sylva-subset fixtures exercising the
//! struct/enum lowering added in this phase, run through the full
//! parse -> IrModule -> RustCodegen pipeline.
//!
//! This checks the pipeline end-to-end at the IR/codegen level (structurally
//! asserting on the generated Rust). The stronger bar — actually compiling
//! the generated Rust with `rustc`/`cargo` and running it, checked against
//! real Titan interpreter output for the Titan fixture — was run manually
//! and is not repeated here (spawning `rustc`/`cargo` from a unit test isn't
//! portable across CI environments); see the project's verification notes
//! for that run's real output.

use ir::{Codegen, IrCompiler, IrTypeDefKind, RustCodegen};

const TITAN_FIXTURE: &str = include_str!("../fixtures/structs_enums.titan");
const SYLVA_FIXTURE: &str = include_str!("../fixtures/structs_enums.sylva");

#[test]
fn titan_fixture_lowers_and_emits_struct_and_enum() {
    let compiler = IrCompiler::new();
    let rust = compiler
        .compile_titan_to_rust(TITAN_FIXTURE, "structs_enums.titan", "structs_enums")
        .expect("titan fixture should lower cleanly");

    assert!(rust.contains("pub struct Point"), "{rust}");
    assert!(rust.contains("pub x: i64"), "{rust}");
    assert!(rust.contains("pub y: i64"), "{rust}");
    assert!(rust.contains("pub enum Shape"), "{rust}");
    assert!(rust.contains("Circle(i64)"), "{rust}");
    assert!(rust.contains("Rectangle(i64, i64)"), "{rust}");
    assert!(rust.contains("Empty,"), "{rust}");
    assert!(rust.contains("Point { x: a, y: b }"), "{rust}");
    assert!(rust.contains("p.x + p.y"), "{rust}");
    assert!(rust.contains("Shape::Circle(7i64)"), "{rust}");
    assert!(rust.contains("Shape::Rectangle(3i64, 4i64)"), "{rust}");
    assert!(rust.contains("Shape::Empty"), "{rust}");
}

#[test]
fn sylva_fixture_lowers_and_emits_struct_and_enum() {
    let compiler = IrCompiler::new();
    let rust = compiler
        .compile_to_rust(SYLVA_FIXTURE, "structs_enums")
        .expect("sylva-subset fixture should lower cleanly");

    assert!(rust.contains("pub struct Point"), "{rust}");
    assert!(rust.contains("pub enum Shape"), "{rust}");
    assert!(rust.contains("Circle(i64)"), "{rust}");
    assert!(rust.contains("Rectangle(i64, i64)"), "{rust}");
    assert!(rust.contains("Empty,"), "{rust}");
    assert!(rust.contains("Point { x: a, y: b }"), "{rust}");
    assert!(rust.contains("Shape::Circle(7i64)"), "{rust}");
}

#[test]
fn titan_and_sylva_fixtures_produce_structurally_identical_typedefs() {
    // Both fixtures declare the same Point/Shape shapes in their respective
    // surface syntaxes — the emitted IrModule type definitions should match.
    let titan_module = ir::lower_titan(TITAN_FIXTURE, "structs_enums.titan", "structs_enums")
        .expect("titan fixture should lower");
    let sylva_module =
        ir::parse(SYLVA_FIXTURE, "structs_enums").expect("sylva-subset fixture should parse");

    assert_eq!(titan_module.types.len(), 2);
    assert_eq!(sylva_module.types.len(), 2);

    for (t, s) in titan_module.types.iter().zip(sylva_module.types.iter()) {
        assert_eq!(t.name, s.name);
        match (&t.kind, &s.kind) {
            (IrTypeDefKind::Struct { fields: tf }, IrTypeDefKind::Struct { fields: sf }) => {
                assert_eq!(tf, sf);
            }
            (IrTypeDefKind::Enum { variants: tv }, IrTypeDefKind::Enum { variants: sv }) => {
                assert_eq!(tv, sv);
            }
            other => panic!("type kind mismatch: {other:?}"),
        }
    }

    let mut cg = RustCodegen::new();
    let titan_rust = cg.emit_module(&titan_module).unwrap();
    let mut cg2 = RustCodegen::new();
    let sylva_rust = cg2.emit_module(&sylva_module).unwrap();
    assert!(titan_rust.contains("pub struct Point"));
    assert!(sylva_rust.contains("pub struct Point"));
}
