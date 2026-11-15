# Original UI resources

## Source

The owner supplied a UI resource archive from the installed FFXI client.
All 46 files match the collector manifest sizes and SHA-256 hashes.
The archive contains the eight window styles and the four static UI DATs.
The archive contains no client executable or live session capture.

## Window artwork

`ROM/0/14.DAT` through `ROM/0/21.DAT` contain `menu    win00`.
Each group contains ten elements with one component per element.
Elements zero and nine contain the background texture and distinct vertex colors.
The other elements contain four corners, two horizontal borders, and two vertical borders.
The parser supplies each component's positions, texture coordinates, four vertex colors, and blend fields.
The border texture coordinates extend beyond the texture dimensions.
The renderer must repeat these textures instead of stretching a cropped border.
The first style has different vertex alpha values within a single corner.
A single color per component loses this gradient.

## Command labels

`ROM/119/51.DAT` contains `menu    windowps` with 859 elements.
The menu labels are complete sprites with authored letter positions and shadows.
The renderer can use these positions without substituting a different font.

| Label | Element |
| --- | --- |
| Status | 13 |
| Equipment | 15 |
| Magic | 17 |
| Search | 18 |
| Items | 19 |
| Party | 20 |
| Abilities | 21 |
| Key Items | 22 |
| Map | 23 |
| Config | 27 |
| Macros | 29 |
| Log Out | 33 |
| Current Time | 34 |
| Shut Down | 35 |

The `menu    comwin` group contains the original title sprites.
Commands uses element 16. Status uses 17. Magic Spells uses 18.
Abilities uses 20. Config uses 21. Item uses 25. Map uses 28.
Equipment uses 37. Communication uses 39.
The `menu    menuwind` layout references the `anc     anc_s` selection sprite.
The `menu    yubi` sprite is a white hand. It is not the same selection sprite.

## Commands layout records

The owner supplied the original English UI resources on 2026-11-15.
The supplied game screenshots show Kuluu on Windows.
The command pane in those screenshots measures approximately 220 pixels across.
The button art measures approximately 88 pixels across.
The first page shows eight rows. The second page mixes button sprites and plain text.

The two original Commands layouts appear in both `ROM/119/51.DAT` and `ROM/0/1.DAT`.
Each pair of layout payloads matches byte for byte.
The first page uses `menu    menuwind`. The second page uses `menu    socialme`.
Sort the button records by their signed vertical position.

| Record | Authored position | Size | Resource group and element |
| --- | --- | --- | --- |
| Both roots | 384, 48 | 112 × 202 | `comwin` 11 and `anc_s` 0 |
| Row *n*, zero based | 16, 6 + 16 × *n* | 88 × 16 | `windowps`, see table below |
| Left page arrow | −16, 7 | 15 × 9 | `comwin` 31 |
| Right page arrow | 113, 7 | 15 × 9 | `comwin` 32 |
| Commands title | 0, 0 | 73 × 9 | `comwin` 16 |

Root coordinates belong to the authored layout. Child coordinates are relative to the pane.
The title record also contains offsets 19 and −3.
The title artwork itself spans x = 19 to 92 and y = −3 to 6.
Applying the offsets a second time would shift the title off center.
The cursor sprite `anc_s` 0 spans x = −32 to 0 and y = −2 to 18 relative to its origin.
The cursor and page arrows have different original artwork.

| Row | Page 1 | Element | Page 2 | Element |
| --- | --- | ---: | --- | ---: |
| 1 | Status | 13 | Missions | 24 |
| 2 | Equipment | 15 | Quests | 26 |
| 3 | Magic | 17 | Key Items | 22 |
| 4 | Items | 19 | View House | 31 |
| 5 | Synthesis | 680 | Bazaar | 30 |
| 6 | Abilities | 21 | Macros | 29 |
| 7 | Party | 20 | Config | 27 |
| 8 | Trade | 28 | Help Desk | 36 |
| 9 | Search | 18 | Current Time | 34 |
| 10 | Linkshell | 16 | Communication | 766 |
| 11 | Region Info | 25 | Shut Down | 35 |
| 12 | Map | 23 | Log Out | 33 |

The row rectangles measure 88 × 16 for every command.
The glyph quads extend to y = 17 on some labels, including Equipment and Region Info.
The View House artwork ends at y = 15.
Keep the 16-unit hit area when the artwork extends above or below its bounds.

These rows come from the owner's DAT. The original client can change which commands are available.
The Synthesis and Party records also reference elements 764 and 765.
The records alone do not establish when the original client selects those secondary resources.

The root artwork, `comwin` 11, has 25 components.
Its first component draws the 112 × 202 panel with the `menu    newtex` window-style texture.
The remaining components draw one black shadow for each button from two quads.
These shadows start two pixels to the right and two pixels below the button.
Each of the eight styles supplies a 128 × 128 `newtex` texture.
The panel texture coordinates extend below the original texture.

The Status label uses `windowps` 13.
No other element in the group has the same sequence of Status glyph texture coordinates.
Element 14 reads “Friend List”. Both elements use the same button background vertex colors.

### Mog House button

The owner's `300px-MainMenu.png` shows a gold Mog House button in the last row on the first page.
The English `windowps` group contains the same lettering in elements 44 and 443.
Element 44 has the bright gold background. Element 443 uses a darker background with lower opacity.
The Map record in `menuwind` gives the position for this row. It has no secondary sprite reference.

LandSandBoat sends the room state in the zone login packet.
Kuluu uses `SceneSnapshot.myroom` to detect the room and excludes the Feretory model.
Inside the room, the client uses element 44 and opens its existing Mog House menu.
Outside the room, the client uses the Map record and opens the map.
The server separately grants Mog Menu access in some outdoor zones.
The reference image does not establish a button change in those zones.

## Published retail comparison

Square Enix's 2011 manual shows the Windows main menu on printed page 25.
The manual describes the numeric keypad minus key and the left and right page controls.
The embedded 445 × 250 illustration shows twelve buttons in the upper-right pane.
The pane and the lower status display share the same right edge.

The pane covers approximately x = 378 to 417 in this illustration.
The button artwork covers approximately x = 384 to 414.
Those widths correspond to the 112-unit pane and the 88-unit buttons at approximately 0.35×.
The selected Status button has a gold background. Other buttons have blue backgrounds.
The two page arrows sit outside the pane at its upper left and upper right.

The manual predates the owner's client. Its compressed print image cannot establish exact colors or current DPI behavior.
The uploaded resource archive contains no live original-client capture or executable.
Screen margins, selection animation, conditional command availability and dynamic text advances still need a matching original-client reference.
The `font    moji` atlas and the static command label font differ.

Source: [Square Enix, FINAL FANTASY XI manual, printed page 25](https://support.na.square-enix.com/document/manual/20/FFXI_manual_vc09_AE5.pdf).

## Owner-supplied retail images

The owner supplied a two-page Commands image and a Mouse/Camera configuration image.
The Commands image shows a white right-pointing cursor on both pages.
The Mouse/Camera image shows a gold cursor. The two images do not identify the cause of the difference.
The Commands records in the supplied install specify a gold double-chevron. The white wedge remains unverified.
The selected label and its button background are gold on both Commands pages.
The first page shows Mog House in the last row. The normal field layout specifies Map in that slot.
The images do not establish every condition that substitutes the Mog House entry.
The Commands title crosses the top edge of the pane. The network block above it has the same width.
The sample combines two cropped menu states. It cannot establish a screen margin.

The median channel ratios of the selected plate in the supplied Commands crop are 1.000, 0.654, and 0.386.
The bright selected glyphs give ratios of 1.000, 0.765, and 0.364.
These samples set the static tint in the renderer. They do not establish the original animation or shader constants.

## Network display

The English `menu    netstat` root starts at (384, 16). The pane measures 112 × 30.
Its `menu    windowps` element 3 contains the Network title and background.
The `menu    netmeter` group contains paired arrow frames. Elements 0–3 are green, 4–7 are yellow, and 8–11 are red.
The percent sign, rate labels, and digits use the installed `font    fontshp` glyphs.
The renderer estimates text placement and activity timing from the supplied reference. Retail thresholds remain unverified.

## Mouse/Camera and Search Results

The supplied Mouse/Camera image matches the English `menu    conf7` layout.
The `menu    configwi` layout links to it through the Mouse/Cam button.
The two columns provide Mouse Control, Screen Edge Panning, Camera View, and separate X and Y axes for each camera perspective.
The installed DAT provides all fourteen buttons and their captions.
Kuluu's existing controller Y inversion has different scope from these controls.

The supplied Search Results window matches the dimensions and row spacing of `menu    scresult`.
The layout contains fifteen 356 × 16 rows. The screenshot shows two header rows and thirteen player rows.
The window artwork supplies no player records. Kuluu's current search client handles auction queries.

## Renderer comparison

The `original-ui-capture` example renders both pages at 100%, 150% and 200% with each of the eight window styles.
It uses the production Commands and status systems with fixture session data.
The example compares rendered pane dimensions, row positions, artwork indices and cursor selection against the observed English layout.
It also checks that every digit in the 100%, S 309 and R 196 readouts fits below the Network title and inside the pane.
It checks for visible pixels in each row and overlap with the status display.
At the normal scale, the Commands pane uses the right column.
At large scales, a tall party roster can reach the Commands pane.
Kuluu moves the Commands pane left when the measured rectangles intersect.
This adjustment keeps the roster and command buttons visible together.
Each file name identifies the page, style, party state and scale.
The capture uses off-screen rendering. It does not measure live game performance.

Run the comparison from the repository root with the English game install selected.
The last argument is a new output directory for the 48 PNG files.

```powershell
$env:FFXI_DAT_PATH = 'E:\Final Fantasy XI\SquareEnix\FINAL FANTASY XI'
cargo run --locked --release -p kuluu-render --example original-ui-capture -- commands-captures
```

## Provenance

The uploaded archive is `VDR-Original-UI-Resources-5eccf8f2.zip`.
The original UI archive remains outside Git.
The collector manifest supplies the individual file hashes.
`ROM/119/51.DAT` SHA-256: `e488c2ca6eb41547534faa56ac92ade7b3f7083e68d36aeda383debf7994bb57`.
`ROM/0/14.DAT` SHA-256: `d0d0f69dd8cedae8799482d13343baaf93a18d11e83c405f1bf95a2280260b96`.
`ROM/0/13.DAT` SHA-256: `9d88e0990c25080ee84434e71c881743147879c43964ce7c969e2e586f08b2e2`.
The survey used the repository chunk, UI element, and texture formats.
Local decoded sheets and component surveys are verification artifacts.
They are not redistributed as project assets.
