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
    title: Records the cell/matrix/scalar direction and reviews and connects the consolidated ordered-operation proposal with its design history and deferred animation desire
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
---

# Mouse and relative-input transform design

## Implemented foundation

The [configuration reference](/docs/config.adoc), section `mouse-axis-map`,
owns implemented behavior: global linear maps, optional independent directions,
mixing/scaling and Linux's extended native relative channels.

Starting examples:

- [Two independent wheels](/cfg_samples/mouse-axis-dual-wheel.kbd).
- [Wheel input driving pointer X/Y](/cfg_samples/mouse-axis-wheel-pointer.kbd).
- [Output axes absent from the physical mouse](/cfg_samples/mouse-axis-generated.kbd).
- [Independent directional branches](/cfg_samples/mouse-axis-directional.kbd).

## Layer-controlled transforms — current design question

- [Latest scope direction](layer-transform-ask0.gpt6a.md#cell-matrix-scalar):
  cell, matrix and scalar operands should coexist, with coefficient-wise
  multiplication and true matrix composition explicitly distinguished.
- **[Current consolidated proposal](layer-transform-design1.gpt6a.md):** ordered
  `set`, `add` and `mul-elements` operations on cell/matrix/scalar operands,
  plus `compose-output` and `compose-input` for true products. Nullable edit
  operands and numeric composition factors have separate contracts. Its
  [decision section](layer-transform-design1.gpt6a.md#decisions-before-implementation)
  calls out the single-map versus independent-gain tradeoff, null boundary and
  directional composition limits; these remain recommendations for selection.
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
