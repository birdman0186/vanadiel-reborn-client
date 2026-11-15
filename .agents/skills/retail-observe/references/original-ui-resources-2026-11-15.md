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

## Implementation limits

These files establish the authored artwork and geometry within each sprite.
They do not establish the complete runtime layout, input policy, or animation timing.
The existing command order and available actions still come from Kuluu.
The fixture places the command pane beside the status column.
This placement prevents overlap at large UI scales.
Original viewport anchoring remains unverified.
The original dynamic text advances remain unverified.
The `font    moji` atlas differs from the static label font.
The first frame implementation uses the existing UI color decoder.
The renderer currently uses background element zero.
Active/inactive background switching remains open.
Its blend mapping and window placement still require comparison with the original client.
Do not describe an artwork capture as full original UI parity.

## Provenance

The uploaded archive is `VDR-Original-UI-Resources-5eccf8f2.zip`.
The original UI archive remains outside Git.
The collector manifest supplies the individual file hashes.
`ROM/0/14.DAT` SHA-256: `d0d0f69dd8cedae8799482d13343baaf93a18d11e83c405f1bf95a2280260b96`.
`ROM/0/13.DAT` SHA-256: `9d88e0990c25080ee84434e71c881743147879c43964ce7c969e2e586f08b2e2`.
The survey used the repository chunk, UI element, and texture formats.
Local decoded sheets and component surveys are verification artifacts.
They are not redistributed as project assets.
