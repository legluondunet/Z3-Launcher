# Interface assets

- `forest-frame.png`: AI-generated background derived from the selected launcher mockup, using the built-in image generation tool. Embedded with `include_bytes!` and loaded once as an egui texture. No labels or controls are baked into this image.
- `DejaVuSerif.ttf`: DejaVu Serif font, embedded for consistent typography on Linux and Windows. See `DejaVu-LICENSE.txt` for its redistribution license.

All buttons, fields, tabs, text, the header emblem and the journal are rendered by egui. The source assets are included in this project; no separate runtime assets folder is required after compilation. The background scales to the window. Native OS window decorations remain controlled by the desktop.

## Image generation prompt

Use case: background-extraction. Edit target: the supplied exact launcher mockup. Create the production background bitmap for implementing this app, not a UI screenshot. Keep the same dark forest green gently textured background, thin gold outer border, small gold corner filigree and restrained green leafy vine ornaments at the four extreme corners. REMOVE ALL text, ALL buttons, ALL input fields, ALL tabs, ALL separators, ALL scrollbars, ALL icons including the triangle emblem, ALL operating system title bar buttons. Fill their removed regions seamlessly with the same very dark muted green texture. Keep center and at least 90% of the canvas empty dark forest green, low contrast suitable for live UI text overlay. Border occupies outermost 8px, foliage only inside outermost 35px corner areas. Straight front-facing rectangle landscape 4:3, no perspective, no shadows around window, no screenshot widgets, no lettering whatsoever. Match source color palette and subtle Zelda inspired texture closely. This single background will be embedded in egui across a resizable window.
