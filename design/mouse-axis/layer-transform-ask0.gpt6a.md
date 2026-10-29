---
type: Request
title: Layer-controlled input transforms and deferred animation desire
description: Human requirements for cell, matrix and scalar layer operations, sparse inheritance, distinct elementwise and true matrix multiplication, and deferred transform animation.
resource: /design/mouse-axis/layer-transform-ask0.gpt6a.md
status: draft
author: openai/gpt-6-astra#xhigh
date: 2026-10-29
sources:
  - resource: opencode://ses_e7b7b7559ffeEC9DfB7nsFmtYO
    title: Human commissions Sol xhigh after Astra notes; parent reviews readiness, approves capture-incompatible reload rejection and dispatches the typed algebra/parser checkpoint
  - resource: /parser/src/mouse_axis.rs
    title: Existing global linear and directional mappings and quantization state
  - resource: /src/kanata/mouse_axis.rs
    title: Current transform replacement resets fractional and wheel remainder state
  - resource: /keyberon/src/layout.rs
    title: Active held layers, default layer and transparent-key resolution order
  - resource: /parser/src/cfg/layer_opts.rs
    title: Existing layer option parsing currently supports icons
  - resource: /src/kanata/mod.rs
    title: Current layer-change detection, tick processing and global mouse-axis map publication
  - resource: /design/mouse-axis/linux-flexibility0.gpt6a.md
    title: Existing distinction between generated axes, source identity and output endpoints
  - resource: /design/mouse-axis/layer-transform-design0.gpt6a.md
    title: Completed independent Astra-high response to this commission, pending selection of its proposed semantics
  - resource: /design/mouse-axis/layer-transform-scope0.gpt6a.md
    title: Focused response to the blanket-operation request, recommending cell replacement and explicit null exclusions
  - resource: /design/mouse-axis/layer-transform-design1.gpt6a.md
    title: Consolidated response proposing ordered edit operands, numeric composition factors and a single-map layer program
  - resource: /design/mouse-axis/layer-transform-implementation0.gpt6a.md
    title: Astra's concrete readiness assessment and implementation handoff
---

# Layer-controlled input transforms and deferred animation desire

## Commission and standing

The human wants a layer that makes physical pointer movement slower or faster,
with the same expressive freedom available to wheels and other supported
relative channels. The larger question is how layers affect a transform:
inherit settings, supersede them, compose transforms, or adjust individual
matrix elements.

The latest direction is **design first, with Astra high**. Inherited overrides
are promising, with explicit null values and three arithmetic cases:

> “an inherited multiplier too, so either addition or multiplication. or
> absolute! those are kind of the three core cases”

This document preserves the evolving ask and rough animation ideas. The initial
commission was design-only; the later [implementation commission](#implementation-commission)
authorizes the next step subject to a readiness review. Proposed configuration
syntax remains distinct from available runtime behavior until implementation.

The commissioned [Astra-high response](layer-transform-design0.gpt6a.md) is now
available. It recommends independent output gains and routing coefficients,
sparse scalar operations, a full active-layer resolver, and explicit report and
fractional-state boundaries. Its worked examples answer this ask; its
[decision section](layer-transform-design0.gpt6a.md#decisions-to-accept-before-implementation)
identifies the choices to select before implementation.
The [latest scope direction](#cell-matrix-scalar) below expands that proposal
and its blanket addendum; earlier recommendations to defer true composition
are not constraints on the revised design.

## Vocabulary and desired freedoms

Use **input transform** or **motion transform** for the overall operation.
An **axis/channel** is one component such as X, Y, V, H or Linux Z; the human's
use of “axis” includes pointer X/Y and is not restricted to wheels.

Distinguish:

- **Routing coefficients:** how input components contribute to output components.
- **Gains:** multipliers on a channel or direction; useful for precision/boost.
- **Layer resolution:** how active declarations produce the effective settings.
- **Transform composition:** applying one complete transform after another.
- **Animation:** changing effective parameters with time or activity.

Desired layer operations, with spelling still open:

| Operation | Intended question |
| --- | --- |
| Omitted / null | Leave this particular entry untouched; a layer need not affect or specify every mapping element. |
| Absolute / set | Can this layer choose a value independent of lower layers? |
| Multiply inherited | Can this layer scale whatever value it inherits? |
| Add to inherited | Can this layer offset an inherited parameter by a chosen amount? |

Null should remain distinct from numeric zero, an identity transform, and an
empty output row. In the existing global mapping, an omitted row is identity
and an empty row suppresses its output. New layer semantics need their own
explicit inheritance contract, rather than accidentally reusing those meanings.

**Human clarification:** by null, the user means sparse **per-element** edits:
“not have to affect every single element in the mapping.” A layer must be able
to adjust one coefficient or gain while all omitted/null entries keep the values
they would otherwise inherit. Null is a no-op at that entry, not a request to
zero an entry, reset it to identity, or replace an entire row/profile. The design
must support this granularity even if it also offers convenient row/group edits.

Addition here concerns a **parameter** (gain or coefficient), not automatic
constant pointer movement when input is zero. The design should state this
distinction and consider how user-facing terminology keeps it understandable.

<a id="cell-and-blanket-scope"></a>

### Individual cells and blanket operations

The human subsequently asked whether the proposal was only a blanket apply,
clarifying: **individual cells are required, and blanket application is desired
too**. Both scopes belong in the design.

The first proposal already covers individual coefficient edits. Its prospective
`coefficients ((x (y (mul 0.5))))` changes only the Y-input to X-output cell;
other entries in that row inherit unchanged. Its independent `output-gain`
parameter can scale an entire output row without enumerating that row's cells.

An explicit whole-matrix blanket operation was not specified in the first
proposal. The next design refinement should make bulk scope usable alongside
sparse cell operations, without requiring repetition of the same operator in
every cell. Keep parameter scope explicit: blanket coefficient edits and
blanket output-gain edits are different operations and should not accidentally
apply the same scaling twice.

Resolve these interactions explicitly:

- Which entries a blanket operation covers, including directional branch cells.
- Whether a specific cell operation replaces the blanket operation for that
  cell, or is applied after it; define what value add/multiply inherits.
- How an explicit null cell can preserve the user's no-change intent when a
  surrounding blanket edit exists, and how omission differs in that context.
- Duplicate/overlap rules and whether a cell can be affected more than once
  within a single layer.

The operation still matters: adding to all coefficients can create routes from
zero off-diagonal cells; multiplying all coefficients preserves those zeros.
These are parameter operations, not full matrix multiplication.

The resulting [Astra-high scope addendum](layer-transform-scope0.gpt6a.md)
recommends selecting one operation per cell per layer: explicit cell operation
(including null), otherwise family blanket, otherwise inheritance. The selected
operation acts on the pre-layer inherited value. Under this proposed refinement,
omission accepts a blanket while explicit null exempts the cell. This is the
recommended answer to the questions above, pending human selection.

<a id="cell-matrix-scalar"></a>

## Updated scope: cell, matrix and scalar

After reviewing the cell/blanket addendum, the human asked to put the operation
on the whole matrix rather than attach an operator to each individual cell.
When asked to distinguish coefficient-wise multiplication from true matrix
multiplication, the human explicitly selected **both, explicitly named**, then
expanded the desired forms again:

> “maybe all three: cell, matrix, scalar”

The current design commission therefore includes all three operand forms:

- **Cell:** a targeted coefficient operation.
- **Matrix:** one operation with a matrix of operand values, including sparse
  entries where mathematically meaningful. It must be possible to request a
  matrix-level replacement or addition directly.
- **Scalar:** one numeric operand applied over a clearly selected scope,
  providing the blanket case without repeating a value in every cell.

Keep operation and operand form separate. Replacement/set, addition and
coefficient-wise multiplication are distinct from genuine transform composition
by matrix product. Explicit names must make the two multiplication meanings
unambiguous. The desired forms coexist; the latest direction is not a request
to remove cell-level control.

Request a consolidated Astra-high revision rather than treating the earlier
blanket-as-default syntax as the final interface. Define how operations at these
scopes interact within a layer and across inherited layers. The earlier
one-operation-per-cell rule was a recommendation for that smaller interface,
not a human restriction that should silently prevent the new matrix operations.

Important questions for the revision:

- Whether cell/matrix/scalar operations use a sequential program, scoped
  overrides, or a small explicit combination; define inherited operands and
  avoid accidental double application.
- Scalar broadcasting versus diagonal/identity-scaled matrices, especially for
  addition and replacement. State the coefficient domain affected by a scalar.
- Sparse/null semantics for coefficient edits versus a matrix used as a
  composition factor. A neutral entry in a factor does not guarantee that the
  corresponding final inherited coefficient stays unchanged; preserve the
  user's sparse no-change intent without making that false promise.
- True composition order (left/post versus right/pre), named input/output
  dimensions, and the current rectangular directional coefficient domain.
  Distinguish products that can be represented by existing coefficients from
  directional stage compositions requiring a richer runtime representation.
- Precision/scaling conveniences that preserve existing routing and rotation,
  without conflating output gains with coefficient-wise edits.

The animation opportunity remains deferred. This is still a design-first
commission; the revision should recommend a coherent contract and identify the
remaining semantic decisions rather than publish new runtime syntax as existing.

The [consolidated response](layer-transform-design1.gpt6a.md) is now available.
It recommends ordered cell/matrix/scalar edits, distinct `mul-elements` and
input/output composition operations, and one resolved coefficient map. This
revises the earlier independent-gain and blanket-specificity recommendations:
later edits read preceding results, null skips only the current edit, and
composition factors require numeric values. Its
[decision section](layer-transform-design1.gpt6a.md#decisions-before-implementation)
identifies the single-map tradeoff and representable directional products for
human selection before implementation.

<a id="implementation-commission"></a>

## Implementation commission

After receiving the consolidated proposal, the human directed:

> “use sol xh to implement, if the proposal is ready for it”

They also requested suggested implementation notes from Astra as the starting
point, rather than handing Sol only the design. The next sequence is therefore:

1. Astra high checks the consolidated proposal against actual code and writes
   actionable implementation notes, identifying genuine readiness blockers.
2. The parent reviews those notes and any consequential unresolved behavior.
3. Sol xhigh implements the bounded consolidated contract when ready, with
   coherent commits, meaningful checks and parent architectural review.

The target is [design1](layer-transform-design1.gpt6a.md): ordered cell/matrix/scalar
edits, distinct elementwise and true products, sparse nullable coefficient
operands, numeric composition factors, full-stack layer resolution and explicit
publication/capture/state behavior. Earlier gain-family and blanket-specificity
recommendations remain historical alternatives. Animation remains deferred.

### Readiness disposition and first checkpoint

The [Astra implementation notes](layer-transform-implementation0.gpt6a.md)
identify no algebra or syntax blocker to the first implementation slice.
Parent review approves the recommended reload policy: a candidate requiring
capture unavailable under the running backend's installed capability or frozen
pending-startup plan must fail preflight, retain the running state and explain
the restart requirement. This concerns motion-preflight failure atomicity;
arbitrary rollback of unrelated existing reload side effects is not promised.

Sol xhigh is commissioned for the first coherent checkpoint: typed ordered
algebra, variable-aware program parsing and focused checks, followed by parent
interface review before native/runtime integration. The complete requested
feature still includes publication, capture and state behavior; the parser
checkpoint alone is not feature completion.

## Questions for the layer proposal

1. What is inherited: whole profiles, output rows, scalar gains, individual
   coefficients, or a small combination of these? Can sparse null-valued
   declarations provide the requested freedom without excessive syntax?
2. What is the evaluation order for active layers and the selected base layer?
   Define set/add/multiply operations against the inherited value, including
   mixed operations and release of a lower layer while a higher one stays held.
3. Can the precision use case preserve a global axis swap/rotation? Scaling
   only diagonal coefficients is inadequate when a route is off-diagonal.
   Distinguish scaling an output row from changing one matrix element.
4. How do independent directional branches inherit or share gains? Existing
   directional mode has positive-magnitude positive/negative input branches;
   an output coefficient controls their resulting sign.
5. What remains possible with explicit full-transform replacement or
   composition, and is either necessary in the first interface? Matrix
   multiplication and scalar multiplication of an inherited coefficient must
   not be conflated. Ordering can change the motion produced.
6. Resolve settings from baseline plus active declarations, not by repeatedly
   mutating yesterday's effective matrix. Show activation, release and reload
   examples that restore the expected state without drift.
7. How should the runtime publish a coherent effective transform, discover
   required input capabilities before a layer first becomes active, and expose
   enough provenance to explain why a current gain/coefficient has its value?

Useful worked cases:

- Global quarter-turn plus a precision layer that reduces X/Y output to 25%.
- A wheel-speed layer alongside the pointer precision layer, affecting
  disjoint fields.
- An absolute gain, then an inherited multiplier, then an additive adjustment;
  remove each layer and explain the resulting values.
- Null inheritance versus an explicit zero, especially for off-diagonal entries.
- Different adjustments to wheel-up and wheel-down or pointer-up and pointer-down.
- A virtual key controlling the same layer as a physical key would.

## Current code constraints to inspect

The baseline implementation is recorded at `abb0aabd`. The global configuration
is described in the [configuration reference](/docs/config.adoc), with transform
types and arithmetic in [the parser model](/parser/src/mouse_axis.rs).

[Keyberon](/keyberon/src/layout.rs) already exposes `active_held_layers`,
`current_layer`, and `trans_resolution_layer_order`. These are not identical:
the top layer can remain the same while lower active layers change. The layer
proposal should examine the existing transparent-key-resolution options before
choosing whether motion settings follow that exact order or a separate contract.

[Runtime state](/src/kanata/mod.rs) currently publishes one global map through
a shared mutex and updates it on configuration reload. The layer-change hook
primarily compares `current_layer()` with `prev_layer`. An inherited resolver
must account for relevant changes below the top layer too.

[Axis state](/src/kanata/mouse_axis.rs) and the parser's transform reset fractional
remainders when the map changes. That is an important boundary for future
animation: resetting on every animated coefficient update could prevent small
deltas from ever accumulating into output. The correct state policy remains a
design question, not a reason to begin implementing animation now.

<a id="animation-desire"></a>

## Animation desire — deferred

Preserve this as an opportunity after layer semantics become clear. The human
asked for settings that ramp over time under conditions, including virtual keys
and sustained axis input. X/Y, wheels, extra relative channels and directional
branches all belong within the potential scope.

Rough possibilities, not proposed Kanata syntax:

```text
enter precision layer:
    transition pointer gain from current value toward 0.25

hold a boost virtual key:
    ramp gain toward 2 while held; return toward inherited gain on release

sustain X/Y movement:
    increase a gain while a declared activity condition remains true
    decay or reset after a declared idle interval
```

A possible separation is **trigger → target/parameter operation → transition
envelope**. Layer inheritance could choose targets while an optional envelope
controls how they are approached. That should not imply that all conditions
need a new general-purpose language or that animation must target every matrix
coefficient directly.

Open questions to preserve:

- Clock time, input distance, sustained duration, or a combination as the driver.
- What counts as sustained input: axis-specific activity, any motion, direction
  persistence, speed, an idle gap, or a threshold with hysteresis.
- Whether activity observes physical input or transformed output, avoiding
  unintended feedback from a transform's own generated events.
- Retargeting from the current interpolated value when layers or conditions
  change, versus restarting from a fixed baseline.
- Gains and angles as animation parameters versus raw coefficient interpolation.
  Interpolating rotation matrices element-wise need not preserve a rotation
  or its magnitude, and can collapse movement at intermediate points.
- Fractional state, report-rate independence, directional changes and zero-input
  behavior during a transition.

## Connections

The [Linux flexibility proposal](/design/mouse-axis/linux-flexibility0.gpt6a.md)
concerns qualified sources and output endpoints. Layer-controlled parameters
could eventually apply within those routes, but source/device expansion is not
a prerequisite for solving the current precision-layer use case.

The prospective backlog epic is `kanatax-axis-futures`. Its existing four
children cover device-qualified inputs, virtual outputs, stateful outputs and
raw-HID investigation. This new layer/animation ask is recorded here as a
separate design concern rather than silently expanding one of those children.
