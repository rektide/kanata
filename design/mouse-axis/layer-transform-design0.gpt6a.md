---
type: Design
title: Inherited layer operations for physical input transforms
description: A proposed sparse null, set, add and multiply resolver over output gains and routing coefficients, with explicit layer order, publication and fractional-state semantics.
resource: /design/mouse-axis/layer-transform-design0.gpt6a.md
status: draft
author: openai/gpt-6-astra#high
date: 2026-10-29
sources:
  - resource: opencode://ses_e7a5dc784ffe94skhw7QeesP8l
    title: Independent commissioned design of layer inheritance, scalar operations, configuration examples and runtime boundaries
  - resource: /design/mouse-axis/layer-transform-ask0.gpt6a.md
    title: Human commission initially read at 9d0e273b, its subsequent sparse per-element null clarification, and the deferred animation desire
  - resource: /design/mouse-axis/linux-flexibility0.gpt6a.md
    title: Prior independent Linux-native source and output endpoint exploration
  - resource: /design/mouse-axis/README.md
    title: Current reading map and separation of implemented behavior from proposals
  - resource: /parser/src/mouse_axis.rs
    title: Actual coefficient domains, identity defaults, input capture detection and quantization
  - resource: /parser/src/cfg/defcfg.rs
    title: Existing complete-row parsing and axis support validation
  - resource: /parser/src/cfg/layer_opts.rs
    title: Current atom-valued icon option parser
  - resource: /parser/src/cfg/deflayer.rs
    title: Layer header parsing, layer indexes and held/default action construction
  - resource: /parser/src/cfg/mod.rs
    title: LayerInfo construction and Linux capture-mode derivation
  - resource: /keyberon/src/layout.rs
    title: Held layer states, default layer, current layer and transparent resolution variants
  - resource: /src/kanata/mod.rs
    title: Runtime map storage, reload publication, tick completion and layer-change hook
  - resource: /src/kanata/mouse_axis.rs
    title: Output quantization and wheel remainder lifecycle
  - resource: /src/kanata/linux.rs
    title: Input-report snapshot boundary and asynchronous key-event submission
  - resource: /src/kanata/windows/interception.rs
    title: Startup movement interception gate and per-device axis state
  - resource: /src/kanata/windows/llhook.rs
    title: Startup mouse-hook gate and callback map snapshots
  - resource: /docs/config.adoc
    title: Implemented global transforms, units, layer headers, virtual keys and generated mouse-speed behavior
---

# Inherited layer operations for physical input transforms

## Recommendation and standing

**Yes: null inheritance plus set, add and multiply is enough for a useful first
version.** Apply those operations to two kinds of scalar parameter:

1. **Output gain:** scale everything arriving at an output channel. This makes a
   precision layer preserve an existing rotation, swap or mixed-input mapping.
2. **Routing coefficient:** edit one input-to-output contribution. This supplies
   individual matrix-element freedom, including independent directional branches.

The human clarified that null means **not having to affect every mapping
element**. Leaf-level sparsity is a requirement: a layer can change just
`C[x,y]`, with every other coefficient in X's row and every other row left at
its inherited value. Output-gain convenience does not replace that freedom.

Resolve sparse layer declarations from the global baseline every time the
relevant active stack changes. Omitted and `null` fields inherit; numeric zero
really sets zero. Higher layers operate on the values produced by lower layers.
Keep matrix composition outside this first interface.

This is the independent **design-first proposal** requested in the
[commission](layer-transform-ask0.gpt6a.md). All `mouse-axis` layer options and
operation syntax below are **prospective**, not accepted or implemented Kanata
features. Existing `defcfg mouse-axis-map`, layer actions and virtual-key actions
are identified separately. No runtime code was changed or behavior exercised.
Code observations use the `abb0aabd` implementation present under commission
commit `9d0e273b`; this document's checks concern documentation only.

## Ground: what exists and what must change

| Observed implementation | Design implication |
| --- | --- |
| `MouseAxisMap` has ten signed linear columns, or positive/negative branch coefficients per input in directional mode. | Preserve that capable domain, including Linux's extra channels. |
| Global map rows replace identity; omitted global rows keep identity; empty global rows suppress output. | Layer patches need a different name and sparse semantics, not reuse of complete-row parsing. |
| `LayerInfo` holds name, source text and icon; `parse_layer_opts` accepts only icon aliases and atom values. | Nested transform options require a typed extension, not an already available option. |
| `active_held_layers()` yields newest held state first; `default_layer` is separate. | Resolve a full ordered stack; `current_layer()` alone is insufficient. |
| Runtime map is `Arc<Mutex<MouseAxisMap>>`; reload replaces it. | Publish an entire resolved snapshot, never individual in-place edits. |
| `MouseAxisTransform::reconfigure` resets all fractions on map changes; `AxisState::apply` also resets both wheel remainder arrays. | Define a deliberate layer-transition policy before frequent changes arrive. |
| Linux snapshots the map per report; Windows snapshots in its native input path. Keyboard actions reach the processing loop asynchronously. | A snapshot is coherent, but physical key arrival and motion are not one ordered transaction today. |

Evidence: [map and arithmetic](/parser/src/mouse_axis.rs),
[layer options](/parser/src/cfg/layer_opts.rs),
[layer metadata](/parser/src/cfg/mod.rs),
[layout ordering](/keyberon/src/layout.rs),
[runtime](/src/kanata/mod.rs), [axis state](/src/kanata/mouse_axis.rs).
The [configuration reference](/docs/config.adoc), sections `mouse-axis-map` and
`mouse-speed`, also confirms that `movemouse-speed` affects generated movement,
not the physical pointer input addressed here.

## The parameter contract

### Two independent parameter families

Let `C[o,i]` be a linear routing coefficient and `G[o]` an output gain. The
global `mouse-axis-map` supplies baseline `C`; baseline `G` is `1` for every
output. In directional mode use `C[o,i+]` and `C[o,i-]` instead. The mode remains
configuration-wide in version one; layers cannot change it.

Resolve `C` and `G` independently, then compile the effective coefficients:

```text
linear:       E[o,i]  = G[o] * C[o,i]
directional:  E[o,i+] = G[o] * C[o,i+]
              E[o,i-] = G[o] * C[o,i-]
```

Output gain is a separate logical parameter, not a destructive rewrite of the
inherited row. Thus a lower `G[x] = 0.25` and higher `C[x,y] = 2` produce an
effective coefficient of `0.5`. Setting that coefficient does not erase the
precision gain. To demand effective coefficient `2` independently of lower
output gains, also set `G[x] = 1` (affecting the whole X row). Introspection must
show both the logical parameters and the compiled product.

This separation is intentional: independent layers can change routing and
sensitivity without having to know each other's matrix entries. An absolute
operation is absolute **for its named parameter**, not for the entire transform.

### One operation per scalar per layer

| Declaration | Function of inherited value `p` | Meaning |
| --- | --- | --- |
| Omitted or `null` | `p` | Transparent inheritance |
| Number `n`, or `(set n)` | `n` | Absolute replacement |
| `(add d)` | `p + d` | Add to the inherited parameter |
| `(mul k)` | `p * k` | Multiply the inherited parameter |

Use `mul`, not a percent: `0.25` means one quarter and `2` means double. Numbers
are signed finite floating-point values, consistent with current map gains.
Negative output gains reverse an output; negative coefficients choose an output
direction. `null` is a literal option value, not a key action or a numeric value.

`0` and `(set 0)` suppress that parameter. `(mul 0)` also yields zero, while
remaining visibly a multiplier in provenance. Neither is inheritance. `(add 0)`
and `(mul 1)` have no numerical effect but remain explicit operations. Null does
not restore the global value, remove a lower declaration, or select identity.

For example, inherited coefficient `C[x,v] = 0` behaves as follows:

| Higher declaration | Result | Route created? |
| --- | ---: | --- |
| `null` | 0 | No |
| `(mul 4)` | 0 | No |
| `(add 0.25)` | 0.25 | Yes |
| `0.25` | 0.25 | Yes |
| `0` with inherited coefficient 0.25 instead | 0 | Route suppressed |

Multiplying a zero off-diagonal entry cannot create a route. Adding to a
coefficient can; adding to an output gain cannot resurrect an all-zero row.

### Algebra, order and no drift

For each parameter, fold operations from lowest to highest priority:

```text
p_effective = op_high(...op_second(op_low(p_baseline))...)
```

Every operation is affine in the *parameter*: `f(p) = a*p + b`. Inheritance is
`(a,b)=(1,0)`, set is `(0,n)`, add is `(1,d)`, multiply is `(k,0)`. Composing a
higher `(a2,b2)` over a lower `(a1,b1)` yields
`(a2*a1, a2*b1+b2)`. This describes resolution, not motion-space matrix
multiplication. Implement a direct, deterministic fold first; regrouping
floating-point operations can alter rounding.

| Baseline | Lower operation | Higher operation | Result |
| ---: | --- | --- | ---: |
| 2 | `(mul 3)` | `(add 1)` | 7 |
| 2 | `(add 1)` | `(mul 3)` | 9 |
| 2 | `(set 4)` | `(mul 3)` | 12 |
| 2 | `(mul 3)` | `(set 4)` | 4 |

Set hides lower values for that parameter. It does not hide unrelated parameters.
Two adds commute mathematically, as do two multipliers; mixed operations do not
generally commute. Distinct parameter slots resolve independently.

Never undo an operation on release by division or subtraction. Never start a
new resolution from yesterday's effective settings. Recompute from immutable
baseline plus the current declarations, or use a cache with exactly those inputs.
Zero multipliers and absolute replacement make inverse-based restoration wrong.

Suppose gain baseline is `1`, with held layers `S=(set 2)`, `M=(mul 3)`, and
`A=(add 0.5)` in that activation order:

| Active low-to-high declarations | Gain | Explanation |
| --- | ---: | --- |
| None | 1 | Baseline |
| S | 2 | Absolute |
| S, M | 6 | Inherited multiplier |
| S, M, A | 6.5 | Inherited addition |
| M, A after releasing S | 3.5 | Rebuilt from 1; top A never changed |
| A after releasing M | 1.5 | Rebuilt from 1 |
| None after releasing A | 1 | Exact baseline resolution |
| S, A if M alone is removed from the full stack | 2.5 | Set then add |
| S, M if A alone is removed from the full stack | 6 | Set then multiply |

Repeated input reports do not repeat these operations. A reload discards the old
configuration's resolver cache and resolves the *new layout's actual state*;
it does not transplant old layer indices or multiply the old effective result.
If the new baseline is `4`, `M,A` would yield `12.5` if those layers are active
in that new state. If reload produces only the default layer, resolve that state
instead. Layer-state migration is not introduced by this proposal.

## Which layers participate

**Use all held layers and the selected default layer, independently of keyboard
transparent-key policy.** Highest priority is the newest surviving held
activation. Global settings are below every layer. Do not implicitly insert the
first-defined layer when a different default is selected: put shared motion
defaults in the global map.

Precise construction:

1. Take `active_held_layers()` in its current newest-first order.
2. Append `default_layer`.
3. Deduplicate layer IDs, retaining their first (highest-priority) occurrence.
4. Reverse this list and fold its declarations over the global baseline.

Consequences:

- A held layer applies once, even if two physical keys or a physical and virtual
  key hold it. It is not a stack of multiplier instances.
- The most recent surviving activation fixes that layer's priority. Releasing
  it can expose an older instance and change ordering; it never requires an
  arithmetic undo. Example: states oldest-first `L, M, L` resolve `M, L`;
  releasing the newest L resolves `L, M`.
- If the default layer is also held, its declaration applies once at the held
  occurrence's priority. Without a held occurrence it is the lowest layer.
- Switching default layers replaces that contribution even with an unchanged
  held top layer. Releasing any lower layer likewise requires a fresh stack key.
- A layer with absent/null motion settings still affects keyboard behavior but
  contributes no parameter operations. One-shot or virtual-key layer states
  participate while Keyberon actually exposes them as held layer states.
- Configuration definition order does not rank held layers. Equal-looking
  declarations on different layer IDs remain separate operations.

The inspected `trans_resolution_layer_order()` has two behaviors. With
`trans_resolution_behavior_v2`, it returns held layers, default, and sometimes
layer zero for `delegate_to_first_layer`; the older branch returns current layer
and sometimes layer zero. Neither is the proposed motion contract. Copying the
older branch would silently lose lower precision/boost declarations; copying
keyboard fallback would make pointer sensitivity depend on a keyboard lookup
option. This proposal deliberately makes motion settings stable across those
keyboard variants. It is a consequential behavior to accept before coding.

## Proposed configuration surface

### Sparse layer header option

Extend the existing parenthesized layer header with **one** new option:

```lisp
(deflayer (precision mouse-axis (
    output-gain ((x (mul 0.25)) (y (mul 0.25)))
    coefficients null
  ))
  ;; ordinary defsrc-aligned key actions follow here
)
```

The same header works with `deflayermap`. `output-gain` is an axis-to-operation
list. `coefficients` uses output rows and input-to-operation pairs, but **patches
individual entries**, unlike global `mouse-axis-map`'s complete rows. Avoid a
layer option also named `mouse-axis-map`: its existing replacement semantics
would make sparse inheritance surprising.

For example, `mouse-axis (coefficients ((x (y (mul 0.5)))))` changes **only**
`C[x,y]`. No other entry has to be written. Adding `(x null)` as another input
term in that same X row explicitly documents inheritance of `C[x,x]`, with
exactly the same result as leaving that term out.

Exact proposed empty/null rules:

| Position | Omitted / `null` | Empty list |
| --- | --- | --- |
| Entire `mouse-axis` option | No operations | No operations |
| `output-gain` or `coefficients` field | No operations in that family | No operations |
| Named output gain, e.g. `(x null)` | Inherit X gain | Invalid scalar operation |
| Coefficient row, e.g. `(x null)` | Inherit every X coefficient | Bare row `(x)` is a no-op patch |
| Coefficient, e.g. `(x (y null))` | Inherit `C[x,y]` | Invalid scalar operation |

In `coefficients`, an absent term also inherits; it does not become zero. To
silence an output use `output-gain ((x 0))`. To disable one route use
`coefficients ((x (y 0)))`. An explicit row `(x null)` cannot also contain terms.
The empty *global* row `(x)` still suppresses X; its distinct complete-map
semantics remain documented separately.

Reject duplicate fields, output rows, gain entries or coefficient entries even
when one is null. Reject unknown names, malformed operations, operation lists
with extra arguments, NaN and infinities. One slot has one operation per layer;
do not introduce nested arithmetic expressions or operation chains in version
one. Users needing both add and multiply can use separate layers; an affine
expression shorthand could be considered only after a concrete need.

The mode fixes coefficient input spelling: unsuffixed inputs in linear mode,
`x+`, `x-`, etc. in directional mode, as in the current parser. Output gain
names are always unsuffixed outputs. Signed zero may be normalized to positive
zero for stable equality; it has no inheritance meaning. The parser should use
the existing variable-resolution conventions for nested option expressions.

### Complete prospective precision, boost and wheel example

The `defcfg` map and actions here exist today. The `mouse-axis` options do not.
This example rotates physical X/Y and makes F1 precision, F2 faster scrolling,
F3 pointer boost, and F4 a virtual-key toggle of the same precision layer.

```lisp
(defcfg
  delegate-to-first-layer yes
  mouse-axis-map ((x (y 1)) (y (x -1)))
)
(defsrc f1 f2 f3 f4)
(defvirtualkeys precision-control (layer-while-held precision))
(deflayer base
  (layer-while-held precision)
  (layer-while-held wheel-fast)
  (layer-while-held boost)
  (on-press toggle-virtualkey precision-control)
)
(deflayer (precision mouse-axis (
    output-gain ((x (mul 0.25)) (y (mul 0.25)) (v null) (h null))
  )) _ _ _ _)
(deflayer (wheel-fast mouse-axis (
    output-gain ((v (mul 2)) (h (mul 2)))
  )) _ _ _ _)
(deflayer (boost mouse-axis (
    output-gain ((x (mul 2)) (y (mul 2)))
  )) _ _ _ _)
```

For a report `X=8, Y=4, V=120, H=0`, before quantization:

| Active motion layers | X output | Y output | V output |
| --- | ---: | ---: | ---: |
| None | 4 | -8 | 120 |
| Precision | 1 | -2 | 120 |
| Wheel-fast | 4 | -8 | 240 |
| Precision + wheel-fast | 1 | -2 | 240 |
| Precision + boost | 2 | -4 | 120 |
| Precision held physically and virtually | 1 | -2 | 120 |

The rotation's diagonal coefficients are zero. Multiplying those zeros would
do nothing; output gains instead scale the nonzero off-diagonal routes. Pointer
and wheel layers touch disjoint slots. In this example their activation order
is immaterial. Precision and boost also commute because both use multiplication.

For an absolute precision choice, replace `(mul 0.25)` with `0.25` or
`(set 0.25)`. A higher absolute precision layer masks a lower boost gain; a boost
activated above it still multiplies it. The layer system supplies priority, not
a separate category of permanently dominant precision settings.

Virtual keys need no new transform action: their existing held layer action
creates the same layout state. A momentary controller can use the existing
`(multi (on-press press-virtualkey precision-control)
(on-release release-virtualkey precision-control))` on a key. A tap may activate
and release before any motion report; it does not promise a measurable precision
interval. Sustained virtual-key holds or toggles express a persistent mode.

### Sparse coefficient changes and explicit inheritance

This prospective option adds a wheel-to-X route while preserving every other
inherited X coefficient, and explicitly removes an inherited X-to-Y route:

```lisp
mouse-axis (
  output-gain ((x null) (y 1))
  coefficients (
    (x (x null) (y null) (v (add 0.25)))
    (y (x 0))
    (v null)
  )
)
```

With the quarter-turn baseline and no other layers, `C[x,v]` rises from zero to
0.25; one vertical notch contributes 30 X counts. Physical Y still contributes
to X. Y output becomes zero because its only nonzero baseline route was X-to-Y.
V scrolling remains present: routing a wheel into X does not implicitly remove
its existing V output. Add `(v 0)` to the `output-gain` list to suppress V.
If a lower precision layer exists, `x null` inherits its gain and the 30 X counts
become 7.5 before quantization. The Y gain set to 1 does not rebuild its row.

Full row/profile replacement is not needed for the initial use cases. A caller
can explicitly set all supported entries, including zeros, when a complete
replacement is genuinely wanted; this is verbose but unambiguous. Do not pretend
that setting just the visible nonzero terms removes inherited hidden routes.
A later named complete-map replacement could reduce that verbosity if demanded.

### Directional branches and Linux-native channels

Start from the existing `defcfg mouse-axis-map-mode directional` with identity
rows. This prospective layer option makes wheel-up twice as strong, wheel-down
quarter strength, pointer-up half strength, and adds a Linux dial route:

```lisp
mouse-axis (
  coefficients (
    (v (v+ (mul 2)) (v- (mul 0.25)))
    (y (y- (mul 0.5)) (y+ null))
    (rz (dial+ (set 0.5)) (dial- (set -0.5)))
  )
)
```

The identity negative coefficients are `-1`: the resolved wheel-down coefficient
is `-0.25`, and pointer-up is `-0.5`. Input `V=-120` therefore emits `V=-30`, not
positive 30. `(add 0.25)` on that inherited `-1` would produce `-0.75`; add is
signed coefficient arithmetic, not an increase in absolute speed. Use multiply
for sign-preserving speed changes or set an explicit signed value.

Directional selection sees the original input axis's signed net report delta;
each selected branch supplies a positive magnitude. An output gain multiplies
both branches after selection; a negative output gain does not reclassify the
physical branch. There are ten output rows and twenty possible input branches,
not twenty independently emitted output directions.

The dial example is Linux/Android-specific, like the current axis vocabulary.
V/H high-resolution companions remain normalized representations of the same
wheel channels. Keep `x y v h z rx ry rz dial misc` wherever the backend supports
them; reject unsupported declarations rather than dropping them to a four-axis
common denominator. The layer feature does not implement macOS axis capture.

### Output versus input gain, units and zero input

The recommended `output-gain` scales a destination, irrespective of its sources.
If V is routed to X, X precision slows that contribution; a V output gain changes
scroll output, not the wheel's contribution to X. This makes the name important.
An input gain would instead scale a source column across all destinations. The
first version can express that explicitly by multiplying its coefficient in
each relevant row, including each desired directional branch. A separate
input-gain family is useful future shorthand, not required to deliver the
commission's individual-coefficient freedom.

Gains and their additive adjustments are dimensionless. `C[o,i]` has units of
output units per input unit; `(add d)` uses those same coefficient units, while
`(mul k)` is dimensionless. Pointer and extra axes use native relative counts;
wheel units are 1/120 notch. Platform processing remains as documented: Linux
uses native pointer counts, LLHOOK screen displacement after desktop processing.

For normalized report `u`, motion before output quantization is:

```text
linear:      y[o] = sum_i E[o,i] * u[i]
directional: y[o] = sum_i E[o,i+] * max(u[i], 0)
                       + E[o,i-] * max(-u[i], 0)
```

Consequently zero input contributes zero, even with additive parameter changes.
Changing a setting emits no movement and no wheel key tap. Existing per-output
fractional credit is accounting for prior input, never a constant-motion term.
Generated `movemouse-*` and `mwheel-*` outputs remain outside this physical-input
transform, so there is no feedback through their own layer-controlled mapping.

## Parameter composition is not a transform pipeline

Layer operations compose scalar parameter values. Compiling `E = diag(G) * C`
is a specific output-scaling step, not a promise to multiply arbitrary matrices
in layer order. Setting coefficients can directly describe swaps, rotations and
shears, but `(mul 2)` on a coefficient is an element operation, not composition
with a second transform. `G` and `C` resolve independently even when their
operations originate on different layers.

For linear transforms, an explicit pipeline would add a different capability:
apply an arbitrary transform relative to whatever mapping is inherited without
enumerating its coefficients. With swap `S` and X-only scale
`D=diag(0.25,1)`, input `(2,4)` becomes `(1,2)` under `D*S`, but `(4,0.5)` under
`S*D`. Output gain selects the first kind of scaling. Arbitrary inherited rotation
composition would need matrix products or a staged transform interface.

Directional pipelines are more general still. First mix `s=x+y`, then emit
`abs(s)` using positive/negative branches. This computes `abs(x+y)`, whose
boundary is `x+y=0`, not a physical input sign boundary. A single map with
independently split original X/Y inputs cannot represent it: matching outputs
1 for `(1,0)` and `(0,-1)` would sum to 2 for `(1,-1)`, whereas `abs(x+y)` is 0.
Do not collapse such pipelines into an ordinary matrix, or even claim that the
current directional representation is closed under arbitrary composition.

**Recommendation:** ship parameter inheritance first. Explicit transform stages
would add inherited coordinate-space operations and post-mix direction tests;
they also require stage order, intermediate units and quantization decisions.
Those are meaningful future freedoms, not hidden meanings of `mul`. The current
precision, independent-wheel, coefficient and directional-edit cases need none
of that machinery.

## Runtime design boundary

### Compile immutable declarations, publish immutable effective settings

Recommended internal separation, descriptive names rather than an API promise:

```text
MotionConfig { config_generation, baseline, layer_patches, capture_requirements }
ResolvedMotion { config_generation, canonical_layers, gains, coefficients,
                 effective_map, output_reset_epochs, provenance }
AxisState { last_seen_output_epochs, output_fractions, wheel_remainders }
```

Extend [layer header parsing](/parser/src/cfg/deflayer.rs) and
[option parsing](/parser/src/cfg/layer_opts.rs) with typed option values; retain
icon behavior. Carry the compiled patch with `LayerInfo` or a correspondingly
indexed immutable table assembled in [cfg/mod.rs](/parser/src/cfg/mod.rs).
Reuse axis-name/support checks and global baseline compilation from
[defcfg.rs](/parser/src/cfg/defcfg.rs), but implement a separate sparse patch
parser: its missing-entry meaning differs. No new Keyberon action is needed.

In [runtime processing](/src/kanata/mod.rs), derive the canonical stack after
layout/custom-action processing, including virtual-key actions, and after any
external default-layer mutation or successful reload. Comparing the small stack
each processing tick is an adequate first implementation; a mutation generation
counter can be introduced later if justified. Keep this separate from the
current top-layer notification hook, which can continue to report the visible
keyboard layer.

Resolver cache identity is `(config_generation, canonical_layer_ids_in_order)`.
Do not key only on `current_layer`, effective-map equality, or raw pointer
identity. Holding/releasing an extra duplicate with unchanged canonical order
does not require a numerical recomputation. Changes in declaration provenance
on reload do require a new configuration generation even if numbers match.
Raw activation details can be recorded separately for diagnosis.

Resolve into a new value off the publication lock, validate it, then replace the
published snapshot atomically. The existing mutex is sufficient; a lock-free
structure is not a prerequisite. Consumers take one snapshot for a native
report/stroke/callback and use it throughout normalization, transformation and
emission. Never expose half a new row or gain with an old coefficient table.

Finite literals do not guarantee finite composed arithmetic: repeated large
multipliers or `G*C` can overflow. Check every resolved operation and compiled
coefficient. Reject an invalid initial/reloaded configuration before publication.
If a newly active combination overflows, surface a contextual resolver error
through the processing error boundary rather than publishing NaN/Inf, silently
clamping parameters or silently retaining a different layer's settings. Preserve
the existing separate policy for saturation of extreme *motion outputs*.

### Publication ordering is an observable contract

Use **the latest fully processed layout state at motion snapshot acquisition**.
A layer operation takes effect for reports that acquire their snapshot after
its publication, not halfway through a report. A mapped wheel report that
produces a key tap capable of changing layers uses the old snapshot for that
report; the resulting layout action can influence subsequent reports only.

The current [Linux event loop](/src/kanata/linux.rs) transforms a report before
submitting its keyboard events, and the keyboard queue is asynchronous. Therefore
even a physically preceding layer key may not yet have been processed when a
motion report arrives. Atomic publication alone does not solve cross-thread
ordering. The first implementation should state this processed-state contract
and test physical precision activation responsiveness on the existing adapters.
If strict physical key/report ordering is required, ordered input envelopes and
processing barriers are a separate architectural change to commission, rather
than a guarantee hidden in this resolver. Do not synchronously wait from a
Windows hook callback without reviewing its existing injection/thread design.

### Capture must cover inactive possibilities

A global identity map with only an inactive precision layer still needs pointer
capture at startup. Inspect **all declared layer patches**, not just the startup
effective map, and retain a stable capture capability set across activations.

Compile a conservative source dependency union:

- Include inputs already requiring interception in the baseline.
- Include every input named by a non-null coefficient operation, even if its
  current value happens to stay zero.
- For each output with a non-null gain operation, include every source that
  could feed it: baseline nonzero routes plus routes any layer can introduce
  with set/add. Include both directional branches' source axes. Multiplication
  alone does not introduce a route from zero.

This need not enumerate every ordering. It may capture more than a particular
reachable state needs, but must never miss a cross-layer route. A declared gain
on V can require X interception if another layer routes X into V. Apply OS
capability validation to inactive declarations too. Null-only patches contribute
no capture requirement; harmless explicit operations may conservatively do so.

Feed this analysis into Linux's device-detect default derivation in
[cfg/mod.rs](/parser/src/cfg/mod.rs), the
[Interception startup filter](/src/kanata/windows/interception.rs), and the
[LLHOOK startup gate](/src/kanata/windows/llhook.rs). Respect explicit device
selection and existing backend restrictions. On reload, compare requested
capabilities with those actually installed: report restart requirements where
they cannot be expanded live, even when the new capability is only in an
inactive layer. Do not claim successful end-to-end adoption merely because the
new map parsed. The existing restart limitations remain platform facts.

### Rounding and state transitions

Keep the existing motion arithmetic: normalize wheel companions, select input
branches, sum all contributions per output, then quantize once with signed
fractional accumulation and existing saturation behavior. Do not round each
route independently. Keep high-resolution wheel accounting and notch/key
conversion after that stage.

For **discrete layer changes**, recommend a modest refinement of the present
reset policy: reset only outputs whose compiled effective row changes. In
directional mode compare both branch rows. Reset that output's fraction; for
V/H also reset its raw-notch and key-notch remainders. An unchanged wheel row
should not lose partial scroll just because pointer precision changed.

- Changing gains/coefficients with an identical compiled row changes provenance
  but does not reset that output's credit.
- Muting, unmuting, inverting or rerouting an output changes its row and discards
  that output's old credit; it does not rescale or emit the credit.
- Successful configuration reload resets all output state explicitly, even when
  numeric rows match. Mode/backend/topology changes are full discontinuities.
- Releasing a layer restores settings, not its historical fractional credit.
  Discrete transitions can therefore discard subunit motion; this is deliberate.

Publish per-output reset epochs, and increment them when the resolver observes
these discontinuities. Each adapter-local `AxisState` notices epochs even on an
identity/passthrough path. Equality against the last map *seen by the reader*
is insufficient: a processed A → B → A transition between motion reports must
still discard the old A credit. If no effective B snapshot was ever published,
there was no observable B regime and no such transition to account for.

This changes current all-output-reset behavior only for layer transitions and
requires explicit implementation/checks; it is not a claim about today's code.
The reload-wide reset is an explicit new generation boundary rather than a
side effect of map inequality. Retaining fractions through continuous animation
is a different future policy, below.

### Explain the resolved result

Keep source spans and layer IDs alongside compiled operations. A debug explanation
should be able to show, for example:

```text
G[x]: baseline 1 -> precision mul .25 -> boost mul 2 = .5
C[x,y]: global 1 -> edit null = 1
E[x,y]: .5 * 1 = .5
order: base, precision, edit, boost; config generation: N
```

For set, mark preceding contributions as masked for that slot; for explicit null,
show the transparent declaration when explaining that slot. Explain absent
values as inherited without allocating a trace for every absent entry. Retain
the source location of the baseline row, including whether it came from default
identity. Log effective changes separately from top-layer changes. Provenance
need not travel as large strings through the input callback; a snapshot can
reference immutable configuration data. A new public introspection protocol is
not required for the first slice, but debuggable source attribution is.

## Deferred animation connection

The maintained desire and rough sketch remain in the
[parent ask](layer-transform-ask0.gpt6a.md#animation-desire); this document does
not duplicate an animation design. Layer patches give future envelopes useful
typed targets: output gains and individual linear/directional coefficients.
Set/add/multiply can reuse this resolver to choose a target from inherited
values. An envelope must not reapply `(mul 2)` to yesterday's interpolated value
on every tick, creating an accidental exponential ramp.

Separate target-resolution invalidation from interpolation samples. A future
continuity-preserving envelope will usually retain output fractions across
small coefficient updates; incrementing today's discrete reset epochs on every
sample could starve low-speed motion. Retargeting, routing discontinuities,
wheel accounting and mode changes need explicit policies before animation lands.
Stable parameter identities, source provenance and distinct reset epochs leave
room for those decisions without settling them now.

Virtual-key-driven ramps and sustained-input triggers remain **desired, deferred
capabilities**, not consequences already delivered by a held layer. What counts
as sustained input, its observation point, time/distance basis, hysteresis and
idle behavior remain open in the parent ask. Arbitrary interpolated matrices
also need not remain rotations; the resolver does not solve that geometry.

## Next coherent slice and meaningful checks

After accepting the decisions below, implement one usable vertical slice:
typed layer patches for both gains and coefficients, the pure resolver, coherent
runtime publication, capture requirements and explicit state resets, with
physical precision and wheel examples in the maintained configuration reference.
Preserve global-only behavior. Do not start with a speed-only special action that
will later need replacement to obtain the requested coefficient freedoms.

Verification should establish properties that distinguish this design:

1. **Parser contract:** null/omitted/zero/empty behavior at every supported level;
   duplicate and malformed rejection; existing global complete-row behavior;
   both layer-header forms; directional suffix and platform checks.
2. **Resolution:** the numerical tables above; all held/default order cases;
   duplicate activations and release of the newest duplicate; default changes
   and lower-layer release under an unchanged top; invariance to keyboard
   transparent-resolution flags; finite-operation overflow diagnostics.
3. **No drift:** repeated ticks/reports leave effective settings unchanged;
   arbitrary activation/release sequences produce the same final parameters
   as a fresh fold of that final canonical stack; zero/set transitions restore
   correctly without inverses; reload does not reuse old layer indices.
4. **Actual math:** rotation plus precision; disjoint wheels; zero off-diagonal
   multiplication versus route creation by add; signed negative branches;
   Linux dial/RZ routes; zero input never emits motion just because a parameter
   was increased. Check cross-unit wheel-to-pointer examples explicitly.
5. **State:** small repeated deltas accumulate; contributions cancel before
   rounding; pointer-only transitions preserve wheel credit; changed rows drop
   their credit; equal numeric results preserve credit; A → B → A between
   reports resets via epochs; reload resets even an equal map.
6. **Adapter integration:** inactive layer requirements install startup capture;
   cross-layer source dependencies are included; one report uses one snapshot;
   virtual and physical holders resolve identically; unsupported reload capture
   expansion is diagnosed; synthetic reports plus a real pointer/wheel exercise
   establish activation responsiveness and reveal processed-state latency.

These are recommended future checks, not tests run for this proposal. Existing
map/axis tests provide starting fixtures; broader native-backend behavior needs
its own receipts rather than being inferred from a pure resolver test.

## Decisions to accept before implementation

The recommendation is coherent as a whole; the consequential choices are:

1. **Sparse scalar inheritance:** output gains plus individual coefficients,
   `null` as transparent, number/set/add/mul as above; no full transform pipeline
   or mode switching in version one. Absolute operations act on the named slot.
2. **Full active stack:** newest-held priority, selected default underneath,
   once per layer at its highest surviving occurrence, independent of keyboard
   transparent-key flags and without automatic first-layer fallback.
3. **Boundary behavior:** latest-processed-state report snapshots; per-changed-row
   fractional reset on discrete layer transitions and a full reset on reload.
   Strict physical key/report ordering would require a separate architectural
   decision; animation will require a continuity policy of its own.

Syntax spelling can be refined without changing that contract. The immediate
payoff is ordinary physical-pointer precision/boost that preserves rotations,
with the same operations available to wheels, Linux native channels and
individual directional routes.

## Connections and ownership

- [Commission and animation ask](layer-transform-ask0.gpt6a.md): authority for
  this design's scope and the deferred opportunity; this proposal answers its
  inheritance questions without promoting them to accepted runtime behavior.
- [Linux flexibility](linux-flexibility0.gpt6a.md): source/device identity and
  output endpoints are orthogonal extensions. Its destination-local state
  guidance supports keeping quantization at outputs. Future qualified parameter
  identities can include endpoints, but endpoint expansion is not a prerequisite.
- [Design reading map](README.md): maintained by the parent session, which can
  link this independent proposal and subsequent discussion without rewriting it.
- [Configuration reference](/docs/config.adoc): implemented behavior remains
  authoritative there; prospective examples here should move into that account
  only when their parser/runtime support exists and has been checked.
