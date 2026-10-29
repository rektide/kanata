---
type: Index
title: Mouse and relative-input transform design
description: Reading map for implemented relative-axis mapping, prospective Linux routing, layer inheritance, and the deferred animation desire.
resource: /design/mouse-axis/README.md
status: draft
author: openai/gpt-6-astra#xhigh
date: 2026-10-29
sources:
  - resource: opencode://ses_e7b7b7559ffeEC9DfB7nsFmtYO
    title: Reviews the delivered runtime and commissions an identity-path wheel-continuity regression, while adding a minimal physical pointer speed example
  - resource: /docs/config.adoc
    title: Implemented global linear and directional axis-map configuration
  - resource: /design/mouse-axis/linux-flexibility0.gpt6a.md
    title: Independent Linux-first endpoint and device-identity proposal
  - resource: /design/mouse-axis/layer-transform-ask0.gpt6a.md
    title: Commissioned null/set/add/multiply layer semantics and deferred animation sketch
  - resource: /design/mouse-axis/layer-transform-design0.gpt6a.md
    title: Astra-high proposal for sparse gain/coefficient operations, full-stack resolution and report/state boundaries
  - resource: /design/mouse-axis/layer-transform-scope0.gpt6a.md
    title: Astra-high refinement of family-local blanket defaults, explicit cell operations and null exclusions
  - resource: /design/mouse-axis/layer-transform-design1.gpt6a.md
    title: Consolidated Astra-high proposal for ordered cell/matrix/scalar edits and explicit input/output matrix products
  - resource: /design/mouse-axis/layer-transform-implementation0.gpt6a.md
    title: Code-grounded Astra readiness review, integration interfaces and staged implementation handoff
  - resource: /design/mouse-axis/layer-transform-implementation0.gpt61s.md
    title: Sol xhigh stage-one implementation receipt and verified parser/algebra contracts
  - resource: /design/mouse-axis/layer-transform-implementation1.gpt61s.md
    title: Delivered native/runtime integration, examples, verification results and hardware limits
---

# Mouse and relative-input transform design

## Implemented foundation

The [configuration reference](/docs/config.adoc), section `mouse-axis-map`,
owns implemented behavior: global linear maps, optional independent directions,
mixing/scaling, Linux's extended native relative channels and layer programs.

Starting examples:

- [Two independent wheels](/cfg_samples/mouse-axis-dual-wheel.kbd).
- [Wheel input driving pointer X/Y](/cfg_samples/mouse-axis-wheel-pointer.kbd).
- [Output axes absent from the physical mouse](/cfg_samples/mouse-axis-generated.kbd).
- [Independent directional branches](/cfg_samples/mouse-axis-directional.kbd).
- [Minimal pointer precision/boost layers](/cfg_samples/mouse-axis-layer-speed.kbd):
  hold F1 for quarter speed, F2 for double speed, both for half speed; no rotation
  or wheel remapping. Parser-only validation passed in the parent session.
- [Layer programs with rotation and virtual control](/cfg_samples/mouse-axis-layers.kbd).
- [Matrix edits](/cfg_samples/mouse-axis-layer-matrix.kbd),
  [true products](/cfg_samples/mouse-axis-layer-products.kbd), and
  [directional layer products](/cfg_samples/mouse-axis-layer-directional.kbd).

## Layer-controlled transforms — delivered, final adapter review underway

- [Runtime integration receipt](layer-transform-implementation1.gpt61s.md):
  typed public headers, full-stack resolution, immutable snapshots and selective
  epochs, frozen capture plans, preflighted reloads, checked examples and native
  adapter integration. It records executed tests versus cross-compilation and
  the remaining physical-device verification limits.
- Parent review identified an identity/transformed wheel-accounting boundary:
  pointer-only layer changes should preserve partial wheel credit even when the
  effective map becomes identity. A fresh Sol-xhigh session is verifying and
  correcting that boundary with native-adapter regressions before final delivery.

- [Implementation commission](layer-transform-ask0.gpt6a.md#implementation-commission):
  the human requests Astra-high readiness and implementation notes, followed by
  Sol-xhigh implementation of the consolidated proposal when ready. Runtime
  support is still described by the configuration reference above.
- [Astra implementation notes](layer-transform-implementation0.gpt6a.md): concrete
  parser/runtime landing points, capture installation facts, reload staging and
  meaningful checks. Parent review found no blocker to the first slice and
  approved rejecting capture-incompatible reloads before live-state mutation.
- [Implemented algebra/parser checkpoint](layer-transform-implementation0.gpt61s.md):
  Sol xhigh delivered validated nullable operands and numeric factors, checked
  ordered evaluation, owned provenance and nested variable-aware parsing.
  Parent interface review found no blocker; the receipt records the focused
  checks. Public layer syntax was gated at that checkpoint and is now connected
  by the runtime integration above.

- [Latest scope direction](layer-transform-ask0.gpt6a.md#cell-matrix-scalar):
  cell, matrix and scalar operands should coexist, with coefficient-wise
  multiplication and true matrix composition explicitly distinguished.
- **[Current consolidated proposal](layer-transform-design1.gpt6a.md):** ordered
  `set`, `add` and `mul-elements` operations on cell/matrix/scalar operands,
  plus `compose-output` and `compose-input` for true products. Nullable edit
  operands and numeric composition factors have separate contracts. Its
  [decision section](layer-transform-design1.gpt6a.md#decisions-before-implementation)
  calls out the single-map versus independent-gain tradeoff, null boundary and
  directional composition limits for the readiness review and implementation
  handoff.
- [Human ask and design commission](layer-transform-ask0.gpt6a.md): precision and
  boost layers for X/Y and other channels; null inheritance; absolute, additive
  and multiplicative parameter operations; questions about gains, matrix
  coefficients and composition. The latest direction is design first with
  Astra high.
- [Earlier Astra-high proposal](layer-transform-design0.gpt6a.md): sparse per-element
  operations over independent output gains and routing coefficients. Includes
  worked activation/release cases, rotation-preserving precision layers and
  directional edits. Its [three decision groups](layer-transform-design0.gpt6a.md#decisions-to-accept-before-implementation)
  cover the parameter model, full active-stack precedence and runtime state
  boundaries. Its independent gain family and one-operation-per-cell model are
  retained as design history rather than silently merged into the new proposal.
- [Cell and blanket scope clarification](layer-transform-ask0.gpt6a.md#cell-and-blanket-scope):
  preserves the human request for both individual coefficient edits and blanket
  application.
- [Earlier blanket and cell addendum](layer-transform-scope0.gpt6a.md): recommends a
  family-local blanket default, overridden by explicit cell operations; explicit
  null excludes a cell, while omission accepts the blanket. Every selected
  operation uses the pre-layer inherited value. This refines the first proposal's
  omitted/null equivalence. It records the preceding scope discussion; the latest
  consolidated proposal now uses explicit instruction order instead of this
  blanket-specificity rule.

## Deferred animation desire

- [Recorded ask and rough sketch](layer-transform-ask0.gpt6a.md#animation-desire):
  time-based ramps, virtual-key-controlled changes and sustained-input
  conditions; trigger/target/envelope separation and open state-policy questions.

## Linux source and destination flexibility

- [Independent Astra medium proposal](linux-flexibility0.gpt6a.md): qualified
  input identities, named virtual outputs, stateful controllers and evidence-led
  raw-HID investigation. It distinguishes generating new output capabilities
  from recovering independent physical input information.
- Prospective work anchor: `kanatax-axis-futures`. Its four children cover those
  source/destination opportunities. The layer and animation questions have their
  own commission above.

Design documents are proposals. The configuration reference remains the place
to check available runtime behavior.
