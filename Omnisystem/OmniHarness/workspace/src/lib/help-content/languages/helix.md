# Helix — GPU / compute-kernel

**Absorbs the design space of:** GLSL, HLSL, WGSL, MSL, CUDA, OpenCL,
SPIR-V, compute shaders.

## Real implementation

`Omnisystem/bootstrap-helix-rs`. Binary: `helix-seed`. The genuine
uniqueness here is the type system and execution model, not punctuation:
first-class swizzled vector types (`v.xyz`, `v.rgb`), a data-parallel kernel
**dispatch** model, and vertex/fragment/compute shader **stages**.

## Verified test status (this session)

```
cargo build --release   (bootstrap-helix-rs)
./target/release/helix-seed.exe test ../bootstrap-helix/tests
→ 5/5 passed
```

Built and run in this session, immediately before this page was written.
This is the largest and most failure-mode-complete suite of the 6
non-Titan languages — it deliberately tests both of Helix's enforced
restrictions (below) as real rejections, not just happy paths.

## Real example — from the fixture suite

`Omnisystem/bootstrap-helix/tests/01_vector_math_swizzle.helix`, verbatim:

```helix
let a = vec3(1.0, 0.0, 0.0)
let b = vec3(0.0, 1.0, 0.0)
puts(dot(a, b))
puts(cross(a, b))
puts(length(vec3(3.0, 4.0, 0.0)))

let v = vec4(1.0, 2.0, 3.0, 4.0)
puts(v.xyz)
puts(v.w)
puts(v.rgb)

let n = normalize(vec3(3.0, 4.0, 0.0))
puts(length(n))
```

Expected output confirms these are numerically correct against known
values, not placeholders: `cross(x̂, ŷ) = ẑ` (`vec3(0,0,1)`),
`length(3,4,0) = 5`, and a normalized vector has length `1.0000`.

## What's genuinely *enforced*, not just implied by the domain

Two restrictions are checked by the implementation, verified by dedicated
rejection fixtures:

- **No recursion** — a call-stack check rejects a fn/kernel/shader calling
  itself (`03_recursion_rejected.helix`).
- **No dynamic loop bounds** — `for i in 0..N` requires `N` to be a literal
  integer in the source, checked against the AST node itself (not
  evaluated); `for i in 0..n` with `n` a variable is rejected with a clear
  error, while `for i in 0..5` runs correctly
  (`04_constant_loop_bound.helix` / `05_dynamic_loop_bound_rejected.helix`).

## What works today (verified against real fixtures)

- swizzled vector types (`vec2`/`vec3`/`vec4`, `.xyz`, `.rgb`, `.w`, ...)
- `dot`/`cross`/`normalize`/`length`/`mix`/`clamp`/`min`/`max`/`abs`/
  `floor`/`fract`/`pow`/`sqrt` — all numerically verified against known
  values
- `dispatch(Kernel, buffer, n)` — runs the kernel body once per thread id
  `0..n` against a shared mutable buffer. Honestly documented as a
  sequential/single-threaded simulation (there is no real GPU here), not a
  claim of actual parallel execution
- `run_stage(Shader, buffer(...))` — runs a vertex/fragment/compute stage
  once per input, collecting outputs. Verified with a real position-
  transform test; no rasterization is simulated (an intentional scope
  boundary)
- mutable shared buffers (SSBO-idiom)

## What's explicitly not implemented

- geometry, tessellation (control/eval), and mesh/task shader stages
- mat2–mat4, samplers, textures (1D/2D/3D/cube/array), images, atomics
- reflect/refract/step/smoothstep, texture sampling, derivatives (dFdx)
- workgroups + local size, shared/groupshared memory, barriers/sync, real
  atomic ops, SIMD lanes
- uniforms, push/root constants, descriptor sets/bind groups, memory
  qualifiers
- grid/block 2D/3D dispatch dims, streams
- real codegen targets — only SPIR-V header words exist (per the compiler
  plan), no full instruction-selection backend, no DXIL/Metal AIR/PTX

## In short

Helix genuinely enforces the two restrictions that make GPU-kernel
semantics honest to reason about (no recursion, no dynamic bounds), and its
vector math is numerically real. It does not generate actual GPU bytecode
and does not run in parallel — `dispatch` is a sequential simulation.
