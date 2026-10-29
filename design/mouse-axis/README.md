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
    title: Records the human layer/animation direction and reviews and connects the commissioned Astra-high inheritance proposal
  - resource: /docs/config.adoc
    title: Implemented global linear and directional axis-map configuration
  - resource: /design/mouse-axis/linux-flexibility0.gpt6a.md
    title: Independent Linux-first endpoint and device-identity proposal
  - resource: /design/mouse-axis/layer-transform-ask0.gpt6a.md
    title: Commissioned null/set/add/multiply layer semantics and deferred animation sketch
  - resource: /design/mouse-axis/layer-transform-design0.gpt6a.md
    title: Astra-high proposal for sparse gain/coefficient operations, full-stack resolution and report/state boundaries
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

- [Human ask and design commission](layer-transform-ask0.gpt6a.md): precision and
  boost layers for X/Y and other channels; null inheritance; absolute, additive
  and multiplicative parameter operations; questions about gains, matrix
  coefficients and composition. The latest direction is design first with
  Astra high.
- [Astra-high proposal](layer-transform-design0.gpt6a.md): sparse per-element
  operations over independent output gains and routing coefficients. Includes
  worked activation/release cases, rotation-preserving precision layers and
  directional edits. Its [three decision groups](layer-transform-design0.gpt6a.md#decisions-to-accept-before-implementation)
  cover the parameter model, full active-stack precedence and runtime state
  boundaries; these remain proposals for human selection.

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
