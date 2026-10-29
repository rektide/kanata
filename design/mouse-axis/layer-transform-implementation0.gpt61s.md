---
type: ImplementationReceipt
title: Stage 1 ordered motion algebra and internal parser
description: Buildable first architectural checkpoint for typed nullable edits, numeric matrix composition, nested variable-aware parsing and focused semantic verification; runtime integration is not yet enabled.
resource: /design/mouse-axis/layer-transform-implementation0.gpt61s.md
status: draft
author: openai/gpt-6.1-sol#xhigh
date: 2026-10-29
sources:
  - resource: opencode://ses_e7a12e10affewTDCvC0unKG2wo
    title: Implements and verifies the stage 1 pure algebra and internal program parser, then records the interface checkpoint
  - resource: opencode://ses_e7b7b7559ffeEC9DfB7nsFmtYO
    title: Parent commissions Sol xhigh stage 1 and reserves runtime integration for architectural review
  - resource: /design/mouse-axis/layer-transform-implementation0.gpt6a.md
    title: Code-grounded Astra handoff and staged implementation boundary
  - resource: /design/mouse-axis/layer-transform-design1.gpt6a.md
    title: Target ordered single-map contract, nullable operands and numeric factor completion
  - resource: /design/mouse-axis/layer-transform-ask0.gpt6a.md#implementation-commission
    title: Human authorization of Sol xhigh implementation following readiness review
  - resource: /design/mouse-axis/README.md
    title: Parent-maintained reading map separating implemented global behavior from the layer implementation underway
  - resource: /parser/src/mouse_axis/program.rs
    title: Implemented pure typed algebra and owned instruction provenance
  - resource: /parser/src/cfg/mouse_axis_program.rs
    title: Implemented internal variable-aware compiler
  - resource: /parser/src/mouse_axis/program/tests.rs
    title: Pure arithmetic, order, finite-checking and stack-fold verification
  - resource: /parser/src/cfg/tests/mouse_axis_program.rs
    title: Parser semantics, restricted-domain, nested-variable and diagnostic verification
---

# Stage 1 ordered motion algebra and internal parser

## Outcome and boundary

**Stage 1 is implemented and verified, ready for the parent's interface review.**
This implements the first checkpoint in the
[Astra handoff](layer-transform-implementation0.gpt6a.md#independently-checkable-commit-sequence),
under the [current contract](layer-transform-design1.gpt6a.md), not the earlier
independent-gain or blanket-specificity alternatives.

There is **no new public configuration option yet**. Layer headers still reject
`mouse-axis`; an explicit regression test protects against accepting a program
that runtime would ignore. Existing global-map parsing and runtime behavior are
unchanged. This is not completion of the overall layer-controlled feature.

Code commits:

- `3f761d0d` — Add typed ordered motion program algebra.
- `9dade8a1` — Compile nested variable-aware motion programs with focused semantic tests.

## Files and concrete API

| File | Contribution |
| --- | --- |
| [program.rs](/parser/src/mouse_axis/program.rs) | Public pure model under `kanata_parser::mouse_axis::program`; constructors validate operands and factors before a program can own them. |
| [mouse_axis.rs](/parser/src/mouse_axis.rs) | Exposes the program module; adds `MouseAxisMap::semantic_eq` and `output_eq` without changing legacy derived equality or `reconfigure`. |
| [cfg compiler](/parser/src/cfg/mouse_axis_program.rs) | Internal `parse_motion_program(expr, vars, domain) -> cfg::Result<MotionProgram>`. |
| [cfg module](/parser/src/cfg/mod.rs) | Compiles that internal entrypoint; a documented temporary `allow(dead_code)` remains until header/runtime wiring. |
| [pure tests](/parser/src/mouse_axis/program/tests.rs) | Fourteen new algebra/stack/type tests. |
| [parser tests](/parser/src/cfg/tests/mouse_axis_program.rs) | Twenty-three new default-feature tests plus one LSP-only test. |
| [test module](/parser/src/cfg/tests.rs) | Registers the focused parser tests. |

The next stage can use these interfaces directly:

```rust
AxisDomain::native(mode)
AxisDomain::with_axes(mode, &[MouseAxis::X, MouseAxis::Y, MouseAxis::V, MouseAxis::H])
CoefficientOperand::new(domain, Vec<CellValue>)
NumericFactor::from_block(domain, from_axes, to_axes, values)
MotionProgram::new(domain, Vec<MotionInstruction>)
MotionProgram::apply(&inherited_map) // Result<MouseAxisMap, EvaluationError>
canonical_layer_order(held_newest_first, default_u16) // Vec<u16>, low first
map.semantic_eq(&other_map)
map.output_eq(&other_map, output_axis)
```

- `AxisDomain` retains the canonical ten-axis storage and arithmetic order.
  Its native selection uses `MouseAxis::supported()`. An explicit domain permits
  restricted four-axis tests on Linux without changing global platform policy.
- `Cell` names an output and `AxisInput { axis, branch }`. Branches are signed
  for linear maps, or positive/negative magnitude branches for directional maps.
- `CellValue` holds `Option<f64>`: explicit nulls remain identifiable; omitted
  cells are absent. Cell, sparse matrix and scalar syntax lower to the same
  `CoefficientOperand`. Scalar exclusions become null entries in its selection.
- `NumericFactor` is a distinct complete numeric type. `from_axes()`, `to_axes()`
  retain supplied order, and `rows()` exposes its completed canonical matrix.
  Listed rows are cleared before filling; all unlisted rows remain identity.
- `MotionOperation` is `Edit(Set | Add | MulElements, operand)`,
  `ComposeOutput(factor)` or `ComposeInput(factor)`. A program's private instruction
  list prevents mutation around constructor validation; `instructions()` offers
  read-only access for capture analysis and explanations.
- `InstructionSource` owns program-use and instruction-use locations, plus an
  optional final resolved instruction-definition location. Locations own an
  `Arc<str>` file name and zero-based byte/line/byte-column coordinates. No
  `Rc`, parser `Span`, `SExpr` or full source file is retained in the IR.
- `EvaluationError` carries an optional zero-based instruction index, operation,
  boxed owned source and `ProgramError` cause. Arithmetic failures identify the
  result cell, failing stage and product term axis. The layer resolver must add
  its own layer name/index, canonical stack and generation context.

Program equality includes provenance and is **not** a reset decision. Compare
final maps/outputs semantically. Linear comparisons ignore unused negative rows;
evaluation canonicalizes that storage to negative identity. Signed zeros are
normalized during construction/evaluation. Mode changes compare unequal for all
outputs. Legacy map equality and quantizer reconfiguration remain deliberately
unchanged for the runtime migration stage.

## Implemented semantics and validation

Each instruction reads the previous result. Null is only a no-edit marker for
that instruction, never zero or restoration. Sparse named rows do not clear
other cells; bare/null rows and empty matrices/programs are no-ops.

Products are `Q * old` for output composition and `old * P` for input composition.
They read copied old tables and evaluate terms in canonical axis order, checking
every multiplication and partial sum for finiteness. There is no intermediate
quantization, saturation, transpose or operation fusion. Directional output
composition processes both magnitude-branch tables with their existing signed
coefficients. Directional input composition is rejected at parse time and by the
pure program constructor.

The compiler uses the existing variable-aware S-expression accessors at every
nested grammar boundary, retaining LSP reference recording. A preliminary alias
chain check reports unknown/cyclic references instead of allowing recursive
accessor overflow. Programs, instructions, operation names, operands, matrix rows
and pairs, selectors/exclusions, factor fields/rows, axes and values can come
from variables. Provenance is instruction-level; the compiled IR does not retain
every nested value's separate definition or all intermediate alias locations.

Validation rejects malformed arity/shape, duplicate rows/cells/selector fields,
duplicate axes/exclusions, exclusions outside the selected Cartesian product,
empty explicit axis lists, unsupported axes, ambiguous directional branches,
factor dimension mismatches, null factor entries and nonfinite numeric literals.
Errors preserve parser source spans and instruction numbers. Literal validation
does not evaluate a dormant program against a guessed inherited map: arithmetic
validation happens when `apply` receives the actual inherited context.

The pure canonical-stack helper implements newest-held-first plus default,
deduplicate highest occurrence, then reverse. A fold test demonstrates lower-layer
release under an unchanged top, duplicate-holder reordering and repeated resolution
without drift. It does not yet read a real Keyberon layout or publish snapshots.

## Verification receipt

All commands below completed successfully on this checkpoint:

| Command | Result |
| --- | --- |
| `cargo nextest run -p kanata-parser -E 'test(mouse_axis)'` | **55 passed**, 184 unrelated tests skipped. Includes 37 new tests and 18 existing motion/global-map regressions. |
| `cargo nextest run -p kanata-parser --features lsp -E 'test(mouse_axis)'` | **56 passed**, 184 unrelated tests skipped; includes nested variable-reference recording. |
| `cargo clippy -p kanata-parser --features lsp --all-targets` | Passed; no new-code warnings. Existing warnings remain at `parser/src/gamepad/analog.rs:503–504` for negated partial-order comparisons. |
| `cargo fmt --all --check` | Passed. |
| `cargo check -p kanata-parser --target x86_64-pc-windows-gnu --all-targets` | Passed; compilation only, not Windows execution. |
| `cargo check -p kanata-parser --target wasm32-unknown-unknown --lib` | Passed; compilation only. |

Cargo also reports existing redundant-homepage manifest warnings. No versions,
dependencies or locks changed.
This receipt also passed Rekon's `check --okf` validation of document structure,
local destinations/anchors and session provenance; that is not semantic or
hardware verification.

Tests establish the design's cell/nonuniform-matrix/scalar examples, null versus
zero and omission, scope/exclusion order, precision/boost/wheel scaling of a
quarter-turn, named rectangular factor completion and row order, distinct left/
right/elementwise products, old-matrix rather than in-place products, canonical
sum order, unquantized staged equivalence followed by one final quantization,
directional negative branches, edit/product/partial-sum overflow on both product
sides, restricted native selection, owned source coordinates and `Send + Sync`
eligibility. No real input capture, uinput constructors, hooks, sudo, device
access or hardware event loops were used. No broad unchanged runtime suite was run.

## Next coherent slice and remaining obligations

**No unresolved stage 1 algebra or parser issue blocks review.** Recommended next
slice is the handoff's stage 2: selective immutable snapshot/state substrate while
keeping only the existing global map wired through it.

1. Build generation/output epochs and semantic final-output comparison around
   `output_eq`; make adapter-local observation authoritative for fraction and
   both wheel-remainder resets. Do not follow it with legacy all-map `reconfigure`.
2. Add conservative capture requirement and planned/installed capability types.
   The IR exposes null/numeric cells and factor rows; the accepted first policy
   can request all supported relative inputs for any potentially effective
   layer instruction. This receipt does not implement capture analysis.
3. After reviewing that substrate, wire typed headers/LayerInfo, constructors,
initial actual-stack resolution and per-inner-tick full-stack refresh together.
   Remove the internal parser's temporary dead-code allowance then, not by
   prematurely accepting public syntax in this checkpoint.
4. Preflight reload resolution and approved missing-capture rejection before
   live-state mutation. Preserve the parent's restart-required policy; do not
   reopen it or claim whole-reload transactionality.

Initial/default-stack checking, runtime failure context, snapshot publication,
capture installation, reload atomicity, adapter reset epochs and user-facing
configuration examples remain later commissioned work. Animation, device-qualified
sources, multiple endpoints and arbitrary directional pipelines remain outside
this bounded implementation, as recorded in the
[human commission](layer-transform-ask0.gpt6a.md#implementation-commission).
The parent maintains the [reading map](README.md); this independent receipt does
not revise Astra's design artifacts or promote prospective runtime syntax.
