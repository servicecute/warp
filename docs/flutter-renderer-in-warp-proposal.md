# Flutter Renderer in Warp

## Summary

Warp can host and interact with Flutter applications in an editor pane using a
Raccoon-style hosted runtime. The recommended design is to run the Flutter
engine in a separate native runner process and connect it to Warp through a
small Rust client and frame/input protocol.

This proposal intentionally does not embed Flutter widgets directly into
WarpUI. Warp and Flutter each own a rendering stack, event loop, platform
integration layer, and GPU lifecycle. Keeping the Flutter engine in a child
process gives Warp a stable integration boundary and prevents a Flutter crash
or engine shutdown from taking down the terminal.

Raccoon provides a useful reference implementation:

- [`raccoon_runner`](../../raccoon_simulator/raccoon/native/raccoon_runner/README.md)
  hosts the Flutter engine through the native embedder API.
- [`RaccoonView`](../../raccoon_simulator/raccoon/packages/raccoon_view/lib/src/raccoon_view.dart)
  forwards size, pointer, keyboard, cursor, and text-input events while
  displaying runner frames.

The local Raccoon Simulator tracks [`SoFluffyOS/raccoon_simulator`](https://github.com/SoFluffyOS/raccoon_simulator),
and its nested runtime tracks [`SoFluffyOS/raccoon`](https://github.com/SoFluffyOS/raccoon).
The upstream runtime is already evolving Windows plugin support, GPU views,
and camera/video capabilities. Warp should consume those improvements through
versioned runner releases or upstream contributions rather than maintaining a
second Flutter runner fork.

## User experience

The user opens a Flutter project or selects a Flutter run target in Warp. Warp
starts a hosted Flutter runtime and shows the application in a pane alongside
terminals, code, and agent views.

The hosted application should behave like a normal Flutter desktop preview:

- pointer and scroll events reach Flutter;
- keyboard focus and shortcuts reach Flutter when the preview is focused;
- text fields support editing and composition where the host supports it;
- the preview resizes with its Warp pane and reports the correct pixel ratio;
- Flutter controls can change the cursor;
- hot reload and hot restart remain available through the Flutter tool or a
  Warp toolbar action; and
- runner disconnects show a recoverable error state rather than closing Warp.

## Agentic controls

The Flutter panel should be operable by both a human and a Warp agent. The
agent API should prioritize semantic actions over raw coordinates so an agent
can interact with widgets reliably across window sizes, pixel ratios, and
minor layout changes.

### Essential controls

The first agent-facing surface should support:

- starting, stopping, restarting, and reconnecting the Flutter runner;
- hot reload and hot restart;
- selecting the project, entrypoint, device profile, size, and pixel ratio;
- reading runner logs and Flutter errors;
- capturing a screenshot;
- tapping, clicking, long-pressing, swiping, scrolling, and typing;
- waiting for a frame, text, or widget state; and
- returning structured success or failure results.

The preferred interaction contract should look like:

```text
tap("Save")
enter_text("Email", "user@example.com")
scroll("Settings list", direction: "down")
assert_visible("Account created")
```

Warp should expose Flutter's semantics/accessibility tree to support lookup by
visible text, accessibility label, key, route, or widget type. Coordinate
actions remain useful as a fallback for custom-painted surfaces, but they
should not be the primary agent interface.

### Development and debugging controls

The next level should provide:

- Flutter widget and semantics-tree inspection;
- lookup by text, key, type, or accessibility label;
- current route and focused-widget inspection;
- streamed console output and uncaught exceptions;
- Dart VM service status;
- reload and restart status;
- frame timings and performance metrics; and
- screenshot comparison between runs.

### Workflow controls

Agents should also be able to:

- run a Flutter test or integration test;
- launch a named route or app state;
- clear app data;
- reset or seed preview state;
- save screenshots, logs, and interaction traces as artifacts; and
- reproduce an interaction sequence from a recorded trace.

### Platform capability controls

Platform capabilities should be explicit and inspectable rather than hidden
behind arbitrary plugin calls. Later controls may include:

- synthetic, file-backed, or physical camera sources;
- permission grant and denial simulation;
- location, notification, and biometric simulation;
- deep-link and URL-launch interception; and
- native plugin capability inspection.

The initial agent action set should remain small:

```text
run_flutter
reload_flutter
inspect_flutter
tap
type
scroll
press_key
wait_for
assert_visible
screenshot
read_logs
```

These actions should support a predictable agent loop:

```text
start -> inspect -> interact -> observe -> assert -> recover
```

The semantics bridge is the highest-value reliability investment. It lets
agents operate Flutter applications without depending on fragile screenshots
or absolute coordinates.

## Agent-control-plane integration

Warp should not create a second, Warp-specific agent command protocol for the
Flutter panel. The existing
[`lib-autonomous-testing` Raccoon proposal](../../rust-workspace/lib-autonomous-testing/docs/raccoon-agentic-runtime-proposal.md)
already defines the appropriate control-plane boundary:

```text
Warp panel and UI
  -> Flutter runtime host and visual surface

lib-autonomous-testing
  -> agent commands, capabilities, policies, sessions, and artifacts

Raccoon runner
  -> Flutter engine, platform channels, frames, and input transport
```

The existing agent-drivable surface should be the model for the Warp
integration. Its command vocabulary, capability profiles, transient semantic
references, screenshot artifact references, and structured errors are more
useful than exposing runner sockets directly to an agent.

### Reuse boundary

The integration should reuse the autonomous-testing control plane for:

- launch and attach lifecycle commands;
- tap, input, press, scroll, wait, and assertion commands;
- capability discovery and honest unsupported-command errors;
- normalized accessibility snapshots and transient element references;
- screenshot, video, log, and console artifact storage;
- session leases, busy-session handling, cancellation, and cleanup;
- policy evaluation, redaction, and auditability; and
- replayable interaction traces and generated test actions.

Warp should own the parts specific to being a desktop host:

- opening and laying out the Flutter preview pane;
- connecting panel focus to the active agent session;
- rendering runner frames through WarpUI;
- exposing human-facing reload, restart, and runtime controls; and
- mapping Warp window/pane state to runner resize and lifecycle messages.

The first integration should use the existing runner socket and VM-service
interfaces behind a `RaccoonBackend`-style adapter. A new Raccoon protocol
extension should be added only for controls that cannot be expressed through
those interfaces, such as Flutter semantics snapshots, explicit touch-drag
gestures, screenshot acknowledgements, or test-only app commands.

### Flutter semantics

The autonomous-testing project deliberately keeps its default sensing contract
framework-neutral. Flutter semantics should therefore be an opt-in backend
capability:

1. Request a Flutter semantics snapshot when the hosted app exposes one.
2. Normalize it into the shared accessibility snapshot schema.
3. Assign transient refs such as `@e3` for the latest snapshot.
4. Allow `tap`, `fill`, `scroll`, and `assert` to target those refs.
5. Return a typed stale-reference error when the UI changes.
6. Fall back to coordinates and screenshots when semantics are unavailable.

This preserves compatibility with the existing Android, iOS, web, and desktop
backends while giving Flutter previews richer interaction when the capability
is present.

### Initial agent flow

The intended end-to-end flow is:

```text
agent
  -> auto-test MCP/CLI command
  -> capability and policy evaluation
  -> Raccoon backend adapter
  -> runner socket / VM service
  -> Flutter preview in Warp
  -> structured result and artifact reference
```

The first useful flow should support launch, snapshot, semantic tap/input,
scroll, wait, screenshot, log retrieval, and close. This gives agents a
deterministic loop without putting an LLM or an agent protocol inside the
Flutter runner itself.

## Proposed architecture

```text
Flutter project
      |
      | build assets, VM service, plugin metadata
      v
Flutter runner process
      |
      | Raccoon-style protocol over local socket
      | frames, resize, pointer, keyboard, text input, channels
      v
Rust FlutterPreview client in Warp
      |
      | dynamic image/texture and input actions
      v
WarpUI FlutterPreview element
```

### Runner process

The runner should be based on the native Flutter embedder API and should be
compatible with the Raccoon runner's responsibilities. The runner should be a
versioned external runtime dependency, not a library linked directly into the
Warp process:

- load the engine matching the Flutter project's engine revision;
- launch the Flutter asset bundle;
- provide a pumped platform task runner;
- accept resize, pointer, keyboard, and text-input events;
- emit frames and lifecycle/log events;
- expose a VM service endpoint for debugging and hot reload;
- load supported native plugins; and
- shut down cleanly when the Warp pane or project is closed.

The runner must remain a separate process for the first implementation. This
allows Warp to kill and restart a stuck engine and avoids sharing Flutter's
global runtime state with Warp's Rust process.

Warp should keep its runner integration narrow enough that improvements to the
Raccoon runner can be consumed without rewriting the Warp client. Protocol
changes should be negotiated by version, and platform-specific runner work
should be upstreamed to Raccoon where possible.

### Rust client

Warp should own a Rust client that manages the runner lifecycle and translates
protocol messages into Warp events. It should provide:

- runner discovery and version negotiation;
- startup, readiness, disconnect, and crash states;
- frame receipt and buffer-release acknowledgements;
- pane-size and device-pixel-ratio updates;
- pointer, scroll, keyboard, and text-input forwarding;
- log and VM-service metadata; and
- cancellation and cleanup when the editor tab closes.

The client should be modeled as an entity-backed app service rather than as a
renderer-global singleton. Multiple Flutter previews may eventually run at
the same time, and each preview needs independent process, socket, focus, and
buffer state.

### WarpUI view

The WarpUI view should be a normal element/view pair with a native preview
model behind it. It should own the input hit region and focus state, then
forward events only while the preview is focused.

The view should render one of two frame paths:

1. A dynamic Warp texture updated from shared-memory pixel frames.
2. A platform-specific imported GPU texture when zero-copy support is enabled.

The first path is the required baseline. It keeps the protocol and UI work
independent from platform-specific GPU interop.

## Frame transport

### Phase 1: shared-memory pixel frames

The runner renders into a reusable BGRA/RGBA buffer and signals Warp when a new
frame is available. Warp copies or uploads the newest frame into a dynamic
`wgpu::Texture`, dropping stale frames when the UI is behind.

This path is slower than direct GPU sharing but is straightforward to debug,
works across platforms, and is sufficient to validate interaction and product
fit.

The frame queue must be bounded. The preview should prefer the newest frame and
never accumulate unbounded frame latency while the user is interacting.

### Phase 2: native GPU buffer sharing

After the baseline is stable, add platform-specific paths:

- macOS: IOSurface/Metal-backed buffers;
- Linux: dmabuf or EGL/Vulkan-backed buffers; and
- Windows: shared D3D textures or handles.

Warp currently renders through `wgpu` and owns its own graphics device. The
GPU path therefore requires backend-specific import code and explicit
synchronization/fence handling. It should not block the first usable version.

## Interaction and input

Frame transport alone is not enough for a useful preview. The protocol must
carry input in the coordinate system and pixel ratio used by the runner.

Required events:

- pointer move, down, up, cancel, and scroll;
- keyboard physical/logical key data and modifier state;
- focus and blur;
- text insertion, deletion, selection, and composing-region updates;
- resize and device-pixel-ratio changes; and
- cursor updates from the Flutter application.

Warp should preserve its own application shortcuts outside the preview. When
the preview has focus, ordinary text and pointer input should be sent to
Flutter; explicit Warp chrome and pane actions should remain available.

IME behavior needs a dedicated design pass. Basic key forwarding is not a
substitute for composition support in languages that depend on an input method.

## Plugin and platform behavior

Flutter applications may depend on plugins that expect a native desktop
window, platform channel, filesystem location, permissions, or GPU texture
registry. The runner should classify plugin behavior instead of pretending
unsupported calls succeeded.

The plugin system should use four compatibility tiers:

1. **Dart-only plugins** work unchanged.
2. **Host-served plugins** are implemented by the runner or Warp host through
   platform-channel handlers.
3. **Native desktop plugins** are built for the host OS and loaded by the
   runner's registrar and messenger.
4. **Mobile-only plugins** are reported as unavailable, simulated where safe,
   or connected through an explicit external-device workflow.

Raccoon's plugin manifest and compatibility report should be the model for the
Warp implementation. Before launching, Warp should inspect the project's
`.flutter-plugins-dependencies` and show the user which plugins are supported,
host-served, unavailable, or not applicable to the desktop host.

Initial host-served support should prioritize:

- Dart-only packages;
- common preferences and path APIs;
- network and serialization packages;
- Flutter plugins that use platform channels without requiring a visible
  native window; and
- plugins whose texture interfaces can be adapted to the selected frame path.

Examples of host services include:

- `shared_preferences`: a per-preview storage directory;
- `path_provider`: a project or preview sandbox;
- `url_launcher`: Warp or operating-system browser handling;
- `file_picker`: Warp's file picker returning a selected path;
- `window_manager` and similar APIs: pane-aware behavior rather than a
  second top-level window; and
- permission APIs: only permissions that have a meaningful host equivalent.

Plugins that require a native window, system modal, camera, microphone, or
platform-specific permission flow should report a clear unsupported state or
open an explicit external runner window.

The runner, not Warp, should load third-party native plugin binaries. The
runner can register their platform channels and texture callbacks, then
composite plugin output into the final Flutter frame. Warp only needs to
transport the completed frame and input events. This keeps native plugin
registrars, Flutter texture registries, and platform ABI details outside the
Rust application process.

Warp must not fake successful responses for unsupported channels. An
unsupported call should produce the normal Flutter missing-plugin behavior,
while the preview diagnostics surface explains the compatibility status.

## Camera plugin support

Camera support is an optional plugin capability, not a prerequisite for the
core Flutter preview. The primary goal remains opening a Flutter application
inside a Warp panel, forwarding interaction, and displaying the completed
Flutter frame.

The camera flow would look like this:

```text
Flutter app calls camera Dart API
          |
          | plugins.flutter.io/camera channel
          v
Flutter runner handles the host implementation
          |
          | AVFoundation / Media Foundation / V4L2 or test source
          v
Flutter engine renders CameraPreview or returns captured image
          |
          v
Warp displays the final Flutter frame
```

Warp should not implement the camera Dart API or interpret camera frames in
the panel. The runner should either load a desktop camera plugin or provide a
runner-side implementation of the camera platform channels. Warp only needs
to display the resulting frame and forward input.

### Support levels

Camera support should be delivered incrementally:

1. **Synthetic camera** — expose deterministic front/back test cameras and a
   generated pattern for plugin and UI testing.
2. **File-backed camera** — use PNG/JPEG frames or a looping video as a camera
   source for repeatable development and CI scenarios.
3. **Camera API support** — implement camera discovery, initialization,
   image-stream formats, and `takePicture` through the runner.
4. **Flutter texture preview** — populate the camera plugin's texture registry
   so the standard `CameraPreview` widget renders inside the hosted app.
5. **Physical camera capture** — connect platform-native capture APIs and
   permission flows: AVFoundation on macOS, Media Foundation on Windows, and
   V4L2/GStreamer on Linux.
6. **Video recording** — add recording only after preview, still capture, and
   lifecycle cleanup are reliable.

The synthetic and file-backed levels are especially valuable because they make
camera-dependent Flutter applications testable without granting Warp access
to a physical camera. They also provide deterministic fixtures for runner and
panel integration tests.

### Permission and privacy behavior

Physical camera access must be explicit and visible. The runner should:

- request permission through the host operating system;
- report denied, restricted, unavailable, and granted states to the Flutter
  application;
- stop capture and release the device when the preview closes or loses its
  session;
- avoid silently using a physical camera when a synthetic or file-backed
  source is configured; and
- show the active source and permission state in preview diagnostics.

Camera frames and captured files must remain inside the preview's declared
data boundary unless the user explicitly saves or exports them.

### Initial acceptance criteria

Camera support should not block the first Flutter-panel release. When camera
support is added, the first useful milestone should require:

- a Flutter app using `availableCameras()` to start without a channel error;
- a deterministic camera source to render a `CameraPreview` texture;
- `takePicture` to return a valid image;
- image streams to reach the Flutter app at a bounded frame rate;
- unsupported recording or physical-device features to fail clearly; and
- runner shutdown to release camera resources and leave Warp usable.

## Lifecycle and security

The runner executes project-provided Dart code and native plugins. It must be
treated as an external project process:

- start it with an explicit project and asset directory;
- use a per-preview temporary/data directory;
- avoid inheriting unrelated secrets where possible;
- restrict protocol connections to a private local socket;
- validate message sizes and frame dimensions;
- terminate the process tree on close or cancellation; and
- surface crashes and stderr without allowing malformed runner data to crash
  Warp.

The VM service endpoint should not be exposed beyond the local preview session
unless the user explicitly enables remote debugging.

## Implementation phases

### Phase 0 — feasibility spike

- Build or reuse a Raccoon runner for macOS.
- Start it from a small Rust command or test harness.
- Receive one frame and send one pointer event.
- Confirm a Flutter counter/button app responds to input.
- Measure startup time, resident memory, frame latency, and CPU use.

### Phase 1 — interactive macOS preview

- Add the Rust runner client and lifecycle state machine.
- Add a WarpUI preview element backed by a dynamic pixel texture.
- Implement resize, pointer, keyboard, focus, and basic text input.
- Add disconnect, crash, and restart states.
- Add an integration test for a Flutter button and text field.

### Phase 2 — developer workflow

- Discover Flutter projects and run targets from `pubspec.yaml`.
- Wire hot reload/restart and VM-service status into the preview toolbar.
- Stream runner logs into the preview diagnostics surface.
- Add project-local runtime and device-size configuration.
- Support multiple previews with independent runners.

### Phase 3 — performance and platform coverage

- Add macOS zero-copy IOSurface/Metal sharing.
- Add Linux shared-buffer and GPU paths.
- Add Windows shared-texture support.
- Add synthetic/file-backed camera support, then physical camera capture by
  platform.
- Expand plugin compatibility and IME coverage.
- Compare frame latency and CPU/memory usage against Raccoon.

Runner/plugin work that is broadly useful should be contributed to Raccoon and
consumed by Warp through a runner release. Warp-specific changes should remain
in the Rust client, host capability bridge, and WarpUI preview surface.

## Acceptance criteria for the first usable version

- A Flutter project can start from Warp without opening a separate preview
  window.
- A Flutter button, scrollable list, and text field work inside the Warp pane.
- Keyboard focus transitions correctly between Warp and Flutter.
- Resizing the pane updates Flutter layout and pixel ratio.
- Hot reload updates the visible application.
- Runner failure leaves Warp usable and offers restart.
- Frame latency remains bounded under interaction.
- The implementation works without GPU texture import; shared pixel frames are
  a supported fallback.
- The runner process and its project data are cleaned up when the preview
  closes.

## Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| GPU interop is expensive or unavailable | Ship shared-memory pixel frames first. |
| Flutter engine crashes the host | Keep the engine in a child process. |
| Input feels delayed | Bound frame queues and prioritize newest frames. |
| IME behavior is incomplete | Treat composition as a first-class protocol feature. |
| Plugins assume a real native window | Define compatibility classes and report unsupported plugins clearly. |
| Multiple engine versions increase distribution cost | Match the runner to the project's engine revision and cache runtimes. |
| Runner protocol becomes a maintenance fork | Pin released Raccoon runtimes, negotiate protocol versions, and upstream generic changes. |
| A mobile-only plugin is mistaken for a desktop capability | Report plugin support before launch and never emulate success for unsupported channels. |

## Recommendation

Proceed with a macOS-only Phase 0 spike using a released Raccoon runner and
shared pixel frames. Do not begin with in-process Flutter embedding or zero-
copy GPU interop. The spike should answer the two product questions first:
whether a Flutter preview belongs naturally beside Warp's terminal/editor
workflows, and whether pointer, keyboard, text input, and hot reload feel
responsive enough in a Warp pane.
