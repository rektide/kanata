---
type: Design
title: Ordered cell, matrix and scalar layer transforms
description: Consolidated proposal for sparse coefficient edits, scalar broadcast and explicitly named true matrix products, with ordered layer programs and a typed boundary around directional composition.
resource: /design/mouse-axis/layer-transform-design1.gpt6a.md
status: draft
author: openai/gpt-6-astra#high
date: 2026-10-29
sources:
  - resource: opencode://ses_e7a5dc784ffe94skhw7QeesP8l
    title: Develops the independent designs and consolidates the expanded cell/matrix/scalar commission into ordered coefficient operations and typed matrix factors
  - resource: opencode://ses_e7b7b7559ffeEC9DfB7nsFmtYO
    title: Parent relays the human selection of both multiplication meanings and commissions this consolidated design-only revision
  - resource: /design/mouse-axis/layer-transform-ask0.gpt6a.md#cell-matrix-scalar
    title: Expanded human requirements, read at commission commit 533817d8
  - resource: /design/mouse-axis/layer-transform-design0.gpt6a.md
    title: Historical independent proposal supplying runtime observations and layer/state foundations
  - resource: /design/mouse-axis/layer-transform-scope0.gpt6a.md
    title: Historical blanket-default proposal whose operation-selection restriction this revision supersedes
  - resource: /design/mouse-axis/README.md
    title: Current parent-maintained reading map and standing of the design corpus
  - resource: /design/mouse-axis/linux-flexibility0.gpt6a.md
    title: Native-axis and independent source/destination scope preserved by this design
  - resource: /parser/src/mouse_axis.rs
    title: Inspected implemented linear/directional domains, native channels and quantization
  - resource: /parser/src/cfg/defcfg.rs
    title: Inspected global complete-row parser and backend axis validation
  - resource: /parser/src/cfg/layer_opts.rs
    title: Inspected icon-only layer options requiring typed extension
  - resource: /parser/src/cfg/deflayer.rs
    title: Inspected layer header parsing and indexes
  - resource: /parser/src/cfg/mod.rs
    title: Inspected LayerInfo construction and Linux device-detection derivation
  - resource: /keyberon/src/layout.rs
    title: Inspected active held/default layers and transparent-key resolution variants
  - resource: /src/kanata/mod.rs
    title: Inspected global-map publication, reload and top-layer notification hook
  - resource: /src/kanata/mouse_axis.rs
    title: Inspected output fractions and wheel remainder state
  - resource: /src/kanata/linux.rs
    title: Inspected report snapshots and asynchronous keyboard submission
  - resource: /src/kanata/windows/interception.rs
    title: Inspected startup capture filters and per-device transform state
  - resource: /src/kanata/windows/llhook.rs
    title: Inspected mouse-hook startup requirements and callback snapshot boundary
  - resource: /docs/config.adoc
    title: Implemented global maps, native units, virtual keys and generated-motion behavior
---

# Ordered cell, matrix and scalar layer transforms

## Recommendation and standing

**Use an ordered list of operations on the inherited coefficient map.** An
operation can take a **cell**, **matrix**, or **scalar broadcast** operand.
Provide `set`, `add`, and `mul-elements` for coefficient editing, and explicitly
different `compose-output` and `compose-input` operations for true matrix products.
The next operation sees the preceding result. Higher layers continue that list.

This directly supports one operation applied to a supplied nonuniform matrix,
alongside individual-cell edits and convenient blanket/selected-row scaling.
It replaces the earlier blanket-default precedence rule with ordinary sequence
order. True composition is now part of the proposed first interface, within the
algebraic boundaries stated below.

This is a **new design recommendation**, answering the expanded
[human commission](layer-transform-ask0.gpt6a.md#cell-matrix-scalar).
All layer-program syntax in this document is **prospective, not implemented or
accepted configuration syntax**. The global `mouse-axis-map`, layer actions and
virtual-key actions used as foundations already exist. No runtime changes or
runtime tests accompany this document.

The [first proposal](layer-transform-design0.gpt6a.md) and
[scope addendum](layer-transform-scope0.gpt6a.md) remain historical independent
artifacts. Their one-operation-per-cell restriction and separate gain family
were recommendations, not human constraints. This revision supersedes those
recommendations while retaining full-stack inheritance, immutable resolution,
native-axis support and explicit report/state boundaries.

## One algebra, with two operand types

Let `C` be the current coefficient map. A layer program is a finite ordered list
`F1, F2, …`; its result is `…F2(F1(C))`. Every instruction reads one complete
pre-instruction map and produces one complete new map. Matrix products must not
update rows in place and accidentally read their own partially written output.

### Coefficient edits: nullable operands

Cell, matrix and scalar forms all compile to a nullable operand table `A` over
the same coefficient domain as `C`:

| Operation | If `A[o,i]` is a number | If `A[o,i]` is null/unselected |
| --- | --- | --- |
| `set` | `C'[o,i] = A[o,i]` | `C'[o,i] = C[o,i]` |
| `add` | `C'[o,i] = C[o,i] + A[o,i]` | `C'[o,i] = C[o,i]` |
| `mul-elements` | `C'[o,i] = C[o,i] * A[o,i]` | `C'[o,i] = C[o,i]` |

**Null means this instruction does not affect that cell.** It does not mean
zero, identity, or restoration of a value from before earlier instructions.
Omitted matrix entries mean null, including entries in a row that is otherwise
specified. Numeric zero is real data: set zero removes a route; add zero leaves
it numerically equal; multiply by zero removes it.

A complete numeric matrix with `set` replaces the complete coefficient map in
one instruction. A sparse/null-valued matrix with `set` replaces only its
specified cells. Full and sparse matrices use the same operator and operand
syntax; sparsity does not change the meaning of the supplied numbers.

### True products: complete numeric factors

For a linear map with output basis `O` and physical-input basis `I`, write
`C` as an `m × n` matrix and input as a column vector `u`:

```text
motion:          y = C u
compose-output:  C' = Q C       Q: m × m, output-to-output
compose-input:   C' = C P       P: n × n, input-to-input
```

`compose-output` acts **after** the inherited mapping on motion; it is a left
product. `compose-input` acts **before** the inherited mapping on physical input;
it is a right product. `m` need not equal `n`. A factor's inner dimension and
named basis must match the side it multiplies, not an unrelated dimension.

These factors are a different type from nullable coefficient operands. Their
numbers participate in sums of products. **Do not accept null in a factor.**
Identity entries in a factor are not promises that matching final coefficients
stay unchanged. The compact factor notation below constructs a complete numeric
factor from explicitly supplied rows and identity passthrough elsewhere.

There is no ambiguous `multiply`/`mul` spelling: `mul-elements` and the two
composition names make the intended algebra explicit.

## Proposed Kanata-shaped interface

Keep the parenthesized layer header extension, but its `mouse-axis` value is now
an ordered instruction list:

```lisp
(deflayer (precision mouse-axis (
    (mul-elements (scalar 0.25 (outputs x y)))
  ))
  ;; ordinary key actions follow
)
```

The same option belongs in `deflayermap` headers. Missing/null/empty `mouse-axis`
means an empty program. Within the list, use only the following fixed forms;
there are no nested arithmetic expressions, variables holding runtime maps,
conditionals, or implicit operation-priority classes.

### Cell, matrix and scalar forms

```lisp
;; Operator first; the operand says shape and scope.
(set          (cell x y 7))
(add          (cell x y 1))
(mul-elements (cell x y 0.5))

(add (matrix
  (x (x 2) (y null))
  (y (x -1) (y 0.5))
))

(mul-elements (scalar 2))                         ;; all supported cells
(mul-elements (scalar 0.25 (outputs x y)))         ;; all inputs into X/Y
(mul-elements (scalar 2 (inputs v h)))            ;; V/H into every output
(mul-elements (scalar 2
  (outputs x y) (inputs x y) (except (x y))
))                                               ;; a block with one hole
```

- **Cell:** `(cell OUTPUT INPUT VALUE)` supplies exactly one operand cell.
  VALUE can be a finite number or `null`.
- **Matrix:** named output rows contain named input/value pairs. The operator
  appears once outside the matrix; entries are values, not per-cell operators.
  Missing rows/entries and explicit null mean no edit. A bare row `(x)` or
  `(x null)` supplies no edits to that row. Neither suppresses its output.
- **Scalar:** one finite number or `null` is broadcast over a selected Cartesian
  product of outputs and inputs. Omitted selectors mean **all** on that side;
  listed names select exactly those axes. Optional `except` lists cell pairs
  to exclude, compiling those operand cells to null. Every other selected cell
  receives the scalar; unselected cells are null.

This has no matrix “default value.” Omitted matrix entries always skip the edit.
A scalar broadcast explicitly supplies a value to all selected cells; to leave
holes use `except`, rather than adding a later null operation and expecting it
to undo the earlier broadcast. `(scalar null …)` is a no-op over its selection.
This separates **selection** from **sequence**, with no specificity-based
override between instructions.

Reject duplicate names within a matrix row, duplicate matrix rows, repeated
selectors or axis names, duplicate exclusions, and exclusions outside the
selected domain. Nonempty explicit axis lists avoid accidental empty selection;
an empty matrix is a valid no-op. Repeated edits to a cell in *different*
instructions are valid and execute in order. Reject unknown axes, malformed
arity, nested operation-valued matrix cells, NaN and infinity. Use existing
configuration-variable resolution for static values; this introduces no new
runtime expression language.

For absolute replacement with a sparsely written zero-filled map, explicitly
clear then fill: `(set (scalar 0))` followed by `(set (matrix …))`. That is
different from a sparse set matrix alone, which must preserve unspecified cells.
The existing global map's “named row replaces identity” syntax remains unchanged
and must not be reused as the parser for these sparse operands.

### Numeric factors with named from/to bases

Use one fixed factor form for either true product:

```lisp
(compose-output (factor
  (from x y)
  (to x y)
  (values (1 1)
          (0 1))
))
```

For `compose-output`, `from` names current output channels and `to` names the
result output channels. For `compose-input`, `from` names physical input channels
and `to` names channels presented to the inherited map. Both sides currently
use existing supported channel names; this is not an endpoint-creation facility.

`values` has one row per `to` axis and one column per `from` axis, in the listed
order. Every entry is a finite number; all listed entries are required. Construct
the full factor as follows:

1. Start with identity on that side's complete supported basis.
2. For each named `to` axis, clear its **entire** factor row to zero.
3. Fill that row's listed `from` columns with the supplied numeric values.

Thus unlisted factor rows are identity; unlisted columns in a listed row are
zero. This is explicit **factor construction**, not nullable coefficient editing.
The lists must be nonempty, unique on each side, and dimensionally match the
values. Their cardinalities may differ: a `to z` / `from x y` factor block is a
1×2 supplied block, completed into a full output-side factor. A full factor lists
the entire side basis; a small principal block preserves other channels without
making the user enumerate them. No implicit axis sorting may reorder its values.

The completed factor preserves the existing input/output basis and dimensions.
Arbitrary dimension-changing endpoints or intermediate channel vocabularies are
outside this first interface; independent input/output route counts and
rectangular supplied blocks are already useful without them.

## Worked coefficient examples

All 2×2 tables below show output rows X/Y and input columns X/Y. Other native
cells inherit unless the shown instruction explicitly selects them too.

### One cell, or one operation on a supplied matrix

Start each row of this table independently from `C = [[1,2],[3,4]]`:

| Instruction | Result |
| --- | --- |
| `(set (cell x y 9))` | `[[1,9],[3,4]]` |
| `(add (cell x y -0.5))` | `[[1,1.5],[3,4]]` |
| `(mul-elements (cell x y 0.5))` | `[[1,1],[3,4]]` |
| Any of those operators with `(cell x y null)` | `[[1,2],[3,4]]` |

Now supply the **single matrix operand** `A = [[2,null],[-1,0.5]]`, written as
the named-row matrix in the syntax example. Again start each case from C:

| Matrix-level operation | Result | Null cell |
| --- | --- | --- |
| `set A` | `[[2,2],[-1,0.5]]` | X←Y remains 2 |
| `add A` | `[[3,2],[2,4.5]]` | X←Y remains 2 |
| `mul-elements A` | `[[2,2],[-3,2]]` | X←Y remains 2 |

Omitting `(y null)` from A's X row has exactly the same result. Replacing null
with numeric zero would instead set X←Y to zero, leave it at 2 under add, or
multiply it to zero, respectively. A fully numeric A is a full operation on the
displayed 2×2 block; selecting the complete native domain gives a full-map
operation, with no need to put an operator inside each matrix entry.

### Scalar broadcast is not scalar times identity

For the same C, broadcasting scalar 2 over its displayed block gives:

| Operation | Result |
| --- | --- |
| `set` | `[[2,2],[2,2]]` |
| `add` | `[[3,4],[5,6]]` |
| `mul-elements` | `[[2,4],[6,8]]` |

`set (scalar 2)` fills **all coefficients**, not just the diagonal. `add` creates
routes from zero off-diagonals: identity plus broadcast 0.1 becomes
`[[1.1,0.1],[0.1,1.1]]`. Multiplying identity elementwise by broadcast 0.5 gives
`[[0.5,0],[0,0.5]]`; multiplication cannot create a route from zero.

For selected-output scaling, `(mul-elements (scalar 0.5 (outputs x)))` changes
C to `[[0.5,1],[3,4]]`. It scales every contribution to X, including off-diagonal
routes. The full-domain spelling without selectors includes wheels and extended
native channels, not an implicit X/Y-only block.

### Overlapping scopes execute, rather than override

Starting from C, one layer can intentionally overlap operations:

```lisp
mouse-axis (
  (mul-elements (scalar 2 (outputs x y) (inputs x y)))
  (add (cell x y 1))
  (set (cell y y null))
)
```

The result is `[[2,5],[6,8]]`: add sees X←Y **4**, the result of multiplication.
Null sees Y←Y **8** and leaves it there. Swapping the first two instructions
gives `[[2,6],[6,8]]`. Set X←Y to 7 after the multiplier yields 7; placing that
set before the multiplier yields 14.

To exclude X←Y from the first instruction, use its `(except (x y))` selector.
Then the following add sees inherited 2 and gives 3. A later null is not a
retroactive exclusion. This supersedes scope0's “cell exception replaces blanket
and sees pre-layer value” rule; here the program states exactly where each edit
occurs and what it reads.

Across layers the same rule applies. With lower L broadcasting ×2 and higher H
adding 1 to X←Y, that cell is 5. Release L while H remains the top layer: it
becomes 3. Re-activate L above H: it becomes 6. Release H: it becomes 4. Release
L: it returns to baseline 2. Nothing is divided/subtracted from the previous
effective map; each state is resolved afresh in its current order.

## Precision, boost and virtual-key layers

This prospective complete configuration retains the existing global quarter-turn
map and existing layer/virtual-key actions. The header programs are the new part:

```lisp
(defcfg
  delegate-to-first-layer yes
  mouse-axis-map ((x (y 1)) (y (x -1)))
)
(defsrc f1 f2 f3 f4)
(defvirtualkeys precision-control (layer-while-held precision))
(deflayer base
  (layer-while-held precision)
  (layer-while-held boost)
  (layer-while-held wheel-fast)
  (on-press toggle-virtualkey precision-control)
)
(deflayer (precision mouse-axis (
    (mul-elements (scalar 0.25 (outputs x y)))
  )) _ _ _ _)
(deflayer (boost mouse-axis (
    (mul-elements (scalar 2 (outputs x y)))
  )) _ _ _ _)
(deflayer (wheel-fast mouse-axis (
    (mul-elements (scalar 2 (outputs v h)))
  )) _ _ _ _)
```

For physical `X=8, Y=4, V=120, H=0`, before quantization:

| Active programs | X output | Y output | V output |
| --- | ---: | ---: | ---: |
| None | 4 | -8 | 120 |
| Precision | 1 | -2 | 120 |
| Boost | 8 | -16 | 120 |
| Precision + boost | 2 | -4 | 120 |
| Precision + wheel-fast | 1 | -2 | 240 |

These particular scalings commute. The precision instruction scales rows, not
diagonal cells, so it preserves the quarter-turn whose diagonal entries are zero.
A physical and virtual holder of the same precision layer apply it once, under
the duplicate-layer rule below. A virtual key provides ordinary persistent held
state; a tap may come and go without a motion report. It does not imply animation.

### Output gain convenience without a second hidden family

Recommend **one coefficient map**, with selected-output scalar multiplication as
the gain convenience. Algebraically that instruction equals left multiplication
by a diagonal output-scale factor. Selected-input scalar multiplication scales
columns; in linear mode it equals right multiplication by a diagonal input
factor. In directional mode its input selectors name rectified branches, as below.

Do not retain a separately resolved `G` and later multiply it into `C` behind the
program's back. No operation implicitly edits both gain and coefficient families.
Two explicit scaling instructions multiply twice because they are two steps;
there is no accidental second scaling at publication.

**Tradeoff:** this revises design0's independent logical output gain. A coefficient
set *after* precision now sets the actual coefficient and is not scaled again;
a set *before* precision is scaled. There is no separate inherited gain slot to
which `(add 0.1)` can add while independently preserving an arbitrary routing
factorization. Scalar add changes coefficients, not a hidden sensitivity value.
Likewise scalar set 0.25 is not “set sensitivity to 25%.” Use row multiplication
for relative sensitivity, or set the desired coefficient matrix for an absolute
map. Accepting this simpler state model is a human decision, not a claim that
it preserves every convenience of the earlier two-family proposal.

## True products, order and dimensions

### Pre-input versus post-output

Let `R = [[0,1],[-1,0]]` be the quarter-turn and `D = [[2,0],[0,1]]` scale X.
Supply D as a factor on X/Y:

```lisp
(compose-output (factor
  (from x y) (to x y)
  (values (2 0) (0 1))
))
```

On inherited R this produces `D R = [[0,2],[-1,0]]`. Input `(2,3)` yields
`(6,-2)`. Changing just the operator to `compose-input` produces
`R D = [[0,1],[-2,0]]`, yielding `(3,-4)`. One scales the rotated output X;
the other scales physical input X before rotation. Neither is an elementwise
multiply by D, which would erase R's off-diagonal routes and yield zero.

Sequence order matters for products too. Starting from identity, an output
composition with R followed by output composition with D yields `D R`; reversing
those two instructions yields `R D`, with the same different outputs above.
For right products, consecutive input factors P1 then P2 compile to `C P1 P2`:
physical motion encounters P2 first. Instructions are applied to the **current
map** in textual order; they are not a textual runtime event pipeline.

### Rectangular routing and a rectangular supplied factor block

Suppose a useful portion of the map routes two inputs V/H to three outputs X/Y/Z:

```text
C (3 × 2) = [[1,0],
             [0,1],
             [1,1]]
```

Use `(compose-output (factor (from x y) (to z) (values (2 3))))`.
The supplied block is 1×2; its completed output factor on these three axes is
`Q = [[1,0,0],[0,1,0],[2,3,0]]`. Then
`Q C = [[1,0],[0,1],[2,3]]`, still 3×2. With V/H `(2,4)`, displayed X/Y/Z
changes from `(2,4,6)` to `(2,4,16)`. Remaining native channels retain the
behavior of their unlisted identity factor rows.

There is no equal-input/output-count requirement. The actual backend retains
its full named channel tables, including zero routes; the displayed rectangular
map is its independently selected input/output relation. Output factors use the
output basis, input factors the input basis. Do not size either factor from the
number of nonzero routes or confuse output X with input X solely because their
names are spelled alike.

### Why factors reject null

For inherited `C = [[1,2],[3,4]]`, the factor
`Q = [[1,1],[0,1]]` gives `Q C = [[4,6],[3,4]]`. Its Q[X,X] entry is 1, but
the result C[X,X] changes from 1 to 4 because another term contributes.
Right multiplication gives `C Q = [[1,3],[3,7]]`; even Q[Y,Y]=1 does not
preserve result C[Y,Y]=4.

It would therefore be false to interpret a null factor cell as “leave the
corresponding inherited result coefficient unchanged” merely by substituting
0 or an identity entry. **Null remains a no-edit marker only in coefficient
operands; factors require numbers.** Factor row identity/zero completion is
explicitly different syntax with different semantics. An unlisted output-factor
row guarantees that entire output row is preserved by a left product. There is
no general per-result-cell preservation promise for input-side factors.

Freezing arbitrary result cells while taking a product for the rest would be a
separate masked-result operation, not this pure product. It is not in the first
interface. Use coefficient patches for sparse no-change editing, and factors
when the entire declared linear composition is wanted. This typed distinction
is another consequential decision to accept before implementation.

## Directional maps and the supported product boundary

The implemented directional representation has `m` output rows and `2n` columns:
one positive-magnitude branch per input sign. With
`phi(u) = (max(u[i],0), max(-u[i],0))` for each input, motion is `y = H phi(u)`.
Coefficient operands address columns as `x+`, `x-`, `v+`, etc. Output names stay
unsuffixed. A scalar input selection must list branches explicitly; omitting the
input selector selects all branches. The mode remains configuration-wide.

For X/Y directional identity, in column order X+, X−, Y+, Y−:

```text
H = [[1,-1,0, 0],
     [0, 0,1,-1]]
```

**Linear output composition is fully supported:** for numeric Q,
`Q(H phi(u)) = (Q H) phi(u)`. The shear Q above produces
`[[1,-1,1,-1],[0,0,1,-1]]`. Input `(-2,3)` produces `(1,3)`.
A subsequent `(set (cell x x- 0.5))` changes that to
`[[1,0.5,1,-1],[0,0,1,-1]]`, giving `(4,3)`. The negative input branch still
supplies positive magnitude 2; its explicit coefficient chooses output sign.

This remains an `m × 2n` map: multiply an `m × m` factor on its left. Do not
pretend it is an ordinary `m × n` signed-input matrix, or a square 2n-output map.

**Recommend rejecting `compose-input` in directional mode in the initial
interface.** Physical pre-mixing asks for `H phi(P u)`, generally not representable
as `H' phi(u)`. For example, a directional output `abs(x)` preceded by
`x := x+y` becomes `abs(x+y)`. Its output is 1 on `(1,0)`, 1 on `(0,-1)`, but
0 on `(1,-1)`. Any one-stage independently rectified-original-input map that
matches the first two must sum them to 2 on the third.

Certain pre-transforms, such as nonnegative diagonal scaling, can be compiled
by adjusting branch columns; the scalar/branch coefficient operations already
express those directly. This proposal does not quietly reinterpret a general
physical-input factor as a 2n×2n branch-space factor. Arbitrary pre-mixing and
composition of directional maps require explicit runtime stages and their own
units/quantization/state policy. They remain a future boundary, while true
products on both sides of linear maps and the output side of directional maps
are delivered by the recommended first interface.

## Layer inheritance, units and no drift

Retain the full-stack contract from design0:

1. Read `active_held_layers()` newest first, append `default_layer`.
2. Deduplicate layer IDs, keeping the first/highest-priority occurrence.
3. Reverse the list. Start from the immutable global map, then run each layer's
   instruction list in order, lowest-priority layer first.

The newest surviving activation sets a held layer's position; multiple holders
do not multiply its program instances. For states oldest-first `L,M,L`, resolve
M then L; releasing the newest L resolves L then M. A held occurrence of the
selected default wins over its lower default occurrence, so that layer runs once.
Without a held occurrence, default is the lowest layer. No implicit first-layer
fallback is inserted. Missing/null programs are empty, not identity resets.

This is independent of Keyberon's older/newer transparent-key lookup variants
and `delegate-to-first-layer`: those govern keyboard resolution, not motion
parameter inheritance. Lower-layer release, duplicate-holder reordering, or
default-layer change may affect motion even if `current_layer()` is unchanged.
Virtual and one-shot layer states participate while exposed by the held iterator.

Never mutate the previously published map to apply a new activation. Resolution
is a pure function of configuration generation and the complete canonical stack.
Input reports do not rerun instructions against the last result. Layer release
rebuilds; it never attempts to invert a set, zero multiplier or singular factor.
Reload uses the new configuration and its actual resulting layout state, not old
layer indexes or a promise of layer-state migration. Identical logical state has
the same deterministic fold, with no cumulative multiplication drift.

Preserve the full backend-supported channel vocabulary: X/Y/V/H and Linux/Android
Z, RX/RY/RZ, dial and misc. Selection includes supported but currently unused
channels. Linear scalar-all covers the whole output×input table; directional
scalar-all covers output×signed-branch columns. Reject unsupported named axes
in inactive programs as well as active ones. This does not add macOS capture.

Coefficient set/add values have output-unit/input-unit ratios; multiply operands
are dimensionless. A factor entry has its `to` unit per `from` unit. Pointer and
extra axes use native counts; wheel units are 1/120 notch. A scalar set/add applies
the same number in each selected cell's respective units, not a universal physical
quantity. There is no implicit pointer/wheel unit conversion in factor products.

All these transforms remain homogeneous in physical input: zero input contributes
zero output, including after coefficient add. Changing the map emits no motion.
Generated `movemouse-*`/`mwheel-*` output remains outside the physical-input map.
Report aggregation and wheel-companion normalization precede branch selection.

## Runtime and parser consequences

### Existing ground versus proposed extension

The inspected [parser model](/parser/src/mouse_axis.rs) already stores a global
linear/directional map and performs output-local fractional quantization. The
[layer option parser](/parser/src/cfg/layer_opts.rs) currently accepts atom-valued
icons, not this program. [Layer headers](/parser/src/cfg/deflayer.rs) and
[LayerInfo construction](/parser/src/cfg/mod.rs) are concrete extension points
for a typed immutable program table. Existing global complete-row parsing in
[defcfg.rs](/parser/src/cfg/defcfg.rs) remains a separate baseline compiler.

Represent coefficient-edit operands and numeric factors as distinct validated
types. Compile selector/exclusion syntax to operand masks and factor syntax to
complete numeric factors. Keep each instruction's source span and ordered index.
No new Keyberon action is needed. The existing runtime's top-layer hook is not
enough; compare the whole canonical stack after layout/custom-action processing,
external default-layer changes, and successful reload.

### Coherent publication and capture

Resolve once per changed `(config_generation, canonical_layer_order)` into a
new map, then publish one immutable snapshot using the existing mutex mechanism
or an equivalent whole-snapshot swap. Intermediate instruction results are not
published and do not trigger input-state resets. Validate finite arithmetic at
each step, including product sums; finite literals alone do not prevent overflow.
Reject initial/reloaded resolution errors before publication and propagate an
invalid newly active combination through the processing error boundary with
its instruction/stack context. Do not silently clamp parameters or publish NaN.
Retain the distinct existing saturation policy for extreme motion outputs.

Use fixed axis ordering for deterministic arithmetic; do not reorder or fuse
instructions merely because they are algebraically equivalent in real numbers.
Products and elementwise operations need no runtime per-report stage: compile
them to the effective map before native motion processing.

Each native report/stroke/callback takes one snapshot. Its contract is the latest
**fully processed layout state at snapshot acquisition**. The inspected
[Linux loop](/src/kanata/linux.rs) submits keyboard events asynchronously and
transforms reports before their keyboard processing, so this is not strict
physical key-arrival ordering. A motion-produced key tap changes only later
snapshots. Strict ordered input envelopes/barriers would need a separate runtime
commission; do not imply that an atomic map swap supplies them.

Capture requirements must include inactive programs and products. Form a
conservative union of possible routes across all declarations, ignoring cancellation
and route-removing edits when computing reachability. Set/add can create edges;
elementwise multiplication cannot create one from zero. Close possible routes
under output-factor and input-factor multiplication: an output factor can make
another row depend on existing sources, and an input factor can introduce new
physical-source dependencies into inherited columns. Repeat conservatively until
stable rather than inspecting only each layer against the global baseline.

Include sources whose identity route can be suppressed or altered, not just
nonzero fan-out. If the first implementation cannot establish a smaller safe
set for composition, capture all supported relative inputs whenever composition
is declared, and make that conservative choice visible. Never miss an inactive
combination because it happens to be identity at startup. Feed requirements into
Linux device detection and the inspected Windows
[Interception](/src/kanata/windows/interception.rs) and
[LLHOOK](/src/kanata/windows/llhook.rs) startup gates. Respect explicit device
selection; diagnose reloads needing capabilities absent from installed capture.

### Rounding, discontinuities and explanations

Keep one final quantization per output after summing all physical contributions.
Do not quantize intermediate matrix products or per-cell routes. On a discrete
layer change, compare final effective rows and reset only changed outputs'
fractional state; for V/H reset their associated raw/key notch remainders too.
Directional comparison includes both branch rows. Unchanged wheel rows retain
credit when only pointer precision changes. Full successful reload resets all
state explicitly, even if numeric maps match; release restores settings, not
historical fractional credit. This is a proposed refinement of today's all-map
change reset in [AxisState](/src/kanata/mouse_axis.rs).

Publish per-output reset epochs so processed A→B→A transitions between input
reports cannot retain stale A credit merely because the next map equals the
last one seen by an adapter. A sequence of instructions that temporarily visits
B while resolving one unpublished final A is not such a transition. Identity
passthrough paths must still observe reset epochs. Equal final numeric rows with
different provenance do not reset, except at the explicit reload boundary.

Explain results by stack, instruction order, operand source and intermediate
coefficient values. For products, retain the factor/source-map dependency relation
so a requested cell explanation can show its sum of contributions. Do not pretend
it has one winning cell declaration. Generate detailed traces on demand from the
immutable program and stack; callback snapshots need only lightweight references,
effective rows and epochs. Normalize signed zero for stable comparisons and reject
NaN before comparison.

## Deferred animation

Keep the [parent's animation desire](layer-transform-ask0.gpt6a.md#animation-desire)
as the maintained ask. Scalar operands, selected matrix values and factor
parameters offer future animation targets. A sample must re-resolve from the
baseline and current sampled operands, not repeatedly apply a multiplier to the
previous effective map. Source-bound instruction identities can identify targets
without reviving a hidden gain family.

Continuous samples will need a different fractional-state policy from discrete
layer discontinuities; resetting on every update can starve small movement.
Factor interpolation need not preserve rotations. Retargeting, sustained-input
triggers, observation point, time/distance driver and inactivity behavior remain
deferred decisions, not implemented or settled by this algebra.

## Next coherent implementation slice and checks

After semantic acceptance, implement the whole bounded contract: typed ordered
program parsing; pure coefficient/matrix/scalar edits; both products for linear
maps and output products for directional maps; full-stack runtime publication;
capture derivation; and explicit per-output reset epochs. Add native configuration
examples for precision, matrix-valued edits and composition. Preserve existing
global-only behavior. This slice requires no generic expression engine or runtime
transform-stage pipeline.

Meaningful checks should establish:

- The numeric examples above, including nonuniform matrix values, null/omission,
  scalar exclusions, zero off-diagonals, full versus sparse set, overlapping
  instructions and release under an unchanged top layer.
- Equivalence of cell/matrix/scalar operands when they denote the same masked
  values; intentional non-equivalence of elementwise and true products.
- Factor dimensional/name validation, rectangular block completion, and equality
  of compiled-product motion to unquantized staged linear calculation followed
  by one quantization. Do not compare to independently rounded stages.
- Directional post-product equality for both input signs, negative branch signs,
  and rejection of null factors or unsupported physical pre-composition.
- Duplicate layer holders/reordering, both keyboard transparent-resolution variants,
  no repeated-tick drift, finite-arithmetic failures and reload generation handling.
- Inactive/cross-layer product capture dependencies, one-snapshot reports,
  processed-state activation responsiveness on actual backends, untouched-output
  remainder preservation and A→B→A reset epochs.

These are recommended implementation checks, not runtime verification receipts
for this proposal. Document checking establishes structure/provenance, not backend
behavior or human acceptance.

## Decisions before implementation

1. **Ordered single-map model:** cell, matrix and scalar edits read preceding
   results; retain explicit set/add/`mul-elements` and both named true products.
   Accept replacing blanket specificity and independently inherited output gains
   with visible instruction order and selected-row scaling.
2. **Typed null boundary:** sparse coefficient operands preserve current cells;
   scalar exclusions create holes; numeric factors use explicit row completion
   and reject null rather than falsely promise result-cell preservation.
3. **Bounded first composition support:** linear pre/post and directional post
   compile into one map; arbitrary directional pre-mixing/stages remain explicit
   future work. Retain full-stack/processed-state publication and selective
   discrete-reset semantics, with animation separately deferred.

The [reading map](README.md) belongs to the parent session. This revision connects
the historical [design0](layer-transform-design0.gpt6a.md) and
[scope0](layer-transform-scope0.gpt6a.md) to the expanded commission without
rewriting their evidence. The [Linux flexibility proposal](linux-flexibility0.gpt6a.md)
still owns source-qualified and endpoint expansion; these coefficient operations
do not depend on that separate work. Implemented behavior remains documented in
the [configuration reference](/docs/config.adoc).
