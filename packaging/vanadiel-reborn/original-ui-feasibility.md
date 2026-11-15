# Running the original FFXI interface with a Kuluu-rendered world

Feasibility investigation, 2026-11-15. The Windows client used for the last playable UI build was `7ac66e8`. This investigation produced no new executable.

## Candidate

Let the original Windows FFXI client own the only LandSandBoat session, keyboard, controller, menu state and chat. Run Kuluu separately to produce the 3D image. Present that image inside the original game window before the original client draws its interface. This would change which client hosts the game. The installed Kuluu client can remain available during the experiment.

## Evidence

- Kuluu pins community project XIClient at `aba6c816d25139f0a4fdb60d05bcec8af5f4d928`. In its reconstruction, `GameManager::MainFlow` draws the primary scene buffer. One path copies the world texture into the UI render target, dispatches menu input and calls the UI draw routine before presenting. The final UI target can therefore already contain the world. Test the earlier world-image draw. Treat this sequence as a hypothesis for the installed Windows build. [XIClient frame loop, lines 2439–2508](https://gitlab.com/Aenge/XIClient/-/blob/aba6c816d25139f0a4fdb60d05bcec8af5f4d928/src/XIClient/source/Game/GameManager.cpp#L2439-2508)
- Square Enix documents separate background and overlay resolution settings. Those settings alone cannot establish where the Windows client blends chat or the cursor. [Square Enix technical support](https://support.na.square-enix.com/faqarticle.php?id=20&kid=12301&q=program)
- The public Ashita v4 SDK exposes Direct3D scene and draw callbacks. Their names alone cannot identify an FFXI menu pass. The archived VDR x86 synthetic test observed eight draws over two armed frames and matched its reference pixels. It did not run the original game. [Ashita SDK](https://github.com/AshitaXI/Ashita-v4beta/blob/main/plugins/sdk/Ashita.h)
- Kuluu has a scene input interface and a headless example that renders to a Bevy image. On the pinned checkout, `cargo test --locked --offline --release -p kuluu-render --lib snapshot::tests` passed 20 tests. This covers scene state without a login. Sharing a live Windows frame, synchronizing the camera, and meeting 60 FPS still need measurement. See [scene input](../../kuluu-render/src/source.rs) and the [headless example](../../kuluu-render/examples/zone-render-headless.rs).

## First experiment to approve

Use an isolated Windows client profile. The original FFXI client logs into the existing private server once. With a reversible test switch, draw a two-color checkerboard at the candidate world-image step. Start with a reference capture with the test off. Capture the same view with it on and again after switching it off.

1. Open Equipment. Move the selection with the controller, inspect an equipped slot, then cancel. Verify the original cursor, item details and window borders remain visible over the pattern.
2. Type in chat, move the caret and scroll history. Verify chat remains interactive and the pattern shows through translucent areas without the old scenery bleeding through.
3. Check target nameplates and zoning fades separately. Their draw order may differ. Record any ordering differences before extending the experiment.
4. Turn the switch off and verify the ordinary scene returns without relogging. A full-window overlay that hides chat, a frozen image or duplicated scenery fails the test.

Only after the backdrop experiment passes, substitute one offscreen Kuluu frame, then a moving view. Validate character position, camera and target alignment as well as input latency and frame times before adopting the design. Test the requested world-click selection behavior separately. The visual experiment does not establish mouse behavior.
