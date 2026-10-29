---
type: ImplementationNotes
title: Implementing ordered layer-controlled motion transforms
description: Code-grounded readiness check and staged handoff for Sol xhigh, covering parser IR, full-stack resolution, capture installation, coherent publication and adapter-local reset epochs.
resource: /design/mouse-axis/layer-transform-implementation0.gpt6a.md
status: draft
author: openai/gpt-6-astra#high
date: 2026-10-29
sources:
  - resource: opencode://ses_e7a5dc784ffe94skhw7QeesP8l
    title: Inspects concrete parser/runtime integration points and prepares implementation-readiness notes following the consolidated design
  - resource: opencode://ses_e7b7b7559ffeEC9DfB7nsFmtYO
    title: Parent relays conditional implementation authorization and commissions the Astra handoff before dispatching Sol xhigh
  - resource: /design/mouse-axis/layer-transform-design1.gpt6a.md
    title: Current implementation target, including ordered single-map semantics and the directional composition boundary
  - resource: /parser/src/mouse_axis.rs
    title: Map storage, supported-axis selection, semantic identity and legacy quantization reconfiguration
  - resource: /parser/src/cfg/layer_opts.rs
    title: Icon-only parser and LayerIcons alias
  - resource: /parser/src/cfg/deflayer.rs
    title: Sole parse_layer_opts call, layer indexes and header-variable handling
  - resource: /parser/src/cfg/mod.rs
    title: Sole LayerInfo literal, Cfg/IntermediateCfg transfer, parse ordering, on-load queue and Linux discovery derivation
  - resource: /parser/src/cfg/defcfg.rs
    title: Existing global-map parsing and axis validation
  - resource: /parser/src/cfg/sexpr.rs
    title: Variable-aware atom/list access and non-Send Rc-backed Span representation
  - resource: /parser/src/cfg/vars.rs
    title: Existing defvar parsing and nested list preservation
  - resource: /parser/src/cfg/defgamepad.rs
    title: Existing variable-aware typed parser helper pattern
  - resource: /parser/src/cfg/tests.rs
    title: Existing icon, layer-variable and axis-map parser tests
  - resource: /parser/src/cfg/tests/device_detect.rs
    title: Linux axis discovery and explicit-policy precedence tests
  - resource: /keyberon/src/layout.rs
    title: Full held-layer iterator, default-layer state and separate keyboard-transparent order
  - resource: /src/kanata/mod.rs
    title: Constructors, partial reload mutation, tick scheduling, external default changes, processing errors and layer-notification tests
  - resource: /src/kanata/mouse_axis.rs
    title: AxisState, wheel remainder arrays and current all-output reset path
  - resource: /src/kanata/linux.rs
    title: Shared state lifetime, report snapshots, pure report conversion and native passthrough
  - resource: /src/oskbd/linux.rs
    title: Startup-retained device-discovery policy, explicit input paths and uinput creation
  - resource: /src/kanata/windows/interception.rs
    title: Installed MOVE filter, per-device state, identity path and pure stroke conversion
  - resource: /src/kanata/windows/llhook.rs
    title: Hook availability, early callback returns, separate injection thread and keystate cleanup
  - resource: /src/kanata/windows/mod.rs
    title: Post-keystate shift workaround can remove layer states
  - resource: /src/tcp_server.rs
    title: Direct layer/virtual-key mutation followed by existing WakeUp delivery
  - resource: /src/main.rs
    title: Parser-only check exit and startup ordering of TCP, processing and capture
  - resource: /src/main_lib/args.rs
    title: Actual --check and --cfg flags
  - resource: /src/tests.rs
    title: Root test parse lock and simulation feature gates
  - resource: /src/tests/sim_tests/mod.rs
    title: Existing virtual-key and direct layer-switch simulation helpers
  - resource: /src/oskbd/mod.rs
    title: Simulated-output selection and KeyValue::WakeUp
  - resource: /wasm/src/lib.rs
    title: Another direct default-layer mutation through the simulation interface
  - resource: /justfile
    title: Maintained normal, simulation, serial ignored-test and clippy suites
  - resource: /Cargo.toml
    title: Workspace packages and runtime feature selection
  - resource: /parser/Cargo.toml
    title: Parser package, error dependencies and lsp feature
---

# Implementing ordered layer-controlled motion transforms

## Readiness verdict

**Ready to implement design1 in staged commits.** No additional algebra or syntax
decision blocks the first parser/algebra slice. Use
[design1](layer-transform-design1.gpt6a.md) as the target: cell/matrix/scalar
set/add/`mul-elements`, explicit input/output products, one coefficient map,
ordered instructions and full held/default resolution. Do not revive design0's
independent gains or scope0's blanket-specificity rule.

The consequential integration work is publication, capture installation and
reload staging, not a new expression language. One observable reload policy
needs parent approval **before the reload slice lands**: I recommend rejecting
a candidate requiring capture that this process cannot provide, retaining the
running configuration and explaining the restart requirement. Today's Windows
path merely warns and installs the new map. Design1 required diagnosis but did
not explicitly settle reject-versus-partial-activation. This does not block the
pure first slice.

Inspection was performed on a clean working copy with parent `6804348b`; the
motion runtime baseline remains `abb0aabd`. Locations below refer to that tree.
This handoff changes documentation only. No device capture, broad code tests or
backend runtime checks were performed.

## Source map: concrete landing points

| Existing location | Observation and intended change |
| --- | --- |
| `parser/src/mouse_axis.rs:13–160` | `MouseAxis`, `AXIS_COUNT`, `AxisRow`, `MouseAxisMap`, `supported`, `is_identity`, `maps_input` are reusable. Add the pure ordered-program model beside this domain, not inside an adapter. |
| `parser/src/cfg/layer_opts.rs:7` | `parse_layer_opts` returns `HashMap<String,String>` and permits only icon aliases/atom values. Replace the return with typed layer options; retain icon alias duplicate checks. |
| `parser/src/cfg/deflayer.rs:18–119` | `parse_layer_indexes` is the sole caller of `parse_layer_opts`, returning `(LayerIndexes, LayerIcons)`. Pass variables and map mode/domain through and return typed options by layer name/index. |
| `parser/src/cfg/mod.rs:355,732–760` | `LayerInfo` and its **one construction literal**. Repository-wide Rust search found no other literal. Add the owned program here or an exactly aligned table. `Cfg`/`IntermediateCfg` already transfer `layer_info` at lines 405–419. |
| `src/kanata/mod.rs:446,632,804` | Update **both** `Kanata::new` and `new_from_str`, plus `do_live_reload`. All currently create/replace only a global map. |
| `src/kanata/mod.rs:1105–1169,2286–2299` | `tick_ms` loops over `tick_states`; layer notification currently runs at the end of `handle_keystate_changes`. `change_layer` directly sets default and returns `()`. Motion refresh must cover more than the notification condition. |
| `src/kanata/mod.rs:362` | `mouse_axis_map` is `pub(crate) Arc<Mutex<MouseAxisMap>>`, not an external public API. Root construction/reload plus Linux, Interception and LLHOOK are all current field consumers found by search. Replace them together. |
| `src/kanata/mouse_axis.rs:9–61` | `AxisState` owns transform fractions and separate key/raw wheel remainders. Introduce snapshot observation independently of applying an input vector. |
| `src/kanata/linux.rs:17–155,231–285` | `event_loop` owns one shared `AxisState`; `map_axis_report` already isolates pure normalization/transform/native-event construction for tests. |
| `src/kanata/windows/interception.rs:14–138,270–362` | `event_loop_inner` owns a state per `ic::Device`; `map_mouse_stroke_axes` preserves combined stroke metadata/buttons and has unit fixtures. |
| `src/kanata/windows/llhook.rs:84–188` | One callback-local state, hook gate and separate `AxisInjection` sender. Snapshot observation must precede early callback returns. |

These paths, together with the [source links in the metadata](#parser-and-ir),
are the change inventory; the following sections specify interfaces and traps.

## Parser and IR

### Keep syntax, math and native publication separate

Suggested source split, names adjustable to surrounding style:

- `parser/src/mouse_axis/program.rs`, declared from existing `mouse_axis.rs`:
  pure typed instructions, finite arithmetic, canonical layer-order helper,
  conservative capture requirement calculation and trace descriptions.
- `parser/src/cfg/mouse_axis_program.rs`: S-expression parsing, variable lookup,
  selector expansion and source diagnostics. `layer_opts.rs` delegates here.
- `src/kanata/mouse_axis.rs`: resolved runtime snapshot, publisher state and
  adapter-local epoch observation. This module already owns shared motion state.

Suggested IR boundaries, **not existing types or mandatory exact names**:

```text
AxisDomain { supported_axes, mode }
Cell { output: MouseAxis, input: MouseAxis, branch: PositiveOrNegative }
Instruction {
  operation: Edit(Set | Add | MulElements, nullable_cells)
           | ComposeOutput(complete_numeric_factor)
           | ComposeInput(complete_numeric_factor),
  source: OwnedSourceRef
}
MotionProgram { instructions }
resolve(base, programs_by_layer, canonical_order) -> Result<MouseAxisMap>
required_capture(base, all_programs, domain) -> CaptureRequirements
```

Cell/matrix/scalar forms lower to the same nullable-cell edit representation.
Keep explicitly written nulls identifiable for diagnostics, but do not turn
omission into numeric zero. Scalars expand only their selected Cartesian product;
`except` removes cells from that instruction. Factors lower to complete numeric
matrices using design1's clear-listed-row/identity-unlisted-row construction.
Dimensions are validated against `from`/`to` list lengths **before** lowering;
user axis order must be preserved. Reject factor nulls and directional
`compose-input` during parsing, including in inactive layers.

**Do not store parser `Span` or `SExpr` in `LayerInfo` or runtime instructions.**
`sexpr.rs:33–37` stores file name/content in `Rc<str>`; these values are not
`Send`/`Sync`, whereas Kanata is shared with processing/capture/TCP threads.
Convert provenance to owned strings or `Arc<str>` with byte/line positions and
instruction index. Keep `Span` only for synchronous parser diagnostics. No
unsafe `Send` workaround is justified.

### Variables and construction order

`parse_cfg_raw_string` parses defcfg around line 625, variables at 672–676, and
layer indexes at 731–732. Thus mode and variables are already available at the
header pass. Change `parse_layer_opts` to receive them, and call
`SExpr::atom(Some(vars))` / `list(Some(vars))` at **every nested boundary**:
program, instruction, operand, matrix row, selector, numeric/null value and factor
lists. The helpers resolve aliases lazily; a list returned once is not recursively
expanded for you. Existing `defgamepad.rs:533–575` is a nearby typed-helper pattern.

Preserve the current header name handling, icon quote trimming and canonical
icon duplicate detection. Test literal and variable-supplied whole programs,
instructions, nested operands, axes and numbers; preserve unresolved-variable
errors and `lsp` reference recording. Capture both the instruction's use-site and
value-definition location when convenient, rather than losing the layer's
identity when a whole instruction comes from `$var`.

`populate_cfg_with_icfg` queues `alias-to-trigger-on-load` into `action_queue`
at lines 393–398; it does not run it. Initial resolution is the initial actual
layout/default, not a prediction of the queued alias's eventual layer. The
normal tick path will publish its later effect. Validate that initial/default
fold in the parser's checked path so `--check` catches its arithmetic failures;
retain runtime validation for later combinations. Do not reject every dormant
program by evaluating it in isolation against the baseline: its valid operand
context may depend on lower layers.

### Axis representation and checked math

Keep the canonical ten-axis indices and storage. `MouseAxis::supported()` returns
all axes on Linux/Android/`target_os="unknown"`, X/Y/V/H elsewhere. Scalar defaults
and explicit-name validation use the supported domain; they must not mutate
unsupported hidden rows into phantom nonidentity output on Windows. Factors
leave the hidden complement identity. Linux's Z/RX/RY/RZ/dial/misc remain usable.
The unknown target is also relevant to wasm; it is not proof of a native backend.

`MouseAxisMap` uses separate `rows` and `negative_rows`, not an interleaved 20-column
allocation. In directional mode they are positive/negative magnitude branches;
the identity negative diagonal is **−1**. Do not negate a negative branch operand
again. In linear mode, ignore `negative_rows` when comparing semantic rows or
detecting capture. Existing derived `PartialEq` compares it even though
`is_identity` does not. Canonicalize unused negative storage in resolved linear
snapshots or use explicit semantic comparison consistently; never carry that
unused storage into a later directional configuration.

For each product read a copy of the previous matrix and write a new result:

```text
output: new[o,i] = sum_k Q[o,k] * old[k,i]
input:  new[o,i] = sum_k old[o,k] * P[k,i]
```

Output composition runs the first formula independently for both branch tables
in directional mode. Input composition runs only on linear `rows`. Use canonical
axis order, checked finite multiplication **and partial sum** at every step;
finite inputs can overflow and opposing infinities must not be “cancelled” back
into an accepted result. No intermediate coefficient rounding/saturation, no
in-place products, no implicit transpose and no algebraic fusion that changes
the documented floating-point evaluation order. Keep existing motion-output
saturation and near-integer quantization in `MouseAxisTransform::apply` separate.

## Resolver and publication interfaces

Suggested runtime separation:

```text
MotionConfig: immutable baseline + layer programs + capture requirements
MotionResolver: generation + last canonical stack + last effective map
MotionSnapshot: map + generation + output_epochs[AXIS_COUNT]
                + four mapped-wheel-direction bits
AxisState: existing fractions/notch state + last-seen epochs/generation
```

Keep the shared publication as `Arc<Mutex<Arc<MotionSnapshot>>>` or a small copied
snapshot under the existing mutex. There is no need to add a lock-free dependency.
The publisher resolves before taking that mutex, replaces the whole snapshot,
then releases it. Adapters clone/copy once per report and immediately release
the publication lock. Wheel mapped-direction bits should come from the same
configuration generation: current `route_wheels` closures repeatedly consult
global `MAPPED_KEYS`, which can change midway through an old-map report on reload.
Snapshotting the four axis-routing bits removes that specific mixed-generation
case without making the entire keyboard configuration transactional.

Canonical stack = newest-first held IDs, then default; deduplicate first occurrence;
reverse. `keyberon/src/layout.rs:2514–2537` exposes everything needed. Do not use
`trans_resolution_layer_order` or `current_layer` as the resolver key. Keep a full
canonical stack for provenance, including programless layers if desired. A pure
helper taking held IDs and a default ID is easy to test without constructing
native input devices. The same layer runs once at its highest surviving position.

### Actual publication scheduling

- Call the common refresh after **each `tick_states`**, not only after the outer
  `tick_ms` batch. Existing `tcp_layer_change_tests` at `mod.rs:3358+` demonstrate
  both held-layer transitions within one multi-ms batch. Motion needs equivalent
  epoch observations even when no report arrives between them.
- Prefer the end of `tick_states`, after custom actions and
  `check_release_non_physical_shift` have completed. That workaround can itself
  remove a `LayerModifier` on some Windows builds (`windows/mod.rs:110–117`).
- Also refresh actual state at the entry to `tick_ms`, including `ms_elapsed=0`,
  to catch an externally changed default without waiting for a layout tick.
  Queued virtual events must still wait until Keyberon actually processes them.
- `change_layer` directly updates the default. TCP's `ChangeLayer` handler calls
  it, and **already sends `WakeUp` after the command** (`tcp_server.rs:435–442`).
  `ActOnFakeKey` likewise queues events and gets that wakeup. Preserve this path;
  no new protocol or synchronous hook wait is needed. The root and wasm simulator
  layer-switch helpers also call `set_default_layer` directly; the common
  tick-entry refresh covers them.
- Audit cleanup outside ordinary ticks: LLHOOK `win_synchronize_keystates`
  removes all states at selected coordinates and runs immediately before the
  processing loop can block. Refresh after it before blocking. Other cleanup
  paths that queue releases must allow the next tick to drain them. Refresh is
  about actual surviving state, not predicted key effects.

Cache equality prevents redundant math/publication. An intermediate instruction
result within a single resolution is never published. Processing errors already
propagate through `handle_time_ticks` into the processing-loop error/panic boundary
(`mod.rs:2646–2649,2764`); attach layer/instruction/source context to a nonfinite
active combination instead of substituting a different effective map silently.

## Capture requirements and installation facts

**A sound initial policy is preferable to incomplete reachability optimization.**
For the first runtime slice, retain exact existing global-only `maps_input`
detection; when any layer has a potentially effective numeric instruction,
conservatively request all supported relative inputs. Empty/all-null programs
need none beyond baseline. This covers every product and cross-layer routing
combination, including suppression, and is an explicit conservative union.
Record that it may install movement capture for a wheel-only layer. It can later
be narrowed by proved structural analysis without changing the language.

Do not evaluate only the startup layer, only each layer against identity, or
only final nonzero columns. All can miss a future route or identity suppression.
In `cfg/mod.rs:899–912`, include declared motion requirements when deriving Linux
`Any`, while keeping the existing `is_none()` guard: an explicit detect policy
or input selection remains authoritative. Extend the existing `device_detect`
tests rather than inventing a second selection policy. Warn about inactive
motion declarations on macOS too; the current warning checks only the global map.

Distinguish **required**, **planned**, and **installed** capture:

| Backend | What is actually installed |
| --- | --- |
| Interception | `event_loop_inner:31–46` sets a process-lifetime mouse filter, possibly excluding MOVE. Record mouse-filter availability and its MOVE bit after installation, including legacy reasons such as explicit HWIDs and movement-key use. |
| LLHOOK | A mouse hook installed for a button or wheel already receives movement; `maps_movement_at_startup` is a gate/logging input, not a narrower installed movement filter. If the hook exists, the callback/injection path supports X/Y/V/H. If absent, reload cannot currently install it. |
| Linux | `KbdIn::new` retains detect mode/include/exclude settings. Nonempty explicit paths bypass autodetection. `KeyboardMice` means keyboard/combined keyboard-mouse, **not standalone mice** (`oskbd/linux.rs:303–321`). Changing the parser's derived mode on reload does not rebuild the existing reader. |

The CLI starts TCP and processing before the event loop (`main.rs:201–236`).
Freeze a startup capture plan before launching those threads, and make the
backend install that plan even if an early reload changes the current map.
Represent pending installation explicitly; compare reload requirements with the
frozen plan while pending and with actual recorded capabilities after success.
Do not treat a not-yet-installed hook as either permanently absent or fully
installed based on the current effective map. Installation failure follows the
existing backend error path.

Explicit device filters are not evidence that every desired physical axis is
available, and must not be widened behind the user. For Linux reload, separately
diagnose a newly required automatic discovery mode that the running reader cannot
adopt; fixed paths/filters remain the selected source universe. No new hotplug
reconciliation or reader rebuild is needed in this commission.

## Reload staging: make new failures occur before installation

Current `do_live_reload` is **not transactional**:

- It parses the candidate, then calls `update_kbd_out` at line 816.
- It replaces `layout` and `layer_info` at 823–825 and `MAPPED_KEYS` at 865.
- Later gamepad edge application and X11 repeat-rate updates can fail.
- It announces success at 897, but replaces the axis map only at 937.

Do not add fallible motion resolution at that late assignment. Build a
`PreparedMotion` from the candidate **before line 816**: immutable program/config,
initial canonical order/map, wheel-routing mask, required capture, and validated
reload capability result. Do this before moving fields out of `cfg`.

After successful preflight, install the new layout/program/generation and motion
snapshot together in the non-fallible state-assignment section. Advance all output
epochs for successful reload, including an equal numeric map. Make the success
message follow the actual motion installation. No new resolver error should be
able to leave a new layout pointing at old programs or an old snapshot.

This provides **motion-preflight failure atomicity**, not rollback of arbitrary
existing gamepad/OS side effects. Do not claim the entire reload function becomes
transactional. If unrelated later fallible work remains, at least the installed
layout/program/map must belong to the same generation and failed-reload reporting
must remain honest. A whole-reload transaction redesign is outside this handoff.

For insufficient installed capture, the recommended policy is an early error
with the missing capability and “restart required,” before modifying live state.
This policy completion needs the parent's approval because the old Windows map
reload merely warns. Compare with actual installation, not an old-current-map
identity test: identity can coexist with a fully installed hook, and a nonidentity
wheel map can coexist with an Interception filter that excludes MOVE.

## Epoch observation and adapter changes

Add `AxisState::observe_snapshot` (name illustrative) that updates its map and
clears only outputs whose epochs changed. For V/H, clear the corresponding index
in **both** wheel remainder arrays. Keep V/H index lookup independent of numeric
axis indices. Generation change/reload clears every output. Publication increments
epochs on semantic final-row changes; mode changes are full discontinuities.

Do not call existing `MouseAxisTransform::reconfigure` afterwards: it resets
**all** fractions on any map difference. Its public legacy API/tests can remain;
the new adapter path can use a selective setter or the currently public fields.
Choose one authoritative reset path. `AxisState::apply` should then apply input
under the already observed snapshot, without a second full-map reset.

| Adapter | Required hook placement |
| --- | --- |
| Linux | Observe each report's snapshot before `is_identity`; retain native identity forwarding and existing `map_axis_report` normalization. A single `AxisState` remains shared across the flattened device stream. |
| Interception | Observe in that device's state before the identity branch; keep `HashMap<Device,AxisState>`. A device inactive across A→B→A must reset when it next reports. Preserve absolute-stroke handling and button/attribute output in `map_mouse_stroke_axes`. |
| LLHOOK | Observe immediately after taking the snapshot, before `current_mouse_point()` or other early returns. Retain callback-local state and the existing separate SendInput worker. |

Map equality alone cannot detect A→B→A between reports; monotonically advanced
output epochs can. Conversely, adding/removing a null-only layer or changing
provenance without a numeric row change must not discard fractional credit.
Compare only active linear storage or both directional tables as appropriate.
Do not narrow Linux state to per-device state in this work: the input envelope
does not yet preserve that identity, and changing credit sharing is another task.

### Locks, queues and report scope

The processing path holds `Kanata` and briefly acquires the snapshot lock.
Readers must release the snapshot lock **before** acquiring `Kanata`, consulting
other mutable locks, writing output, or calling a sender. Linux later locks
`Kanata` for `kbd_out`; reversing that order would deadlock publication.
No resolver execution, source formatting or notification work belongs under
the shared snapshot mutex.

Retain native sender behavior: Linux/Interception use `try_send`; LLHOOK uses
its preprocessor `try_send_panic` and asynchronous injection channel. Do not
block a hook waiting for a layout change or send to the processing thread's own
bounded channel while holding its lock. TCP's existing blocking wake send occurs
after its mutation lock is released; keep that ordering.

One snapshot governs a report's transform and axis wheel gating. This does not
order physical keyboard arrival ahead of all concurrently arriving motion.
Design1's latest-processed-layout-state contract remains the visible behavior.
Queued output already produced using a prior snapshot is not retroactively
retransformed when publication advances.

## Diagnostics and meaningful tests

Keep diagnostics bounded: source location, layer name, instruction index/kind,
selected cells/factor axes, canonical stack and generation. A debug-level change
summary plus a pure on-demand explanation helper over the immutable program
is enough. Product explanations show a sum of source contributions, not one
“winning cell.” No new TCP protocol, generic DSL or public introspection service
is required. Preserve `LayerInfo.cfg_text`, icons and existing top-layer messages.

Reuse these tests and seams:

- `parser/src/cfg/tests.rs`: icon duplicates/variables at 2213–2252; global
  axis parsing at 2354 onward. Add a focused layer-program test module including
  nested variables, `deflayer`/`deflayermap`, sparse/null/zero, selector errors,
  factor completion/order, directional rejection and startup resolution errors.
- `parser/src/mouse_axis.rs::tests`: retain current quantization/saturation and
  directional identity tests. Pure program tests use design1's numeric cases,
  nonuniform matrix operands, rectangular named blocks, finite product overflow,
  and proof that products read the old matrix. Test a four-axis domain even on
  Linux so unsupported hidden storage cannot corrupt identity detection.
- `src/kanata/mod.rs::tcp_layer_change_tests`: adapt its `new_from_str`, input and
  tick patterns to inspect snapshots/epochs. Cover lower-layer release with an
  unchanged top, default changes with held layers, duplicate priority fallback,
  queued virtual actions, on-load actions and both transitions in one tick batch.
- `src/kanata/mouse_axis.rs::tests`: A→B→A without intervening apply, pointer-only
  changes preserving V/H credit, equal-map provenance, reload reset, and independent
  device states. Update the existing pointer-change-resets-wheel expectation to
  the explicitly selected policy; do not accidentally preserve the old blanket reset.
- `src/kanata/linux.rs::axis_tests` and Interception `axis_tests`: retain separate
  wheel-resolution deduplication, combined-stroke metadata and native extended
  axes. Add snapshot-aware fixtures without opening readers/hooks.
- Pure capture-plan/preflight tests: inactive programs, products, explicit filters,
  LLHOOK already present for buttons, Interception missing MOVE, pending startup
  installation, failed candidate preserving old generation/map, and equal-map reload.

Parser/root tests use their existing `CFG_PARSE_LOCK` where appropriate. A plain
root `Kanata::new_from_str` creates a real `KbdOut` unless compiled for simulated
output; do not use it casually in unattended native tests. The simulation runner
already has virtual-key/layer-switch verbs; a small runtime test can inspect the
snapshot and feed `AxisState` directly without inventing a complete motion DSL.

## Independently checkable commit sequence

Commit explicit owned paths with `jj commit -m "…" PATHS…`; no squash/push.
Each stage should build and carry its relevant checks before the next stage.

1. **Typed motion algebra and program parser.** Implement IR, finite math,
   mode/domain validation and nested S-expression parsing, with pure tests.
   Keep it callable internally; do not yet accept a top-level runtime option
   that would parse successfully but be ignored by the running program.
2. **Selective snapshot/state substrate.** Add semantic row comparison,
   generation/epochs, snapshot observation and conservative capture-plan types.
   Keep the current global map wired through this substrate so adapters and
   global-only identity behavior can be tested independently of layer syntax.
3. **Layer declaration and resolver integration.** Wire typed header options,
   the sole LayerInfo construction, both constructors, initial/default validation,
   full-stack per-tick publication, and native startup capture consumers together.
   Until the staged reload is ready, reject reload of the new motion-program
   configurations explicitly rather than silently installing partial state.
4. **Prepared reload and installation accounting.** Land actual capability
   records/pending-plan handling, preflight before mutation, coherent generation
   installation and approved restart-error policy. Remove the temporary explicit
   reload restriction only after its regression cases pass.
5. **User-facing examples and verification receipt.** Update the maintained
   configuration reference, add minimal precision/matrix/product examples and
   parse-sample coverage; complete cross-target checks and record remaining
   hardware verification separately. Avoid treating partial stages as delivery.

Stages 2–4 share the snapshot/capture interfaces above; Sol should settle their
type signatures before changing all adapter call sites. Fresh interfaces do not
need a new dependency. If one stage proves too interwoven to build independently,
keep the coherent integration in one commit rather than landing dead user syntax.

### Useful verification commands

These commands are for the implementation, **not receipts of tests run here**.
Use focused test filters while developing; run the broader listed suites once
integration changes justify them. Source inspection confirms the `justfile`
suites; its unqualified root tests can construct native output, so prefer the
simulated-output form for unattended runtime testing.

```sh
cargo nextest run -p kanata-parser -p kanata-keyberon
cargo nextest run -p kanata --features simulated_output -E 'test(mouse_axis) | test(axis_tests) | test(tcp_layer_change_tests)'
cargo nextest run -p kanata --features simulated_output -E 'test(sim_tests)'
cargo test -p kanata --features simulated_output -- must_be_single_threaded --ignored --test-threads=1
cargo fmt --all --check
cargo clippy -p kanata -p kanata-parser -p kanata-keyberon --features simulated_output --all-targets
cargo check -p kanata-parser --features lsp
cargo check -p kanata --target x86_64-pc-windows-gnu --all-targets
cargo check -p kanata --target x86_64-pc-windows-gnu --features interception_driver --all-targets
```

Include newly named program tests in targeted filters; parser package tests
already run them all. Review full compiler diagnostics instead of attributing
unrelated pre-existing warnings to this feature. Cross-checking is not Windows
execution. Additional workspace/wasm checks are warranted if shared public types
break their consumers, not a substitute for the native adapter-specific builds.

Validate each new sample with `cargo run --bin kanata -- --check --cfg PATH`.
`main.rs:121–141` exits through parser-only validation before `Kanata::new` and
capture setup. Do not launch the normal event loop, use sudo, open `/dev/input`,
or install hooks against the human's live devices as an unattended check. Physical
activation responsiveness, hardware packet fidelity and Windows hook behavior
remain separate manual verification, with honest limits in the implementation
receipt.

## Handoff conclusion

**Dispatch Sol xhigh for stage 1 now:** pure typed program algebra plus the
variable-aware parser and focused tests, compiled behind internal entrypoints.
Return a commit and numerical/parser verification at that interface checkpoint,
then continue the vertical integration after parent review.

No mathematical or syntax blocker remains for that slice. Before the reload
stage, approve the recommended reject-before-mutation policy for unavailable
installed capture. The non-Send source spans, per-inner-tick updates, inactive
capture installation and epoch observation on passthrough paths are concrete
implementation obligations, not reasons to restart design. Animation remains
deferred under [design1's boundary](layer-transform-design1.gpt6a.md#deferred-animation).

Readiness evidence here is code inspection, not runtime validation. Environment
discovery confirmed `cargo-nextest 0.9.122` and installed targets
`x86_64-unknown-linux-gnu`, `x86_64-pc-windows-gnu`, and `wasm32-unknown-unknown`;
that does not prove a cross build or backend test has passed. The parent maintains
the [reading map](README.md) and can link this handoff without altering the
preserved design artifacts.
