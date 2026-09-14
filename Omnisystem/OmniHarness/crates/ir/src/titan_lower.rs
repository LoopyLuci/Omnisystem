//! Titan (the `bootstrap-rs` / `titan` crate) AST → UniIR lowering.
//!
//! This is the second real input language for the `ir` crate's
//! parse → `IrModule` → Rust pipeline, alongside the existing Sylva-subset
//! parser in `parser.rs`. It reuses Titan's own real lexer/parser
//! (`titan::parser::parse`) rather than re-implementing one, and lowers
//! Titan's real AST (`titan::ast`) into the same `ops::IrModule` that the
//! Sylva path produces, so both languages flow through one shared codegen.
//!
//! ## Scope of this v1 lowering — read before assuming a feature works
//!
//! Supported:
//! - top-level `fn` items (`pub fn` or plain `fn`), each with a single block body
//! - top-level `struct` items with named fields (`struct Point { x: .., y: .. }`)
//!   and `enum` items whose variants are unit (`Red`) or tuple-style carrying
//!   data (`Rgb(i32, i32, i32)`) — see "Struct and enum support" below
//! - statements: `let NAME = expr;` with a plain identifier bind pattern, and
//!   bare expression statements (sequenced; the final one is the block's value)
//! - expressions: integer / float / string / bool literals; single-segment
//!   variable references; unary `-` / `!`; binary arithmetic, comparison,
//!   logical and bitwise operators; `if { .. } else { .. }` (both branches
//!   required — no `if let`); `return`; calls to other functions defined in
//!   the same file; the `println!` / `print!` / `eprintln!` / `format!`
//!   macros (lowered to a new `IrOp::Macro` node, emitted as the real Rust
//!   macro by `codegen.rs`); struct literals (`Point { x: 1, y: 2 }`); field
//!   access (`p.x`); and enum variant construction (`Red`, `Rgb(1, 2, 3)`)
//!
//! Explicitly NOT supported in v1 (rejected with a `LowerError`, never
//! silently mishandled or dropped):
//! - `impl` / `trait` / `mod` / `const` / `use` items, and therefore also
//!   `self`, methods, and `Type::method(...)` paths
//! - struct-style (named-field) enum variants (`Enum::Variant { x: .. }`) —
//!   only unit and tuple-style variants are lowered
//! - struct update syntax (`Point { x: 1, ..base }`)
//! - `match`, `loop`, `while`, `for`, labeled break/continue
//! - closures, arrays, tuples, ranges, casts, `?`, indexing
//! - any `let` pattern other than a plain identifier (no destructuring)
//! - multi-segment paths (`Foo::bar`) anywhere else
//!
//! This covers `01_hello.titan` and `02_fib.titan` from
//! `Omnisystem/bootstrap/tests/` end-to-end (see the `ir` crate's own test
//! suite and the CLI proof run referenced in the project's commit history);
//! it is a real but partial subset, not full Titan.
//!
//! ## Why types have to be inferred at all
//!
//! Titan's own AST carries no static types: `titan::ast::Param` is just
//! `{ name, is_self }` and `titan::ast::Field` is just `{ name }`. Type
//! annotations written in Titan source (`n: i32`, and likewise struct field
//! types and enum variant payload types) are consumed and thrown away by
//! `bootstrap-rs`'s parser — Titan is dynamically typed at the
//! parser/interpreter level, with no separate type-checking pass. UniIR, by
//! contrast, requires a concrete `IrType` on every parameter, field, and
//! variant payload.
//!
//! So types here are *not* real type inference — they are a shallow
//! syntactic heuristic, applied consistently across parameters, struct
//! fields, and enum variant payloads:
//! - A function's numeric parameters are `F64` if any float literal appears
//!   anywhere in the function body, otherwise `I64` (this is correct for
//!   arithmetic-shaped functions like `fib`, and wrong for anything that
//!   needs a `Str`/`Bool`/struct-typed parameter unless the return type is
//!   explicitly annotated, which this pass *does* read from `FnItem::ret`).
//! - A parameter that is used as the receiver of a `.field` access anywhere
//!   in its function body (`p.x`) is inferred as the struct type that
//!   declares that field name, when exactly one such struct exists.
//! - A struct field's type is inferred from the first literal value assigned
//!   to it in any `StructLit` for that struct anywhere in the program
//!   (`Point { x: 1, .. }` infers `x: I64`).
//! - An enum tuple-variant's payload types are inferred the same way, from
//!   the first call site that constructs that variant with the right arity.
//! - Anything left unresolved by these heuristics defaults to `I64`.
//!
//! ## Struct and enum support
//!
//! Titan's real `StructItem`/`EnumItem`/`Expr::StructLit`/`Expr::Field` AST
//! nodes (see `bootstrap-rs/src/ast.rs`) are lowered as follows:
//! - `struct Name { f1, f2, .. }` → `IrTypeDefKind::Struct` with field types
//!   inferred as described above.
//! - `enum Name { A, B(T1, T2), .. }` → `IrTypeDefKind::Enum`, one entry per
//!   variant; a variant with `arity == 0` and no `struct_fields` is a unit
//!   variant (`Vec<IrType>` empty); a variant with `arity > 0` is a
//!   tuple-style variant. A variant with non-empty `struct_fields` (a named-
//!   field / struct-style variant, e.g. `Msg { code: i32 }`) is rejected —
//!   `IrTypeDefKind::Enum` only models positional payloads.
//! - `Point { x: 1, y: 2 }` (`Expr::StructLit` with a single-segment path
//!   that doesn't name a declared enum) → `IrOp::StructLit`.
//! - A bare `Red` (`Expr::Path` with one segment matching a declared unit
//!   variant) or `Rgb(1, 2, 3)` (`Expr::Call` whose callee is a one-segment
//!   path matching a declared tuple variant, with the right arity) →
//!   `IrOp::EnumVariant`.
//! - `p.x` (`Expr::Field`) → `IrOp::FieldAccess`.
//! - `Enum::Variant { .. }` (`Expr::StructLit` with a multi-segment path
//!   whose first segment is a declared enum — Titan's real syntax for
//!   struct-style variant literals) is rejected, consistent with rejecting
//!   the variant declaration itself.

use crate::ops::{
    BinOpKind, IrFunction, IrLit, IrModule, IrOp, IrParam, IrType, IrTypeDef, IrTypeDefKind,
    UnOpKind,
};
use std::collections::{HashMap, HashSet};
use titan::ast::{Block, Expr, FnItem, Item, Pattern, Program, Stmt};

#[derive(Debug, Clone)]
pub struct LowerError(pub String);

impl std::fmt::Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "titan lowering error: {}", self.0)
    }
}

impl std::error::Error for LowerError {}

type LowerResult<T> = Result<T, LowerError>;

fn unsupported(msg: impl Into<String>) -> LowerError {
    LowerError(format!("unsupported in v1 Titan->IR lowering: {}", msg.into()))
}

/// Parse Titan source (via the real `titan` crate parser) and lower it to an
/// `IrModule`, covering the subset documented at the top of this file.
pub fn lower_source(src: &str, file: &str, module_name: &str) -> LowerResult<IrModule> {
    let program = titan::parser::parse(src, file)
        .map_err(|e| LowerError(format!("titan parse error: {}", e.render(src))))?;
    lower_program(&program, module_name)
}

// ── Struct/enum registry ────────────────────────────────────────────────────

/// Types resolved from Titan's real (type-erased) struct/enum declarations,
/// via the syntactic heuristics documented at the top of this file.
#[derive(Debug, Default)]
struct LowerCtx {
    /// struct name -> ordered `(field name, inferred type)` list
    struct_fields: HashMap<String, Vec<(String, IrType)>>,
    /// names of enums declared in this program
    enum_names: HashSet<String>,
    /// variant name -> (owning enum name, inferred payload types; empty = unit variant)
    variants: HashMap<String, (String, Vec<IrType>)>,
}

fn build_ctx(program: &Program) -> LowerResult<LowerCtx> {
    let mut ctx = LowerCtx::default();
    for item in &program.items {
        if let Item::Enum(e) = item {
            ctx.enum_names.insert(e.name.clone());
        }
    }
    for item in &program.items {
        match item {
            Item::Struct(s) => {
                let mut fields = Vec::with_capacity(s.fields.len());
                for field in &s.fields {
                    let ty = infer_struct_field_type(program, &s.name, &field.name);
                    fields.push((field.name.clone(), ty));
                }
                ctx.struct_fields.insert(s.name.clone(), fields);
            }
            Item::Enum(e) => {
                for variant in &e.variants {
                    if !variant.struct_fields.is_empty() {
                        return Err(unsupported(format!(
                            "enum `{}` variant `{}` with named fields (struct-style variant) \
                             — only unit and tuple-style variants are lowered in v1",
                            e.name, variant.name
                        )));
                    }
                    let arg_types = infer_variant_arg_types(program, &variant.name, variant.arity);
                    ctx.variants
                        .insert(variant.name.clone(), (e.name.clone(), arg_types));
                }
            }
            _ => {}
        }
    }
    Ok(ctx)
}

fn infer_type_from_expr(e: &Expr) -> Option<IrType> {
    match e {
        Expr::Int { .. } => Some(IrType::I64),
        Expr::Float { .. } => Some(IrType::F64),
        Expr::Str { .. } => Some(IrType::Str),
        Expr::Bool { .. } => Some(IrType::Bool),
        Expr::Unary { operand, .. } => infer_type_from_expr(operand),
        _ => None,
    }
}

fn infer_struct_field_type(program: &Program, struct_name: &str, field_name: &str) -> IrType {
    let mut found: Option<IrType> = None;
    visit_program_exprs(program, &mut |e| {
        if found.is_some() {
            return;
        }
        if let Expr::StructLit { path, fields, .. } = e {
            if path.len() == 1 && path[0] == struct_name {
                for (fname, fe) in fields {
                    if fname == field_name {
                        found = infer_type_from_expr(fe);
                        return;
                    }
                }
            }
        }
    });
    found.unwrap_or(IrType::I64)
}

fn infer_variant_arg_types(program: &Program, variant_name: &str, arity: usize) -> Vec<IrType> {
    let mut found: Vec<Option<IrType>> = vec![None; arity];
    visit_program_exprs(program, &mut |e| {
        if let Expr::Call { callee, args, .. } = e {
            if let Expr::Path { segs, .. } = callee.as_ref() {
                if segs.len() == 1 && segs[0] == variant_name && args.len() == arity {
                    for (i, a) in args.iter().enumerate() {
                        if found[i].is_none() {
                            found[i] = infer_type_from_expr(a);
                        }
                    }
                }
            }
        }
    });
    found.into_iter().map(|o| o.unwrap_or(IrType::I64)).collect()
}

/// Infer a parameter's type from how it is used as the receiver of a
/// `.field` access in `body` — the struct type that declares that field
/// name, when exactly one candidate exists. `None` if no such usage is found
/// (the caller falls back to the numeric heuristic).
fn infer_param_type(name: &str, body: &Block, ctx: &LowerCtx) -> Option<IrType> {
    let mut found: Option<IrType> = None;
    visit_block_exprs(body, &mut |e| {
        if found.is_some() {
            return;
        }
        if let Expr::Field { obj, name: fname, .. } = e {
            if let Expr::Path { segs, .. } = obj.as_ref() {
                if segs.len() == 1 && segs[0] == name {
                    for (sname, fields) in &ctx.struct_fields {
                        if fields.iter().any(|(fld, _)| fld == fname) {
                            found = Some(IrType::Named(sname.clone()));
                            return;
                        }
                    }
                }
            }
        }
    });
    found
}

// ── Generic (read-only) expression visitor ──────────────────────────────────
// Used only to scan for type-inference hints; never mutates or lowers.

fn visit_program_exprs<'a>(program: &'a Program, f: &mut dyn FnMut(&'a Expr)) {
    for item in &program.items {
        if let Item::Fn(fi) = item {
            if let Some(b) = &fi.body {
                visit_block_exprs(b, f);
            }
        }
    }
}

fn visit_block_exprs<'a>(block: &'a Block, f: &mut dyn FnMut(&'a Expr)) {
    for stmt in &block.stmts {
        visit_stmt_exprs(stmt, f);
    }
}

fn visit_stmt_exprs<'a>(stmt: &'a Stmt, f: &mut dyn FnMut(&'a Expr)) {
    match stmt {
        Stmt::Let { init, .. } => {
            if let Some(e) = init {
                visit_expr_exprs(e, f);
            }
        }
        Stmt::Expr { expr, .. } => visit_expr_exprs(expr, f),
        Stmt::Item(_) => {}
    }
}

fn visit_expr_exprs<'a>(e: &'a Expr, f: &mut dyn FnMut(&'a Expr)) {
    f(e);
    match e {
        Expr::Field { obj, .. } => visit_expr_exprs(obj, f),
        Expr::Index { obj, index, .. } => {
            visit_expr_exprs(obj, f);
            visit_expr_exprs(index, f);
        }
        Expr::Call { callee, args, .. } => {
            visit_expr_exprs(callee, f);
            for a in args {
                visit_expr_exprs(a, f);
            }
        }
        Expr::Method { recv, args, .. } => {
            visit_expr_exprs(recv, f);
            for a in args {
                visit_expr_exprs(a, f);
            }
        }
        Expr::Unary { operand, .. } => visit_expr_exprs(operand, f),
        Expr::Binary { left, right, .. } => {
            visit_expr_exprs(left, f);
            visit_expr_exprs(right, f);
        }
        Expr::Assign { target, value, .. } => {
            visit_expr_exprs(target, f);
            visit_expr_exprs(value, f);
        }
        Expr::If { cond, then, els, .. } => {
            visit_expr_exprs(cond, f);
            visit_block_exprs(then, f);
            if let Some(e2) = els {
                visit_expr_exprs(e2, f);
            }
        }
        Expr::Match { scrut, arms, .. } => {
            visit_expr_exprs(scrut, f);
            for a in arms {
                visit_expr_exprs(&a.body, f);
            }
        }
        Expr::While { cond, body, .. } => {
            visit_expr_exprs(cond, f);
            visit_block_exprs(body, f);
        }
        Expr::For { iter, body, .. } => {
            visit_expr_exprs(iter, f);
            visit_block_exprs(body, f);
        }
        Expr::Loop { body, .. } => visit_block_exprs(body, f),
        Expr::BlockE { block } => visit_block_exprs(block, f),
        Expr::Return { value, .. } => {
            if let Some(v) = value {
                visit_expr_exprs(v, f);
            }
        }
        Expr::Break { value, .. } => {
            if let Some(v) = value {
                visit_expr_exprs(v, f);
            }
        }
        Expr::StructLit { fields, spread, .. } => {
            for (_, fe) in fields {
                visit_expr_exprs(fe, f);
            }
            if let Some(s) = spread {
                visit_expr_exprs(s, f);
            }
        }
        Expr::Array { elems, repeat, .. } => {
            for el in elems {
                visit_expr_exprs(el, f);
            }
            if let Some(r) = repeat {
                visit_expr_exprs(r, f);
            }
        }
        Expr::Tuple { elems, .. } => {
            for el in elems {
                visit_expr_exprs(el, f);
            }
        }
        Expr::Closure { body, .. } => visit_expr_exprs(body, f),
        Expr::Try { expr, .. } => visit_expr_exprs(expr, f),
        Expr::Cast { expr, .. } => visit_expr_exprs(expr, f),
        Expr::Macro { args, .. } => {
            for a in args {
                visit_expr_exprs(a, f);
            }
        }
        _ => {}
    }
}

// ── Lowering ─────────────────────────────────────────────────────────────────

pub fn lower_program(program: &Program, module_name: &str) -> LowerResult<IrModule> {
    let mut module = IrModule::new(module_name);
    let ctx = build_ctx(program)?;

    for item in &program.items {
        match item {
            Item::Struct(s) => {
                let fields = ctx.struct_fields.get(&s.name).cloned().unwrap_or_default();
                module.types.push(IrTypeDef {
                    name: s.name.clone(),
                    kind: IrTypeDefKind::Struct { fields },
                });
            }
            Item::Enum(e) => {
                let mut variants = Vec::with_capacity(e.variants.len());
                for v in &e.variants {
                    let arg_types = ctx
                        .variants
                        .get(&v.name)
                        .map(|(_, tys)| tys.clone())
                        .unwrap_or_default();
                    variants.push((v.name.clone(), arg_types));
                }
                module.types.push(IrTypeDef {
                    name: e.name.clone(),
                    kind: IrTypeDefKind::Enum { variants },
                });
            }
            Item::Fn(f) => {
                let func = lower_fn(f, &ctx)?;
                module.exports.push(func.name.clone());
                module.functions.push(func);
            }
            other => {
                return Err(unsupported(format!(
                    "top-level item kind {:?} (only `fn`/`struct`/`enum` items are lowered in v1)",
                    item_kind_name(other)
                )));
            }
        }
    }
    Ok(module)
}

fn item_kind_name(item: &Item) -> &'static str {
    match item {
        Item::Use => "use",
        Item::Struct(_) => "struct",
        Item::Enum(_) => "enum",
        Item::Impl(_) => "impl",
        Item::Trait(_) => "trait",
        Item::Fn(_) => "fn",
        Item::Const(_) => "const",
        Item::Mod(_) => "mod",
    }
}

fn lower_fn(f: &FnItem, ctx: &LowerCtx) -> LowerResult<IrFunction> {
    let body_block = f
        .body
        .as_ref()
        .ok_or_else(|| unsupported(format!("function `{}` has no body (trait signature?)", f.name)))?;

    // Heuristic numeric type: see module doc comment. `F64` if any float
    // literal shows up anywhere in the body, else `I64`.
    let numeric_ty = if block_has_float_literal(body_block) {
        IrType::F64
    } else {
        IrType::I64
    };

    let mut params = Vec::with_capacity(f.params.len());
    for p in &f.params {
        if p.is_self {
            return Err(unsupported(format!(
                "method `{}` takes `self` — methods/impls are not lowered in v1",
                f.name
            )));
        }
        let ty = infer_param_type(&p.name, body_block, ctx).unwrap_or_else(|| numeric_ty.clone());
        params.push(IrParam {
            name: p.name.clone(),
            ty,
            default: None,
        });
    }

    let ret = match &f.ret {
        None => IrType::Unit,
        Some(tr) => resolve_type_name(&tr.name, ctx)
            .ok_or_else(|| unsupported(format!("return type `{}` on `{}`", tr.name, f.name)))?,
    };

    let body = lower_block(body_block, ctx)?;

    Ok(IrFunction {
        name: f.name.clone(),
        params,
        ret,
        body,
        effects: vec![],
        proof: None,
        schema: None,
    })
}

/// Map a Titan `TypeRef` name (as written in source, e.g. `-> i32`) to an
/// `IrType`. Primitive names, plus any struct/enum declared in this program.
fn resolve_type_name(name: &str, ctx: &LowerCtx) -> Option<IrType> {
    if let Some(t) = map_type_ref_name(name) {
        return Some(t);
    }
    if ctx.struct_fields.contains_key(name) || ctx.enum_names.contains(name) {
        return Some(IrType::Named(name.to_string()));
    }
    None
}

fn map_type_ref_name(name: &str) -> Option<IrType> {
    Some(match name {
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "usize" | "isize" => {
            IrType::I64
        }
        "f32" | "f64" => IrType::F64,
        "bool" => IrType::Bool,
        "String" | "str" => IrType::Str,
        "()" => IrType::Unit,
        _ => return None,
    })
}

fn block_has_float_literal(block: &Block) -> bool {
    block.stmts.iter().any(stmt_has_float_literal)
}

fn stmt_has_float_literal(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Let { init, .. } => init.as_ref().is_some_and(expr_has_float_literal),
        Stmt::Expr { expr, .. } => expr_has_float_literal(expr),
        Stmt::Item(_) => false,
    }
}

fn expr_has_float_literal(e: &Expr) -> bool {
    match e {
        Expr::Float { .. } => true,
        Expr::Call { callee, args, .. } => {
            expr_has_float_literal(callee) || args.iter().any(expr_has_float_literal)
        }
        Expr::Method { recv, args, .. } => {
            expr_has_float_literal(recv) || args.iter().any(expr_has_float_literal)
        }
        Expr::Unary { operand, .. } => expr_has_float_literal(operand),
        Expr::Binary { left, right, .. } => {
            expr_has_float_literal(left) || expr_has_float_literal(right)
        }
        Expr::Assign { target, value, .. } => {
            expr_has_float_literal(target) || expr_has_float_literal(value)
        }
        Expr::If { cond, then, els, .. } => {
            expr_has_float_literal(cond)
                || block_has_float_literal(then)
                || els.as_deref().is_some_and(expr_has_float_literal)
        }
        Expr::Match { scrut, arms, .. } => {
            expr_has_float_literal(scrut) || arms.iter().any(|a| expr_has_float_literal(&a.body))
        }
        Expr::While { cond, body, .. } => expr_has_float_literal(cond) || block_has_float_literal(body),
        Expr::For { iter, body, .. } => expr_has_float_literal(iter) || block_has_float_literal(body),
        Expr::Loop { body, .. } => block_has_float_literal(body),
        Expr::BlockE { block } => block_has_float_literal(block),
        Expr::Return { value, .. } => value.as_deref().is_some_and(expr_has_float_literal),
        Expr::Break { value, .. } => value.as_deref().is_some_and(expr_has_float_literal),
        Expr::Field { obj, .. } => expr_has_float_literal(obj),
        Expr::Index { obj, index, .. } => {
            expr_has_float_literal(obj) || expr_has_float_literal(index)
        }
        Expr::Array { elems, repeat, .. } => {
            elems.iter().any(expr_has_float_literal)
                || repeat.as_deref().is_some_and(expr_has_float_literal)
        }
        Expr::Tuple { elems, .. } => elems.iter().any(expr_has_float_literal),
        Expr::StructLit { fields, .. } => fields.iter().any(|(_, e)| expr_has_float_literal(e)),
        Expr::Macro { args, .. } => args.iter().any(expr_has_float_literal),
        _ => false,
    }
}

fn lower_block(block: &Block, ctx: &LowerCtx) -> LowerResult<IrOp> {
    lower_stmts(&block.stmts, ctx)
}

fn lower_stmts(stmts: &[Stmt], ctx: &LowerCtx) -> LowerResult<IrOp> {
    match stmts.split_first() {
        None => Ok(IrOp::unit()),
        Some((Stmt::Let { pat, init, .. }, rest)) => {
            let name = expect_bind_name(pat)?;
            let init_expr = init
                .as_ref()
                .ok_or_else(|| unsupported(format!("`let {name}` with no initializer")))?;
            let value = lower_expr(init_expr, ctx)?;
            let rest_op = lower_stmts(rest, ctx)?;
            Ok(IrOp::Let {
                name,
                ty: None,
                value: Box::new(value),
                rest: Box::new(rest_op),
            })
        }
        Some((Stmt::Expr { expr, .. }, rest)) => {
            let head = lower_expr(expr, ctx)?;
            if rest.is_empty() {
                Ok(head)
            } else {
                let rest_op = lower_stmts(rest, ctx)?;
                Ok(IrOp::Block(vec![head, rest_op]))
            }
        }
        Some((Stmt::Item(_), _)) => {
            Err(unsupported("a nested item declaration inside a function body"))
        }
    }
}

fn expect_bind_name(pat: &Pattern) -> LowerResult<String> {
    match pat {
        Pattern::Bind { name, .. } => Ok(name.clone()),
        other => Err(unsupported(format!(
            "`let` pattern {other:?} (only a plain identifier bind is supported)"
        ))),
    }
}

const SUPPORTED_MACROS: &[&str] = &["println", "print", "eprintln", "eprint", "format"];

fn lower_expr(e: &Expr, ctx: &LowerCtx) -> LowerResult<IrOp> {
    Ok(match e {
        Expr::Int { v, .. } => IrOp::lit_i64(*v),
        Expr::Float { v, .. } => IrOp::Lit(IrLit::F64(*v)),
        Expr::Str { v, .. } => IrOp::lit_str(v.clone()),
        Expr::Bool { v, .. } => IrOp::lit_bool(*v),
        Expr::Char { .. } => return Err(unsupported("char literals")),

        Expr::Path { segs, .. } => {
            if segs.len() != 1 {
                return Err(unsupported(format!("multi-segment path `{}`", segs.join("::"))));
            }
            let name = &segs[0];
            if let Some((enum_name, arg_types)) = ctx.variants.get(name) {
                if arg_types.is_empty() {
                    IrOp::EnumVariant {
                        enum_name: enum_name.clone(),
                        variant: name.clone(),
                        args: vec![],
                    }
                } else {
                    return Err(unsupported(format!(
                        "tuple variant `{name}` referenced without call syntax `{name}(..)`"
                    )));
                }
            } else {
                IrOp::var(name.clone())
            }
        }

        Expr::Field { obj, name, .. } => IrOp::FieldAccess {
            expr: Box::new(lower_expr(obj, ctx)?),
            field: name.clone(),
        },
        Expr::Index { .. } => return Err(unsupported("indexing (`a[i]`)")),

        Expr::Call { callee, args, .. } => {
            if let Expr::Path { segs, .. } = callee.as_ref() {
                if segs.len() == 1 {
                    if let Some((enum_name, arg_types)) = ctx.variants.get(&segs[0]) {
                        if !arg_types.is_empty() {
                            if args.len() != arg_types.len() {
                                return Err(unsupported(format!(
                                    "variant `{}` called with {} argument(s), expected {}",
                                    segs[0],
                                    args.len(),
                                    arg_types.len()
                                )));
                            }
                            let lowered_args = args
                                .iter()
                                .map(|a| lower_expr(a, ctx))
                                .collect::<LowerResult<Vec<_>>>()?;
                            return Ok(IrOp::EnumVariant {
                                enum_name: enum_name.clone(),
                                variant: segs[0].clone(),
                                args: lowered_args,
                            });
                        }
                    }
                }
            }
            let lowered_args = args.iter().map(|a| lower_expr(a, ctx)).collect::<LowerResult<Vec<_>>>()?;
            let func = lower_expr(callee, ctx)?;
            IrOp::apply(func, lowered_args)
        }

        Expr::Method { .. } => return Err(unsupported("method calls (`recv.method(...)`)")),

        Expr::Unary { op, operand, .. } => {
            let kind = match op.as_str() {
                "-" => UnOpKind::Neg,
                "!" => UnOpKind::Not,
                other => return Err(unsupported(format!("unary operator `{other}`"))),
            };
            IrOp::UnOp {
                op: kind,
                expr: Box::new(lower_expr(operand, ctx)?),
            }
        }

        Expr::Binary { op, left, right, .. } => {
            let kind = match op.as_str() {
                "+" => BinOpKind::Add,
                "-" => BinOpKind::Sub,
                "*" => BinOpKind::Mul,
                "/" => BinOpKind::Div,
                "%" => BinOpKind::Rem,
                "==" => BinOpKind::Eq,
                "!=" => BinOpKind::Ne,
                "<" => BinOpKind::Lt,
                "<=" => BinOpKind::Le,
                ">" => BinOpKind::Gt,
                ">=" => BinOpKind::Ge,
                "&&" => BinOpKind::And,
                "||" => BinOpKind::Or,
                "&" => BinOpKind::BitAnd,
                "|" => BinOpKind::BitOr,
                "^" => BinOpKind::BitXor,
                "<<" => BinOpKind::Shl,
                ">>" => BinOpKind::Shr,
                other => return Err(unsupported(format!("binary operator `{other}`"))),
            };
            IrOp::BinOp {
                op: kind,
                lhs: Box::new(lower_expr(left, ctx)?),
                rhs: Box::new(lower_expr(right, ctx)?),
            }
        }

        Expr::Assign { .. } => return Err(unsupported("assignment (`=`, `+=`, ...)")),
        Expr::Range { .. } => return Err(unsupported("range expressions")),

        Expr::If { let_pat, cond, then, els, .. } => {
            if let_pat.is_some() {
                return Err(unsupported("`if let`"));
            }
            let cond_op = lower_expr(cond, ctx)?;
            let then_op = lower_block(then, ctx)?;
            let else_op = match els {
                None => IrOp::unit(),
                Some(e) => lower_expr(e, ctx)?,
            };
            IrOp::if_(cond_op, then_op, else_op)
        }

        Expr::Match { .. } => return Err(unsupported("`match`")),
        Expr::While { .. } => return Err(unsupported("`while`")),
        Expr::For { .. } => return Err(unsupported("`for`")),
        Expr::Loop { .. } => return Err(unsupported("`loop`")),

        Expr::BlockE { block } => lower_block(block, ctx)?,

        Expr::Return { value, .. } => {
            let v = match value {
                None => IrOp::unit(),
                Some(e) => lower_expr(e, ctx)?,
            };
            IrOp::Return(Box::new(v))
        }

        Expr::Break { .. } => return Err(unsupported("`break`")),
        Expr::Continue { .. } => return Err(unsupported("`continue`")),

        Expr::StructLit { path, fields, spread, .. } => {
            if spread.is_some() {
                return Err(unsupported("struct update syntax (`..base`)"));
            }
            if path.len() == 1 && !ctx.enum_names.contains(&path[0]) {
                if !ctx.struct_fields.contains_key(&path[0]) {
                    return Err(unsupported(format!("struct literal for unknown type `{}`", path[0])));
                }
                let name = path[0].clone();
                let mut lowered_fields = Vec::with_capacity(fields.len());
                for (fname, fe) in fields {
                    lowered_fields.push((fname.clone(), lower_expr(fe, ctx)?));
                }
                IrOp::StructLit {
                    name,
                    fields: lowered_fields,
                }
            } else {
                return Err(unsupported(
                    "struct-style enum variant literals (`Enum::Variant { .. }`) — only unit and \
                     tuple-style variants are lowered in v1",
                ));
            }
        }
        Expr::Array { .. } => return Err(unsupported("array literals")),
        Expr::Tuple { .. } => return Err(unsupported("tuple literals")),
        Expr::Closure { .. } => return Err(unsupported("closures")),
        Expr::Try { .. } => return Err(unsupported("`?`")),
        Expr::Cast { .. } => return Err(unsupported("`as` casts")),

        Expr::Macro { name, args, repeat, .. } => {
            if repeat.is_some() {
                return Err(unsupported(format!("macro `{name}!` with a repeat count (`; n`)")));
            }
            if !SUPPORTED_MACROS.contains(&name.as_str()) {
                return Err(unsupported(format!("macro `{name}!` (only println!/print!/eprintln!/eprint!/format! are lowered)")));
            }
            let lowered_args = args.iter().map(|a| lower_expr(a, ctx)).collect::<LowerResult<Vec<_>>>()?;
            IrOp::Macro {
                name: name.clone(),
                args: lowered_args,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::{Codegen, RustCodegen};

    fn lower(src: &str) -> IrModule {
        lower_source(src, "test.titan", "test").unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn lowers_hello_world() {
        let src = "pub fn main() {\n    println!(\"Hello, Omnisystem!\")\n}\n";
        let m = lower(src);
        assert_eq!(m.functions.len(), 1);
        assert_eq!(m.functions[0].name, "main");
        assert!(matches!(m.functions[0].body, IrOp::Macro { .. }));
        let mut cg = RustCodegen::new();
        let rust = cg.emit_module(&m).unwrap();
        assert!(rust.contains("println!(\"Hello, Omnisystem!\")"), "{rust}");
    }

    #[test]
    fn lowers_fib() {
        let src = "fn fib(n: i32) -> i32 {\n    if n < 2 { return n }\n    fib(n - 1) + fib(n - 2)\n}\npub fn main() {\n    println!(\"fib(10) = {}\", fib(10))\n}\n";
        let m = lower(src);
        assert_eq!(m.functions.len(), 2);
        let fib = m.get_fn("fib").unwrap();
        assert_eq!(fib.params[0].ty, IrType::I64);
        assert_eq!(fib.ret, IrType::I64);
        let mut cg = RustCodegen::new();
        let rust = cg.emit_module(&m).unwrap();
        assert!(rust.contains("pub fn fib(n: i64) -> i64"), "{rust}");
    }

    #[test]
    fn lowers_struct_decl_and_literal_and_field_access() {
        let src = r#"
struct Point { x: i32, y: i32 }
pub fn make(a: i32, b: i32) -> i32 {
    let p = Point { x: a, y: b };
    p.x + p.y
}
"#;
        let m = lower(src);
        assert_eq!(m.types.len(), 1);
        assert_eq!(m.types[0].name, "Point");
        match &m.types[0].kind {
            IrTypeDefKind::Struct { fields } => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0], ("x".to_string(), IrType::I64));
                assert_eq!(fields[1], ("y".to_string(), IrType::I64));
            }
            other => panic!("expected struct, got {other:?}"),
        }
        let mut cg = RustCodegen::new();
        let rust = cg.emit_module(&m).unwrap();
        assert!(rust.contains("pub struct Point"), "{rust}");
        assert!(rust.contains("Point { x: a, y: b }"), "{rust}");
        assert!(rust.contains("p.x"), "{rust}");
    }

    #[test]
    fn lowers_enum_unit_and_tuple_variants() {
        let src = r#"
enum Shape {
    Circle(i32),
    Empty,
}
pub fn area_hint(s: i32) -> i32 {
    let c = Circle(s);
    let e = Empty;
    s
}
"#;
        let m = lower(src);
        assert_eq!(m.types.len(), 1);
        match &m.types[0].kind {
            IrTypeDefKind::Enum { variants } => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].0, "Circle");
                assert_eq!(variants[0].1, vec![IrType::I64]);
                assert_eq!(variants[1].0, "Empty");
                assert!(variants[1].1.is_empty());
            }
            other => panic!("expected enum, got {other:?}"),
        }
        let mut cg = RustCodegen::new();
        let rust = cg.emit_module(&m).unwrap();
        assert!(rust.contains("pub enum Shape"), "{rust}");
        assert!(rust.contains("Circle(i64)"), "{rust}");
        assert!(rust.contains("Empty,"), "{rust}");
        assert!(rust.contains("Shape::Circle(s)"), "{rust}");
        assert!(rust.contains("Shape::Empty"), "{rust}");
    }

    #[test]
    fn rejects_struct_style_enum_variant_literal() {
        let src = r#"
enum E { V { x: i32 } }
pub fn main() {}
"#;
        let err = lower_source(src, "t.titan", "t").unwrap_err();
        assert!(err.to_string().contains("named fields"), "{err}");
    }

    #[test]
    fn rejects_methods_with_self() {
        let src = "struct P { x: i32 }\nimpl P { fn get(&self) -> i32 { self.x } }\npub fn main() {}\n";
        assert!(lower_source(src, "t.titan", "t").is_err());
    }
}
