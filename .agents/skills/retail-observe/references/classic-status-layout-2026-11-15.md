# Classic status layout reference

## Sources

Reviewed on 2026-11-15:

- Square Enix, FINAL FANTASY XI manual, printed page 20:
  https://support.na.square-enix.com/document/manual/20/FFXI_manual_vc09_AE5.pdf
- Square Enix, 2014-06-17 update, system section:
  https://forum.square-enix.com/ffxi/threads/42611
- Self gauges, attachment 9829:
  https://forum.square-enix.com/ffxi/attachment.php?attachmentid=9829
- Party entries, attachment 9830:
  https://forum.square-enix.com/ffxi/attachment.php?attachmentid=9830

These are published references, not a live retail observation.
The 2014 images show the classic interface with modern TP units.
They do not establish the exact ToAU-era interface.

## Supported layout

The manual places character and party information at the bottom right.
The self image has three rows labeled HP, MP, and TP.
Each row has a gauge and a right-aligned value.
The gauge colors are pink, yellow, and cyan, respectively.
The party image has a name and HP value above a thin HP gauge.
MP and TP values occupy the next line.
The party frame has a centered title.
Neither image has job icons, distance values, activity text, or alternating red bands.

## Implementation limits

Logical dimensions, spacing, and colors are provisional visual tunings.
The self TP gauge saturates at weapon-skill readiness in this first pass.
That fill rule is an inference, not a verified retail calculation.
The client retains its bundled font and flat window frame.
Exact font, gauge textures, window textures, and additive borders need asset verification.
The self and party frames use separate positions in the existing HUD column.
Their exact separation is not established by these cropped images.
Out-of-zone rows retain names and suppress numeric values.
That missing-data policy needs a live comparison.

## Required verification

Check solo and six-member party layouts at 100%, 150%, and 200% UI scale.
Include long names, low HP, zero MP, and an out-of-zone member.
Check zoning, delayed self data, target selection, and custom-layout rollback.
Inspect rendered pixels before claiming visual completion.
These references do not establish complete interface parity.
