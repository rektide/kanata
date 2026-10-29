---
type: Request
title: Layer-controlled input transforms and deferred animation desire
description: Human requirements and design questions for inherited null, set, add and multiply operations on motion-transform settings, with a deferred animation sketch.
resource: /design/mouse-axis/layer-transform-ask0.gpt6a.md
status: draft
author: openai/gpt-6-astra#xhigh
date: 2026-10-29
sources:
  - resource: opencode://ses_e7b7b7559ffeEC9DfB7nsFmtYO
    title: Human commissions Astra high to design layer-controlled transforms with sparse per-element null/omitted edits and set/add/multiply operations, while preserving animation as a deferred desire
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

This document preserves the ask and rough animation ideas. It does not select
final configuration syntax or authorize implementation of layer transforms or
animation. The corresponding proposal should distinguish recommendations from
accepted runtime behavior.

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
