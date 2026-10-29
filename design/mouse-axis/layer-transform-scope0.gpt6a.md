---
type: Design
title: Individual cells and blanket layer operations
description: Focused addendum selecting family-local blanket defaults with per-cell replacement and explicit null exclusions, without multiplying operations within one layer.
resource: /design/mouse-axis/layer-transform-scope0.gpt6a.md
status: draft
author: openai/gpt-6-astra#high
date: 2026-10-29
sources:
  - resource: opencode://ses_e7a5dc784ffe94skhw7QeesP8l
    title: Independent scope refinement defining blanket defaults, cell exceptions, inherited operands and null exclusions
  - resource: opencode://ses_e7b7b7559ffeEC9DfB7nsFmtYO
    title: Parent commissions this design-only addendum after the human requests both individual-cell and blanket application
  - resource: /design/mouse-axis/layer-transform-ask0.gpt6a.md#cell-and-blanket-scope
    title: Human scope clarification and questions, read after commit 0f376ce1
  - resource: /design/mouse-axis/layer-transform-design0.gpt6a.md
    title: Original independent proposal and its sparse scalar, null and one-operation rules
  - resource: /design/mouse-axis/README.md
    title: Maintained reading map distinguishing proposed behavior and the new scope question
---

# Individual cells and blanket layer operations

## Recommendation

**Keep individual-cell edits, and add one blanket default per parameter family.**
A specific cell operation replaces that blanket for the cell; explicit `null`
excludes the cell from the blanket. Both operations use the value inherited
**before this layer**, so each scalar receives at most one operation per layer.

The [original proposal](layer-transform-design0.gpt6a.md) already supports cell
edits and output-row gains. This addendum answers the
[new whole-matrix scope request](layer-transform-ask0.gpt6a.md#cell-and-blanket-scope).
All syntax below is **prospective design**, not accepted or implemented behavior.
The original document remains intact; the amendments below apply if this
refinement is selected. Layer order and runtime boundaries are otherwise as
proposed there.

## One small selection rule

For a parameter slot `s`, let `p[s]` be its value after lower layers have resolved.
Select an operation, then apply it once:

```text
selected[s] = explicit cell operation, if present (including null)
              otherwise this family's blanket operation, if present
              otherwise null

result[s] = selected[s](p[s])
```

Here `null(p)=p`, `set(n)(p)=n`, `add(d)(p)=p+d`, and `mul(k)(p)=p*k`.
A bare number remains shorthand for set. No cell first receives the blanket
and then its exception. Declaration order within the layer has no effect.

| Cell declaration | No blanket | With blanket `(mul 2)` |
| --- | --- | --- |
| Omitted | Inherit `p` | Use blanket: `2*p` |
| `null` | Inherit `p` | Exclude from blanket: inherit `p` |
| `(set 7)` or `7` | 7 | 7 |
| `(add 1)` | `p+1` | `p+1`, not `2*p+1` |
| `(mul 0.5)` | `0.5*p` | `0.5*p`, not `0.5*(2*p)` |
| `0` | 0 | 0, not inheritance |

Thus omission means **no specific exception**, while explicit null means
**this layer must leave this slot unchanged**. Without a blanket they coincide,
preserving every sparse example in the original proposal. A higher layer's null
does not undo a lower layer's blanket: its inherited `p` already contains that
lower layer's result. Release still recomputes from the baseline and surviving
layers, never by arithmetic undo.

## Prospective configuration and a 2×2 example

Use `*` only as a family-level default key. In `coefficients`, its value is an
operation on every cell, not another output row containing input terms:

```lisp
;; Prospective contents of one layer's mouse-axis option:
mouse-axis (
  coefficients (
    (* (mul 2))
    (x (y (set 7)))
    (y (x (add 1)) (y null))
  )
)
```

Consider this inherited X/Y submatrix, with output rows and input columns:

```text
         input x  input y
output x    1        2
output y    3        4
```

| Cell | Inherited value | Selected operation | Result |
| --- | ---: | --- | ---: |
| `C[x,x]` | 1 | Blanket `(mul 2)`; no exception | 2 |
| `C[x,y]` | 2 | Explicit `(set 7)` replaces blanket | 7 |
| `C[y,x]` | 3 | Explicit `(add 1)` replaces blanket | 4 |
| `C[y,y]` | 4 | Explicit `null` excludes cell | 4 |

The resulting submatrix is `[[2,7],[4,4]]`, not the blanket-only `[[2,4],[6,8]]`.
Changing the `C[y,x]` exception to `(mul 0.5)` produces `[[2,7],[1.5,4]]`:
the inherited operand is **3**, not the blanket result 6. The null exclusion
retains 4; it does not retain a post-blanket 8 or reset the diagonal to 1.

The example displays a submatrix for readability. The wildcard also doubles
every other supported coefficient without an exception, including V/H and
Linux-native channels; it is not implicitly restricted to X/Y. With no blanket,
the original `coefficients ((x (y (mul 0.5))))` still edits exactly one cell.

## Families remain separate

The wildcard belongs to the field containing it:

| Declaration | Parameter domain |
| --- | --- |
| `coefficients ((* (mul 2)))` | Every supported routing coefficient `C[o,i]`, or directional branch cell |
| `output-gain ((* (mul 2)))` | Every supported output gain `G[o]` |

For example, `output-gain ((* (mul 0.5)) (v null) (h null))` halves every output
gain except V/H. It leaves coefficient parameters unchanged. The existing
`output-gain ((x (mul 0.25)) (y (mul 0.25)))` remains the more narrowly scoped
pointer precision example.

There is **no implicit blanket across both families**. Compilation still uses
`E[o,i]=G[o]*C[o,i]`. Explicitly multiplying both families by 2 produces a factor
of 4 in their effective product; this happens only if the configuration asks
for both. A coefficient's null exemption preserves that coefficient parameter,
not its final effective product if a separate output-gain operation changes.
To leave that product unchanged, inspect both independently declared families.

Output-gain operations are dimensionless. Blanket coefficient add/set uses the
same numeric operand in each cell's native output-unit/input-unit ratio, which
can differ across pointer, wheel and extra-axis routes. It does not perform unit
conversion. Output gain is the simpler blanket for uniform sensitivity; blanket
coefficient edits provide the explicitly requested elementwise mapping freedom.

## Wildcard domain and overlap

- The coefficient domain is **all supported outputs × all supported inputs**,
  including currently zero cells and axes absent from the physical device. It
  is not just the sparse entries written in the global map. For Linux/Android's
  ten-channel domain that is 100 linear cells; for X/Y/V/H it is 16. Unsupported
  explicit axis names remain errors, not new capabilities acquired via wildcard.
- Directional mode covers both branches of every input: 200 cells for ten
  outputs and twenty input branches, or 32 for four outputs and eight branches.
  Specific exceptions require existing directional spelling, e.g.
  `coefficients ((* (mul 2)) (v (v- null)))`. That excludes only the V-output,
  negative-V-input cell; the positive branch still receives the blanket.
- Allow one `*` entry per family. Reject duplicate blanket entries, duplicate
  output rows, and duplicate explicit cell/gain entries, including duplicates
  containing null. Blanket-plus-cell overlap is intentional selection, not a
  duplicate operation; the cell wins regardless of textual order.
- Preserve the original `(x null)` coefficient-row convenience as an explicit
  null exclusion for every supported cell in that output row. It cannot also
  have terms or appear beside another X row. A bare row `(x)` declares no cell
  exceptions, so it does **not** block a family blanket.
- No new row/column arithmetic selectors are required. In particular, reject
  nested `*` input selectors and wildcard rows containing cell lists in this
  first spelling. Row/column shorthand can be considered later; it must not be
  a prerequisite for either whole-matrix defaults or single-cell exceptions.

`(* null)` is a no-op default; explicit cell operations alongside it still work.
An absent, null or empty entire family has neither a blanket nor exceptions.
Duplicate family fields remain invalid, so `coefficients null` cannot coexist
with another `coefficients` field carrying edits.

## Zeros, signs and composition

Starting from the 2×2 identity, blanket `(mul 0.5)` yields
`[[0.5,0],[0,0.5]]`: multiplying zero off-diagonals cannot create routes. Blanket
`(add 0.1)` yields `[[1.1,0.1],[0.1,1.1]]`, creating off-diagonal routes too.
Blanket `(set 1)` makes all cells 1, **not** an identity matrix; blanket set zero
removes all coefficient contributions unless cells are exempted or overridden.

In directional mode add remains signed arithmetic: adding 0.25 to the identity
negative coefficient -1 gives -0.75. Multiply by a positive factor preserves
its output sign. Null exclusions and specific exceptions follow exactly the
same selection rule on either branch. Zero input still contributes zero motion;
coefficient addition is not a constant-motion term.

These are scalar operations applied elementwise. They are **not matrix
multiplication**, a transform pipeline, or post-mix directional selection.
The original proposal's distinction and nonlinear composition limits remain.

## Semantic amendment and implementation consequence

The original document equated omitted and null leaf entries. Refine that to:
**equal in the absence of a same-family blanket; omission selects the blanket
when present, while explicit null opts out**. Its empty-row wording likewise
means “no explicit cell operations,” not an exemption from a blanket. Explicit
row null expands to null leaves. The one-operation-per-scalar-per-layer rule is
preserved by selecting an operation before evaluating it.

A compiler can expand defaults and exceptions into the original scalar patch
representation, retaining whether each selected operation came from the blanket
or a particular cell. Explain `C[y,x]: inherited 3; cell add 1 overrides blanket
mul 2; result 4`, rather than presenting a fictitious intermediate value 6.
Resolve capture requirements after this expansion: blanket add/set can introduce
routes from every supported input, including routes not previously active.
No new runtime transform stage or cache dimension is needed beyond the compiled
configuration generation and active-layer order already proposed.

Meaningful future checks are the table above, omission versus explicit null,
declaration-order independence, duplicate rejection, directional exclusions,
zero-route creation, family independence, and expansion equivalence to a fully
enumerated one-operation patch. These are proposed checks, not executed runtime
tests. Accept the **blanket-as-default, cell-replaces, pre-layer-operand** rule
before implementation; the precise wildcard spelling can be refined separately.

The [reading map](README.md) remains parent-maintained. This addendum supplements
the [original proposal](layer-transform-design0.gpt6a.md) and the
[scope commission](layer-transform-ask0.gpt6a.md#cell-and-blanket-scope); it leaves
the deferred animation ask and broader source/endpoint designs with their
existing owners.
