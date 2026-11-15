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

### Proposed first graphics trial

Use Bastok Markets as the first comparison zone. Record the same walk from the fountain to the auction counters in daylight and at night. Keep the camera position, output resolution, frame cap, and game time the same between each pair of captures. Record zoning separately from steady movement. Use a repeatable combat sequence outside the city to check effect visibility and animation timing.

| Pass | Proposed change | Check in game |
| --- | --- | --- |
| Stone and metal | Test albedo, normal, roughness, and metalness on one stone stair and one metal fitting. Use the existing zone material shader and texture mipmaps. | Stone keeps its relief in oblique light. Metal responds to the lamp. Insignias remain readable. |
| Lighting and effects | Tune the existing lighting, directional shadows, lamp range, and bloom around this scene. | Check railings, shadows, and sign brightness at noon and at night. Keep combat cues and menu text legible. |
| Geometry and texture detail | Try one higher-detail railing and one static tree with distance-based detail. Enable higher-resolution textures selectively. | Check silhouettes and texture shimmer while moving. Walk the original routes and verify targeting and collision. |

Use the existing Minimum through Maximum presets. Start with original assets on Minimum and Low. Measure each additional effect before assigning it to higher presets. Expose accepted settings in both graphics menus. Keep the original asset as the fallback when an enhancement is missing.

For each trial, record the GPU, output and internal resolutions, warm frame-time median, 95th and 99th percentiles, and the proportion of frames above 16.67 ms. Record texture memory and zone-loading delays separately. Start with the owner's RTX 4090 and add a lower-spec machine before claiming a supported hardware range. The menu capture test does not measure these costs.

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

The owner supplied the missing original window and menu resources.
All 46 collected files match the manifest sizes and SHA-256 hashes.
The client reads this artwork from the selected installation at runtime.
The repository and client packages contain no collected game resources.

The shared frame renderer reads all eight original styles.
It uses their background, corner, and border textures with authored geometry and vertex colors.
Command labels, supported menu titles, and the selection cursor use original sprite components.
Menu text remains available as a fallback when artwork cannot load.
The loader resolves textures across static menu DATs and the selected window style.
It refreshes cached artwork when the installation or style changes.

This pass does not restore the complete original interface.
Dynamic text still uses the existing font.
Command order, available actions, window positions, and cursor animation still need original-client comparison.
The renderer currently uses background element zero.
Active/inactive background switching remains open.
Several original menu screens and commands remain unavailable.
Keep these acceptance cases open.

Run the production artwork fixture with:

```bash
FFXI_DAT_PATH=/path/to/FFXI cargo run --locked --release -p kuluu-render --example original-ui-capture -- /absolute/capture/directory
```

The fixture renders each style at 100%, 150%, and 200% UI scale.
It uses production menu and status widgets with synthetic session data.
It rejects collapsed menu rows, overlapping rows, and overlapping status text.
It also rejects overlap between the Command menu and status windows.
These captures cannot establish live Windows input, gameplay, or performance.

The local library suite passed 2,366 tests, with four ignored tests.
Focused resource and menu checks passed 53 tests.
The 24 artwork captures passed and were inspected.
Additional production captures checked Items, dialog, and delivery window frames.
The collected subset lacks their item icon DATs.
Those captures show empty icon cells and cannot verify quantities over bright or dark icons.
Scoped Clippy, blocking-policy fixtures, formatting, and source checks passed.
Live Windows input and original-client comparison remain unverified.

For another resource collection, run:

```powershell
.\Collect-VdrUiResources.ps1
```

The default game path is `E:\Final Fantasy XI\SquareEnix\FINAL FANTASY XI`.
Use `-GameRoot` to select another installation.
The script writes a uniquely named ZIP to the Desktop.
Use `-OutputDirectory` to select another output directory.
Use that ZIP for private resource inspection.

The collector copies the index tables, early UI DATs, and static menu resource DATs.
Its manifest records copied files, sizes, hashes, and missing optional files.
It checks source hashes before and after each copy.
It rejects output directories inside the game installation.
The repository and client packages contain no collected game resources.

Run the collector checks with:

```powershell
.\Test-UiResourceCollector.ps1
```

If PowerShell blocks the unsigned collector, permit only its new process:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File ".\Collect-VdrUiResources.ps1"
```

This command does not change the saved execution policy.

The collector checks passed with synthetic files on PowerShell 7 for Linux.
They check archive contents, hashes, source preservation, output boundaries, and missing required files.
Windows PowerShell execution remains unverified.

## Original Commands layout

Open the Commands menu with the minus key on the numeric keypad.
Use Left or Right to change pages. Use Up or Down to select a row.
Press Enter to open the selected command. The controller D-pad follows the same row and page order.
For client graphics settings, open page two, select Config, then select Graphics.

Both English pages read their pane dimensions, row positions, and decoration from the installed FFXI menu files.
The supplied game files give a 112 × 202 pane with twelve 88 × 16 buttons per page.
Gauges, chat, and other screens have separate renderers.
Some menu entries still report that their action is unavailable.

The client derives the selected button tint from the supplied menu reference.
The exact retail tint animation and the display scale on the owner's Windows installation still require comparison.
The Commands and Network panes use a common right inset when the status display fits below them.
With insufficient vertical room, the client clears space for the party display.

For a local Windows check, extract the ZIP into a new folder. Open that folder.
Double-click `kuluu.exe`. Select your existing server profile and sign in with your usual character.
If the launcher asks for game files, close it and open PowerShell in the new folder.
For the installation used during this project, run the following commands.

```powershell
$env:FFXI_DAT_PATH = 'E:\Final Fantasy XI\SquareEnix\FINAL FANTASY XI'
.\kuluu.exe
```

To compare builds, log out and launch the previous client from its original folder.

## Astral Cobalt client pass

The standalone client draws framed windows in cobalt with a silver rule and gold corners.
Commands uses a gilded focus edge. Confirming a submenu draws three stars for 0.84 seconds.
Navigation and server commands take effect as soon as the player confirms.
Chat keeps its channel colors, scroll position, input and history.
The separate HP, MP and TP gauges and the party roster read the current scene snapshot.
An unavailable out-of-zone value displays as two dashes.

The Equipment panel uses the 16 slot locations in the installed English `menu    equip` record.
The slot frame index is 40 plus the row times four plus the column.
The item name, worn slot and available equipment come from the session.
Without an icon table, the cell displays the slot abbreviation.
The Commands and Party headings have extra clearance for the new typeface.

| Commands selection | Game connection in this pass |
| --- | --- |
| Party → Seek Party or Play Solo | The client sends one configuration flag to LandSandBoat. The label changes when the server confirms the state. |
| Party → Member List | A party member or leader can read the current party roster. |
| Equipment | Opens the existing equipment grid, slot picker and session-backed equip operation. |
| Magic, Abilities, Items, Status, Key Items, Communication, Config, Map | Reuses each existing window. Some child actions still lack a complete workflow. |
| Mog House, Current Time, Shut Down, Log Out | Keeps the existing context or direct action. |
| Synthesis, Trade, Search, Linkshell, Region Info, Missions, Quests, View House, Bazaar, Macros, Help Desk | The Commands entry still lacks its complete retail workflow. |

Kuluu relay peers that exchange serialized snapshots require protocol 47 on both ends.
The LandSandBoat game protocol keeps its existing version.

Other Party management commands remain unavailable. The English `prty1`, `prty2`, `prty3`
and `prty7` records supply row positions for solo, member, leader and allied-leader layouts.
Check the allied-leader choice against live retail. The Square Enix Windows manual describes
the solo Seek Party action on printed page 28. LandSandBoat defines the change in
`0x0dc_config.h` and confirms it in `0x0b4_config.h` and self character status.

To check the client on Windows, start the existing server. Extract the client to a new folder,
open it, and double-click `kuluu.exe`. Sign in as usual. Open Commands with the minus key on
the numeric keypad. Choose Equipment and press Enter. Use the arrow keys to inspect the
slots. Press Escape to return. When solo, open Party and select Seek Party. After the server
confirms the change, select Play Solo to clear the flag. In a party, use Member List to see
the roster. Compare the same views on the original client at the same UI resolution to verify retail behavior.
