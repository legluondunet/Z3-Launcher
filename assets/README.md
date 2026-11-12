# Interface assets

## Forest background

- `forest-frame.png`: AI-generated pixel-art forest background derived from the selected launcher mockup supplied during the design process. The user directed and selected successive versions. Its provenance is generative, **not** an export from a Nintendo game.
- The background is embedded with `include_bytes!` and drawn as repeating tiles, with a slight brightness reduction to keep controls legible.
- For testing a different background without rebuilding, place `config/background.png` beside the executable/AppImage; see the main README for other supported locations. The external image takes priority over the embedded image.

### Rights and provenance

The artwork was generated using ChatGPT image generation with user-provided design direction. The fork's `LICENSING.md` includes the generated artwork under GPL-3.0-or-later **to the extent copyright applies**. Whether an AI-assisted output attracts copyright protection depends on applicable law and the human creative contribution. The generation process does not guarantee that third-party rights are absent. Zelda and its original game graphics remain the intellectual property of their respective rights holders; this project does not claim rights over them.

## Other assets

- `DejaVuSerif.ttf`: DejaVu Serif font embedded for consistent typography on Linux and Windows; see `DejaVu-LICENSE.txt` for its redistribution license.
- All buttons, fields, tabs, text, the header emblem and the journal are drawn by egui rather than baked into the background.

## Historical background-generation prompt

The following was an earlier background-extraction prompt kept for provenance; it describes the **former dark mockup** and not necessarily the currently embedded forest tile:

> Use case: background-extraction. Edit target: the supplied exact launcher mockup. Create the production background bitmap for implementing this app, not a UI screenshot. Keep the same dark forest green gently textured background, thin gold outer border, small gold corner filigree and restrained green leafy vine ornaments at the four extreme corners. REMOVE ALL text, ALL buttons, ALL input fields, ALL tabs, ALL separators, ALL scrollbars, ALL icons including the triangle emblem, ALL operating system title bar buttons. Fill their removed regions seamlessly with the same very dark muted green texture. Keep center and at least 90% of the canvas empty dark forest green, low contrast suitable for live UI text overlay. Border occupies outermost 8px, foliage only inside outermost 35px corner areas. Straight front-facing rectangle landscape 4:3, no perspective, no shadows around window, no screenshot widgets, no lettering whatsoever. Match source color palette and subtle Zelda inspired texture closely. This single background will be embedded in egui across a resizable window.
