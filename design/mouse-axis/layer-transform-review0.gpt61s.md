---
type: ImplementationReceipt
title: Stable native wheel accounting across identity layers
description: Confirms the parent-review identity-routing gap with a failing production-boundary regression, corrects Linux and both Windows adapters, and records focused executed versus compiled verification.
resource: /design/mouse-axis/layer-transform-review0.gpt61s.md
status: draft
author: openai/gpt-6.1-sol#xhigh
date: 2026-10-29
sources:
  - resource: opencode://ses_e79e734f7ffegcfXUYNxhCPk4s
    title: Reproduces the native identity-routing gap before changing policy, implements stable configuration-wide analog routing and verifies the focused correction
  - resource: opencode://ses_e7b7b7559ffeEC9DfB7nsFmtYO
    title: Parent identifies the accumulator and cadence mismatch and commissions this bounded review correction
  - resource: /design/mouse-axis/layer-transform-implementation1.gpt61s.md
    title: Preserved preceding implementation and verification receipt; this correction qualifies its native identity-forwarding claim
  - resource: /design/mouse-axis/layer-transform-design1.gpt6a.md
    title: Target selective output-state contract preserving unchanged wheel credit through pointer-only transitions
  - resource: /src/kanata/mouse_axis.rs
    title: Configuration-wide routing policy in immutable motion snapshots and unchanged epoch reset authority
  - resource: /src/kanata/mouse_axis/runtime_tests.rs
    title: Real configuration, inactive declaration, layout refresh and reload policy verification
  - resource: /parser/src/mouse_axis/program.rs
    title: Existing declaration-aware capture requirements used to derive the stable policy
  - resource: /src/kanata/linux.rs
    title: Production report-routing boundary, Linux wheel normalization and executed regression fixtures
  - resource: /src/kanata/windows/interception.rs
    title: Production stroke-routing boundary and compiled combined-stroke and wheel fixtures
  - resource: /src/kanata/windows/llhook.rs
    title: Extracted pure callback boundary, stable wheel accounting and compiled native passthrough fixtures
  - resource: /src/oskbd/windows/llhook/mouse.rs
    title: Inspected native wheel delta and legacy sign-only KeyEvent conversion
  - resource: /docs/config.adoc
    title: Maintained distinction between axis-enabled identity layers and legacy identity configurations
---

# Stable native wheel accounting across identity layers

## Finding and correction

**Confirmed and corrected.** Commit `2aadaac9` makes analog wheel accounting a
configuration-wide policy rather than a property of the current effective map.
No blanket wheel reset was introduced. The preceding implementation commits,
independent receipts, Astra documents and parent-maintained reading map are intact.
This qualifies the [preceding receipt](layer-transform-implementation1.gpt61s.md)'s
identity-forwarding claim to satisfy the
[design's selective-state contract](layer-transform-design1.gpt6a.md), rather than
changing that contract to discard wheel credit at pointer transitions.

Before changing routing policy, the Linux event loop's actual decision was
extracted into `route_axis_report`, retaining its `map.is_identity()` branch.
`native_wheel_credit_crosses_identity_precision_in_both_directions` constructs a
real simulated configuration with an inactive X/Y precision program and both
mapped wheels. It refreshes actual default-layer state and submits native hi-res
reports through that boundary, not directly to the unconditional mapper.

The first run failed: V=60/H=60 on identity followed by V=60/H=60 on precision
produced `[]`, instead of `[MouseWheelUp, MouseWheelRight]`. The mapped identity
path ignored the first partial report. Inspection confirmed the corresponding
Windows split: identity conversion produced one sign-only tap per report, while
the transformed path accumulated native delta magnitudes. Windows was not run.

## Exact behavior and compatibility boundary

- `MotionSnapshot::pipeline_enabled` is derived at initial installation and
  reload from the existing declaration-aware `PreparedMotion.required.any()`.
  This covers a non-identity global map or any numeric layer edit/product,
  including dormant and identity-looking instructions. Ordinary layer refresh
  copies the policy unchanged; a debug assertion guards that invariant.
- Linux routes known relative axes through the same `AxisState` even when an
  enabled configuration's effective map is identity. `AxisReportRoute` is also
  the event loop's tested consumption boundary: its output replaces known axes
  at their first report position, while keys, unknown relative codes, metadata
  and synchronization records retain their native routing.
- Interception uses `route_mouse_stroke_axes` for the production policy decision.
  Its existing mapper preserves combined button/motion/metadata components and
  processes signed partial and multiple wheel notches on identity layers too.
- LLHOOK's callback delegates to pure `route_mouse_event`; cursor lookup, key
  delivery and injection are supplied closures. Known wheels always use analog
  accounting when enabled. Untouched pointer movement still returns to Windows
  without querying the cursor or reinjecting it. Unknown events and buttons keep
  their previous routing. The dedicated injection worker remains unchanged.
- Every boundary observes the snapshot before passthrough/early returns. The
  existing per-output epochs and generation remain the only reset authority.
  Pointer-only transitions preserve V/H key and raw credit; changed wheel rows
  still clear their own credit; reloads clear all analog state even when changing
  between enabled and legacy identity policies.
- Linux's existing analog per-wheel low/high-resolution deduplication and signed
  legacy-companion accumulation are reused, not duplicated in a second pipeline.
  A hi-res vertical event still cannot hide a low-res horizontal event.

**Legacy compatibility is deliberate.** An identity global map with no numeric
layer programs keeps its pre-existing native routing. Empty/null-only programs
do not enable analog accounting. Global-only non-identity configurations already
used analog accounting and retain it. The legacy Linux `KbdOut::scroll` policy
and generated-output accumulation are not rewritten or merged with physical
analog state; the selective-state contract here concerns that analog state.
Legacy Windows mapped-wheel sign-only cadence is likewise left intact outside
axis-enabled configurations. The maintained [reference](/docs/config.adoc)
now states this distinction explicitly; the earlier receipt's general native
identity-forwarding statement should be read with this correction.

The interface delta is one crate-private snapshot field and its constructor/
successor arguments, plus private testable adapter boundaries. There is no new
configuration syntax, public parser API, dependency, version, capture policy,
device selection or native installation behavior.

## Focused verification

The regression command before the fix was:

```sh
cargo nextest run -p kanata --features simulated_output -E 'test(native_wheel_credit_crosses_identity_precision_in_both_directions)'
```

It failed as described above, then passed after the policy correction. Final
targeted checks, without repeating the preceding broad suite:

| Check | Result |
| --- | --- |
| `cargo nextest run -p kanata --features simulated_output -E 'test(axis_tests) \| test(mouse_axis::tests) \| test(native_pipeline_policy)'` | **14 passed**, 316 skipped; executed Linux/simulated and shared-state tests only. |
| `cargo check -p kanata --target x86_64-pc-windows-gnu --all-targets` | Passed; native LLHOOK code and its four new pure fixtures **compiled, not executed**. |
| `cargo check -p kanata --target x86_64-pc-windows-gnu --features interception_driver --all-targets` | Passed; native Interception code, three new boundary fixtures and existing adapter fixtures **compiled, not executed**. |
| `cargo clippy -p kanata --all-targets` | Passed; checks the real Linux event-loop write branch, not only simulated output. |
| `cargo fmt --all --check` | Passed. |
| Rekon `check --okf design/mouse-axis/layer-transform-review0.gpt61s.md` | Passed; structure, local destinations and native session provenance, not semantic or hardware verification. |

Executed coverage includes both half-notch transition directions; repeated toggles
without dormant-credit revival; signed partial/multi-notch amounts on both wheels;
Linux raw hi-res and legacy companions; mixed-resolution per-wheel deduplication;
unchanged pointer output, unknown relative codes and metadata; declaration-scoped
policy for global-only/empty/null/numeric/product configurations; and real reload
policy/generation changes. Existing selective epoch/reset tests also passed.

Windows fixtures cover both wheel axes/signs and both transition directions,
partial/multiple taps, raw magnitudes, combined pointer/button/attribute metadata,
native untouched-pointer and unknown-event forwarding, legacy identity cadence
and observation before early returns. Compilation is not evidence of Windows
runtime behavior. During Linux fixture development, the purported unknown REL
code was corrected from `0x0c` (actually horizontal hi-res wheel) to `0x0f`;
that intermediate fixture failure was not another production defect.

Cargo emitted existing manifest/unused-dependency advisories; nextest emitted the
existing missing build-script-output warning. No new clippy warning was reported.

## Limits and ownership

No live input devices, grabs, uinput constructors, Windows hooks, driver execution
or sudo were used. Actual packet fidelity, wheel/button delivery, cursor latency,
Raw Input interaction and native Windows execution remain hardware/platform
verification limits. No macOS/Android expansion or pipeline redesign is included.

The correction commit owns only the three native adapters, shared motion snapshot
and runtime tests, and `docs/config.adoc`. This new receipt records the review
correction separately; it does not revise the independent earlier account or
claim human acceptance.
