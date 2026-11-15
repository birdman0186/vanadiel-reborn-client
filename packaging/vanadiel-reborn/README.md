# Vana'diel Reborn Windows baseline

This package uses Kuluu with the existing LandSandBoat server.
Keep the working server, database backups, original FFXI client, and launcher.
The package contains no server or game assets.

## Build

Use Windows 11 x64, Git, Rustup, and Visual Studio C++ Build Tools with a Windows SDK.
Open an x64 Developer PowerShell.
Run `./packaging/vanadiel-reborn/build-windows.ps1` from the checkout.
The script reads the Rust pin and initializes only build-time vendor submodules.
The build uses LLVM, MSVC, native-window, and locked dependencies.
DLSS and gameplay enhancements are excluded.
The output is `dist/vanadiel-reborn-windows-x64.zip`.
The VDR Windows baseline workflow builds this package on `vdr/**` pushes.

The upstream baseline is `1b611a74e0211892bf141ad849ebca175cb7ddee`.
The lockfile uses Bevy 0.20.0 and wgpu 30.0.1.
Rust is pinned to nightly-2026-06-01.
The vendored LSB revision is `73e015192696e3208d0e3af46215db8fc79fb49c`.

## Inspect the existing installation

Python 3 and Git are required for inspection.
From the extracted package, run:

```powershell
python ./inspect_lsb.py --server-root 'E:\ActualServerDirectory' --dat-root 'E:\ActualFFXIDirectory' > vdr-inspection.json
```

Replace the example directories with the actual paths.
In a checkout, use `packaging/vanadiel-reborn/inspect_lsb.py` instead.
The tool reads selected port and version assignments and hashes the FFXI index files.
It does not execute Lua, access MariaDB, authenticate, or change either installation.
It excludes passwords from its JSON output.
Assignments are candidates; conditions and cross-file overrides can change active settings.
Confirm active values against the running server.

## Connect

Extract the package into a separate directory.
Set the game-file path for the current PowerShell session:

```powershell
$env:FFXI_DAT_PATH = 'E:\ActualFFXIDirectory'
./kuluu.exe
```

Add a server profile named Vana'diel Reborn.
Enter the confirmed server address and ports.
Choose Current LSB for JSON authentication or Legacy LSB for binary authentication.
Confirm that protocol from the existing server source and launcher.
Enter credentials in the launcher.
Use a separate test account and character for initial checks.
Keep the original client disconnected from that account during testing.

Vendored defaults: auth TCP 54231, lobby-data TCP 54230, lobby-view TCP 54001, and map UDP 54230.
The lobby supplies the actual map endpoint.
Port 54003 is the internal ZMQ port, not the client map UDP port.
The vendored client stamp is 30261006_0 and the JSON loader version is 2.2.0.
These defaults do not establish the owner's active server settings.
Keep version checks enabled.
Do not change the server stamp to conceal an asset mismatch.

## Verification boundary

Compilation, executable startup, GPU rendering, and live server entry are separate checks.
The build manifest records source, toolchain, executable hash, and verification limits.
Live acceptance requires authentication, character selection, world entry, movement, zoning, combat, logout, and reconnect.
Compare the original gameplay and animation timing.
Record server revision, asset version, GPU, resolution, and frame times.
The target is stable 60 FPS with an approximately 16.67 millisecond frame budget.
Measure busy areas and combat.

Cinematic Engine implementation starts after live integration passes.
Preserve original gameplay and animations.
Keep graphics enhancements optional and scalable through presets.
Ray tracing and animated vegetation are outside scope.
Keep Rust, Bevy, and wgpu as the foundation.

## November server compatibility

The recorded server revision is `7a91059f856933fcec58f2287e22c6ef6148935c`.
That revision requires JSON loader version `3.0.x` and TLS on both lobby connections.
Select Current LSB and enter loader version `3.0.0` in the server profile.
The client reuses the authentication certificate pin for both encrypted connections.
A TLS failure never causes a plaintext retry.
The selected loader version and authentication flavor persist through reconnect.
Older loader profiles retain their original lobby transport.

That server revision defaults to client stamp `30261102_0`.
Confirm the active setting with the inspection tool.
Use matching game files; keep server version checks enabled.
The handoff layout and Huffman resource bytes match the older build input.
The build input remains pinned; the running server remains unchanged.
Encrypted loopback tests cover character selection, handoff, and certificate mismatch rejection.
Live server compatibility and GPU presentation still require a local test.

## Classic HUD pass 1

The default HUD has separate HP, MP, and TP gauges at the bottom right.
Compact party entries appear above those gauges when a party exists.
Each party entry has a name, HP gauge, and MP/TP values.
The classic layout omits custom activity text, job icons, distances, and red bands.
Target selection remains available through the party entries.
The custom HUD remains available through UI Settings in debug builds.
Keep the earlier Windows package for rollback.

The layout follows Square Enix's published classic-interface references.
See [the source record](../../.agents/skills/retail-observe/references/classic-status-layout-2026-11-15.md).
The current font, colors, dimensions, and flat borders are provisional.
Exact retail window textures, gauge textures, and font matching remain pending.
The TP gauge fill rule also needs retail confirmation.
This pass does not restore every FFXI menu or establish complete interface parity.

The production-widget fixture passed six rendered cases on Linux with Mesa Lavapipe Vulkan.
All captures were opened and inspected.
Solo and six-member parties passed at 100%, 150%, and 200% UI scale.
The cases include a long name, low HP, zero MP, and an out-of-zone member.
Computed text bounds did not intersect within each entry.
Four automated tests cover solo status, missing self data, unavailable values, zoning, and layout rollback.
Thirteen existing party-frame tests also passed.
Those tests check grouping and custom layout behavior.

Run the fixture with:

```sh
cargo run --locked --release -p kuluu-render --example party-frame-capture -- /absolute/capture/directory
```

The six output files use `classic-solo-100.png` through `classic-party-200.png` names.
The fixture uses synthetic data and production widgets.
Its captures do not establish Windows GPU behavior, live gameplay, or performance.
The Windows workflow builds the playable client and runs the four new state tests.
A live Windows screenshot remains required before accepting this candidate.
Keep the working server and original FFXI client unchanged.

The earlier Windows software-renderer capture returned uniform frames.
The fixture rejects uniform images.
Explicit `[hud-verify-only]` runs still fail if capture fails.
Normal Windows package builds do not run that off-screen capture.

## Original interface restoration

The status layout above is an approximation.
It does not satisfy the requested original FFXI windows, fonts, menus, and controls.
Original resource loading and menu behavior need separate verification.
Do not describe the current client as a complete original-interface restoration.

The target includes original window artwork, fonts, cursors, status displays, chat, dialogs, and game menus.
Compare main commands, Status, Equipment, Items, Magic, Abilities, party controls, trade, shops, and auction windows against the original client.
Preserve keyboard and controller confirmation, cancellation, targeting, and navigation.
The requested mouse preference disables automatic world-click menus.
Clicks still select targets and clear unlocked targets on empty ground.
Keyboard and controller handlers remain unchanged.

The mouse correction has 24 passing production-system tests.
The tests cover empty ground, selected entities, dragging, locks, and an open keyboard menu.
These tests use synthetic events.
Live Windows mouse input and menu navigation remain unverified.
The correction is not included in the earlier Windows ZIP.

The available Drive texture pack omits the original window DATs.
Collect the required resources from the installed game:

```powershell
.\Collect-VdrUiResources.ps1
```

The default game path is `E:\Final Fantasy XI\SquareEnix\FINAL FANTASY XI`.
Use `-GameRoot` to select another installation.
The script writes a uniquely named ZIP to the Desktop.
Use `-OutputDirectory` to select another output directory.
Upload that ZIP for resource inspection.

The collector copies the index tables, early UI DATs, and static menu resource DATs.
Its manifest records copied files, sizes, hashes, and missing optional files.
It checks source hashes before and after each copy.
It rejects output directories inside the game installation.
The repository and client packages contain no collected game resources.

Run the collector checks with:

```powershell
.\Test-UiResourceCollector.ps1
```

The collector checks passed with synthetic files on PowerShell 7 for Linux.
They check archive contents, hashes, source preservation, output boundaries, and missing required files.
Windows PowerShell execution remains unverified.
