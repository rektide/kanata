---
type: Design
title: Linux axis flexibility beyond a single virtual mouse
description: Independent Linux-first exploration of generated axes, device-qualified routing, directional branches, and consumer-specific output devices.
resource: /design/mouse-axis/linux-flexibility0.gpt6a.md
status: draft
author: openai/gpt-6-astra#medium
date: 2026-10-29
sources:
  - resource: opencode://ses_e7a8c71fdffen9QBXXuNIZgrM8
    title: Independent Astra medium research and design of Linux axis extensibility
  - resource: opencode://ses_e7b7b7559ffeEC9DfB7nsFmtYO
    title: Commissioning context and reported Keychron capabilities
  - resource: /parser/src/mouse_axis.rs
    title: Existing ten-channel transform and fractional quantization
  - resource: /src/kanata/mouse_axis.rs
    title: Shared wheel routing and remainder state
  - resource: /src/kanata/linux.rs
    title: Native relative-axis encoding and report normalization
  - resource: /src/oskbd/linux.rs
    title: Input flattening and virtual output capabilities
  - resource: /docs/config.adoc
    title: Current configuration account of native axes and unqualified device scope
  - resource: https://docs.kernel.org/input/event-codes.html
    title: Linux input event semantics and report boundaries
  - resource: https://docs.kernel.org/input/uinput.html
    title: Userspace creation of virtual input devices
  - resource: https://docs.kernel.org/hid/hidraw.html
    title: Unmodified HID reports and descriptor access
  - resource: https://docs.kernel.org/hid/uhid.html
    title: Userspace HID transport and virtual HID devices
  - resource: https://wayland.freedesktop.org/libinput/doc/latest/what-is-libinput.html
    title: Desktop input scope and explicitly unsupported joystick devices
  - resource: https://wayland.freedesktop.org/libinput/doc/latest/device-configuration-via-udev.html
    title: Device classification, seats, and explicit libinput exclusion
  - resource: file:///usr/include/linux/input-event-codes.h
    title: Locally inspected REL and ABS namespaces
---

# Linux axis flexibility beyond a single virtual mouse

## Answer first

**Yes: a wheel can already generate an output axis that its physical device
does not have.** The current Linux implementation can turn vertical scrolling
into `z`, `rz`, `dial`, ordinary pointer motion, or several outputs together.
Its input and output sets need not have equal sizes. The ten-channel internal
array is a representation, not a requirement that every input have one output.

Linux can go further: create several virtual devices, give each its own axes,
and route independently identified inputs to them. This makes additional wheel
identities meaningful to applications that preserve device identity. Linux can
also expose virtual absolute controllers or HID devices for suitable consumers.

The limits are at two different places:

1. **Information arriving:** if two controls have become the same observable
   stream, software cannot reliably recover which control produced each delta.
   Device identity or raw HID fields can sometimes preserve information that a
   higher-level interface discarded.
2. **Meaning at the destination:** generating `REL_Z` does not make the desktop
   cursor three-dimensional. A receiving application must consume that channel
   and assign it a useful meaning.

The best next extension is **device-qualified input reports and named virtual
output endpoints**, with Linux-native capabilities. It builds on working axis
math without making another OS's smaller mouse interface the design ceiling.

## Ground and evidence

This is an independent proposal, not implementation authorization or accepted
configuration syntax. Code observations refer to the implementation at
`dc213b63`, inspected in the working tree before the parent's directional-mode
work. Sources above were read on 2026-10-29. Official kernel and libinput docs
support interface claims; this exploration did not run physical-device captures
or verify application compatibility experimentally.

The commissioning session reports Keychron Ultra-Link 8K capability masks
`REL=1943` on one interface and `REL=1040` on a keyboard interface. These indicate
X/Y, V/H and high-resolution companions on the former, H and its high-resolution
companion on the latter. **Capabilities do not prove which interface each
physical wheel uses, or whether an event is duplicated.** That requires turning
one control at a time while recording all relevant event nodes.

Cross-reference: the `mouse-axis-map` section of the maintained
[configuration reference](/docs/config.adoc) explains the current native units,
extra channels, and all-intercepted-devices scope. This proposal extends that
scope with qualified endpoints; the reference remains the place to check
implemented configuration behavior as the parent's directional work lands.

Local third-party archive discovery found libevdev and a libinput organization
directory containing libei, but no libinput implementation checkout. Focused
official documentation supplied the desktop-stack evidence.

### What works in the current implementation

- [The parser transform](/parser/src/mouse_axis.rs) exposes `x y z rx ry rz v h
  dial misc`; explicit output rows replace identity, omitted rows keep identity,
  and empty rows suppress output. Inputs refer to the original report, not the
  results of preceding rows.
- [The Linux adapter](/src/kanata/linux.rs) normalizes legacy wheel deltas to
  120 units per notch, avoids counting a report's high-resolution companion
  twice, and emits all ten native relative channels.
- [The uinput setup](/src/oskbd/linux.rs) already advertises these channels.
  Generated Z or dial values therefore require neither a physical Z/dial input
  nor a new output-device capability in this implementation.
- [Shared state](/src/kanata/mouse_axis.rs) retains fractional output and wheel
  remainders. Generalization must preserve these semantics deliberately.

For example, this uses the existing row syntax inside `mouse-axis-map`:

```lisp
((z (v 0.008333333333333333))
 (rz (h 0.008333333333333333))
 (dial (v 0.008333333333333333) (h 0.008333333333333333))
 (v)
 (h))
```

One vertical notch generates approximately one native Z count and one dial
count; one horizontal notch generates approximately one RZ count and one dial
count. V/H scrolling is suppressed; X/Y retain identity. These are generic
native counts, not automatically millimetres, degrees, or application actions.
This illustrates source-level capability, not an observed CAD integration.

## Where Linux's flexibility lives

| Interface | What can be represented | What it does not promise |
| --- | --- | --- |
| evdev `EV_REL` | Named relative codes on each event device | Unlimited independently named wheels on one node |
| uinput | Newly created devices with chosen supported event types/codes | Automatic consumer recognition of every code |
| `EV_ABS` | Stateful axes such as X/Y/Z, rotations, throttle and rudder | Correct gamepad identity or conversion of deltas into positions without policy |
| hidraw | Original input reports, including numbered-report IDs and fields | Information that firmware never transmitted |
| UHID | A virtual HID device described by reports and a descriptor | An escape from evdev's namespace when a consumer still reads evdev |
| Application-specific channel | Arbitrary logical controls with an agreed protocol | Compatibility with unmodified desktop applications |

The local Linux header defines ten logical REL axes plus two high-resolution
wheel companions. `REL_RESERVED` is explicitly not for driver use; `REL_MAX`
is an ABI bound, not a promise of free user-defined wheel slots. `REL_MISC` is
one code, not a collection of independently named values. Do not invent
`REL_WHEEL_3` by assigning unused integers.

Repeated `REL_WHEEL` events on the same device can express multiple deltas, but
there is no standard per-event wheel-ID field. Timing or position within a
report is not a reliable identity contract. The useful identity is ordinarily
**(event device, event type, event code)**.

The [libinput scope documentation](https://wayland.freedesktop.org/libinput/doc/latest/what-is-libinput.html)
also explains why kernel capability and desktop usefulness differ: libinput
serves conventional desktop interactions and explicitly does not handle
joysticks. Typical Wayland applications receive compositor-mediated events,
not raw event-device channels. Multiple mice usually feed the same desktop
pointer; extra virtual mice do not automatically produce extra cursors.
Seat/compositor arrangements can change that, but constitute separate
integration work. Direct evdev consumers can retain identities and extra codes.

## Design A: qualified routes over native endpoints

### Preserve identity before expanding syntax

`KbdIn::read` currently returns `Vec<InputEvent>` after extending a common vector
from each polled device. The reader knows the device token and path, but drops
them before returning. Consequently even a sophisticated downstream map cannot
distinguish two captured devices' `REL_WHEEL` channels.

Proposed internal report envelope:

```text
SourceReport {
  source_instance, configuration_alias, timestamp,
  events_through_syn_report
}
```

The instance identifies one attachment; the alias identifies a configured
logical source across reconnects. Preserve per-device partial-frame state if
reads can split reports. Never deduplicate legacy/high-resolution wheels
across different source instances. `SYN_DROPPED` is a discontinuity: discard the
damaged packet and recover state according to evdev rules; relative motion
lost to overflow cannot be reconstructed from a current position query.

Source selection should support a combination of bus/vendor/product, interface,
serial/unique identity where present, and physical path where needed. Names
alone and `/dev/input/eventN` numbers are weak identities. A selector matching
several devices should require explicit plural matching or report ambiguity.
Absence can be a declared hotplug-wait state; do not silently substitute a
different mouse. Discovery should display matching nodes and advertised axes.

### Separate logical channels from destination encoding

Conceptually name channels by `(endpoint, axis)`; the endpoint owns its backend
and capabilities. Retain the current map as a convenient single-endpoint form.
Compile advanced routes into sparse source-to-destination edges rather than
expanding one global square matrix whenever another device is added. Small
native arrays can remain inside individual endpoints.

All examples below are **prospective route notation, not Kanata syntax**:

```text
source keychron.pointer = <selector for the verified pointer event node>
source keychron.aux     = <selector for the verified auxiliary event node>
output desktop         = uinput relative-pointer
output orbit           = uinput relative-controller { z, rz, dial }

keychron.pointer.v -> desktop.v * 1
keychron.pointer.h -> orbit.rz * (1 / 120)
keychron.aux.h     -> orbit.dial * (1 / 120)
```

This gains an independent auxiliary wheel only if capture demonstrates an
independent auxiliary stream. If it duplicates the primary interface, routing
both produces duplicate effects. Deduplication must follow a verified device
profile or explicit user selection, not a timing heuristic.

Output profiles should describe native capabilities and intended consumer:
ordinary mouse, raw relative controller, or later absolute controller. A raw
relative controller need not pretend to be a mouse. It can be consumed directly
even if libinput ignores it. Conversely, a desktop pointer profile needs the
appropriate mouse capabilities/classification. Merely setting a udev type label
does not guarantee libinput acceptance, as its
[classification documentation](https://wayland.freedesktop.org/libinput/doc/latest/device-configuration-via-udev.html)
states. Explicit exclusion from libinput is useful for application-only devices.

### Collision, ordering, and state semantics

- Several source contributions to the same destination axis sum within one
  source report before destination quantization. Different devices' reports
  remain separate; do not fabricate a shared simultaneity window.
- A relative source absent from a report contributes zero, not its previous
  delta. Later absolute-source support would need a different state model.
- Keep output quantization and legacy-wheel companion remainder per destination
  endpoint/axis. Route-local processors, if introduced, own separate state.
  Document that mixed sources can complete a shared fractional output unit.
- Scope configuration-change resets to the affected routes/endpoints. Disconnect
  discards source-local partial reports; destination state follows its declared
  lifetime. Reconnection must not reuse another attachment's partial frame.
- Preserve keyboard/button ordering when repacking reports. Flush each affected
  output with its own `SYN_REPORT`; no cross-device atomicity is promised.
- Exclude owned virtual devices from capture by robust ownership identity, so
  generated motion cannot recursively re-enter its source routes.
- Validate destination codes against backend/profile capabilities when compiling
  configuration. Report physical advertised capabilities separately from virtual
  capabilities and separately again from observed consumer behavior.

### Why several devices help

Three virtual devices can each have `REL_WHEEL`, giving three separately
addressable wheel streams to a device-aware consumer. A new identity is genuinely
created at the output, even if two streams derive from one physical wheel.
It does not add an independent human control: their values remain related by
the configured transform. Desktop scrolling may merge all three. The payoff is
largest in a custom application, direct-input tool, simulator, or explicit
multi-device binding system.

Costs are lifecycle, node discovery, per-endpoint state, consumer classification,
and reporting errors across several sinks. Keep device topology stable while
changing routes; capability changes should be an explicit device-recreation
boundary rather than surprise hotplug churn on every mode change.

## Design B: application-shaped absolute or protocol outputs

### Virtual joystick or gamepad

For a consumer that actually accepts a joystick axis, converting a wheel to
`ABS_RZ` may be more useful than emitting `REL_RZ`. The conversion requires a
stateful integrator, not merely another matrix row:

```text
camera.abs_rz = clamp(camera.abs_rz + wheel_delta * gain, minimum, maximum)
reset_action -> camera.abs_rz = center
```

Specify range, initial value, deadzone/fuzz where applicable, reset behavior,
and saturation. A wraparound dial needs a different policy from bounded rudder
position. A spring-return stick needs time-based return and shutdown neutral
handling. A consumer interpreting a held stick as velocity is not equivalent to
one interpreting its value as position. Device recognition also depends on the
chosen buttons/axes and the receiving library; emitting ABS values alone does
not establish a broadly compatible gamepad. Start with one named target
application and validate its actual input path.

### UHID and arbitrary logical axes

[UHID](https://docs.kernel.org/hid/uhid.html) can instantiate a HID device with a
descriptor and feed input reports. This is useful when a consumer speaks HID
and needs particular report shapes, usages, collections, or a device protocol.
It entails report/control-request handling beyond uinput. If the consumer goes
through generic HID-to-evdev translation, finite evdev codes still apply; UHID
is not a generic unlimited-wheel desktop extension.

For genuinely arbitrary channels such as `camera.zoom`, `timeline.scrub`, and
`brush.size`, an application-facing socket or existing application protocol may
be the cleaner endpoint. It can preserve names, units, and many controls without
assigning misleading Linux codes. This is a separate adapter requiring a real
consumer, not a reason to restrict the Linux relative path today.

## Design C: recover hidden inputs only where evidence justifies it

[hidraw](https://docs.kernel.org/hid/hidraw.html) returns unmodified device reports
and exposes their descriptors. Numbered reports retain a report-number prefix.
Two physical wheels might occupy different fields or reports even if a driver
maps them to the same evdev code. A HID-aware source adapter could then expose
separate logical inputs using a key such as `(device, report ID, collection,
usage, field occurrence)`; report ID alone is not necessarily a control ID.

This is a possibility to investigate, not a claim about this Keychron. First
record all evdev interfaces while operating each wheel independently. If the
streams already distinguish them, retain device identity and stop there. If
they do not, inspect the descriptor and raw reports, correlating each control
with field changes. Only build a decoder after demonstrating an actual loss
at the HID-to-evdev boundary.

An adapter must handle report layouts, signed widths, units, collections,
transport-specific variants, and duplicate evdev delivery. Select exactly one
authoritative input path for each decoded control. Firmware that transmits only
one already-merged wheel value offers nothing for hidraw to separate. Firmware
changes or a different device would then be required for true independence.

## Optional direction-sensitive routing composes naturally

Direction splitting is an additional transform stage, independent of how many
devices or axes exist. Retain the ordinary signed map as the default. In an
advanced mode, split a normalized source value into nonnegative branches:

```text
positive(d) = max(d, 0)
negative(d) = max(-d, 0)

keychron.v.positive -> desktop.y * -0.25
keychron.v.negative -> orbit.dial * (1 / 120)
pointer.y.negative  -> desktop.v * 4
pointer.y.positive  -> desktop.y * 0.5
```

These examples describe branch semantics, not proposed parser spelling. Output
sign remains explicit in the gain. Native pointer Y-negative means up;
V-positive means wheel-up before compositor natural-scrolling policy.

Normalize high-resolution/legacy companions before splitting, so one physical
notch cannot trigger both representations. Split before mixing distinct inputs;
otherwise opposite-source contributions could cancel before their different
destinations are selected. Within one native axis/report, decide explicitly
whether splitting sees the aggregate delta or each ordered event. Aggregate
report semantics fit the current transform; preserving within-report reversals
would require richer event-sequence semantics and cannot recover reversals
already combined by hardware.

Directional routes can still converge on the same destination. Decide whether
their fractional credits share the normal destination remainder or are retained
per branch; the latter prevents cancellation but may emit movement after
alternation that the former would cancel. Prefer destination accumulation for
linear analog outputs, and explicit branch-local thresholds when defining
independent actions. Coordinate this choice with the parent's implementation;
this document does not settle its syntax or state policy.

## Recommended next commissions and evidence gates

1. **Prove current output freedom.** Use a captured or synthetic input report to
   generate Z/RZ/dial on the existing virtual device, inspect that output with an
   evdev reader, and separately demonstrate V/H-to-X/Y through the desktop.
   This answers the immediate question without conflating kernel delivery and
   application interpretation.
2. **Capture the Keychron topology.** Turn each wheel separately, then together,
   across every relevant event node. Record source node identity, report framing,
   legacy/high-resolution pairs, and actual relationships between interfaces.
   This determines whether qualified sources solve a present need.
3. **Implement qualified reports and routing.** Carry source identity through the
   Linux reader, expose stable aliases, and validate two sources using the same
   native code. Preserve the simple existing map. This is the highest-leverage
   general extension even before creating additional outputs.
4. **Add named uinput endpoints with one demonstrated consumer.** Show two output
   `REL_WHEEL` streams remaining independently addressable to that consumer;
   test replug, ambiguous selectors, output exclusion, fractional accumulation,
   and report boundaries. Verify libinput behavior separately where relevant.
5. **Choose a consumer-driven branch.** ABS integration for a real controller
   consumer, hidraw recovery for demonstrated hidden inputs, or UHID/protocol
   output for a demonstrated interface need. These are alternatives with
   different payoffs, not prerequisites for ordinary mouse flexibility.

Recommended basis: **native endpoints plus qualified channels, with explicit
consumer evidence**. It permits Linux's richer capabilities immediately, leaves
room for different input/output cardinalities and direction-dependent routing,
and makes the distinction between creating output controls and recovering
physical input information understandable and testable.
