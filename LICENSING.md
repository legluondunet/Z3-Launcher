# Licensing and authorship

Z3-Launcher — Rust implementation
Copyright (C) 2026 legluondunet
Author: legluondunet — https://github.com/legluondunet

This fork is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program. If not, see <https://www.gnu.org/licenses/>.

The SPDX license identifier for this fork is `GPL-3.0-or-later`.
The complete GPL version 3 text is in `LICENSE`.

## Original launcher credit

Z3-Launcher was informed by the installation, settings and controller workflows
of [Zelda 3 Launcher](https://github.com/RadzPrower/Zelda-3-Launcher),
created by Anthony Johns (RadzPrower), Copyright (c) 2023, under the MIT License.

The current application is a Rust/egui reimplementation. A provenance review
of the current repository against the original C#/WinForms sources found no
identifiable copied source code, original icons, or other original assets.
Consequently, the original project's MIT license text is not shipped as a
license for this application. The original project remains MIT-licensed at
its own repository; this historical credit does not relicense it.

Earlier Z3-Launcher revisions included `LICENSE-MIT-original` as a
precautionary notice. Those historical revisions remain available in Git
history. The notice can be reinstated if a future review identifies retained
material from the original project.

## Assets and dependencies

DejaVu Serif retains its own license, reproduced in `assets/DejaVu-LICENSE.txt`.
The generated forest background is described in `assets/README.md` and is
included in the fork under GPL-3.0-or-later to the extent copyright applies.
The packaged resource extractor contains source files from Z3R, separately
licensed under Z3R's upstream MIT terms. Binary packaging preserves the
upstream license at `extractor/Z3R-LICENSE.txt` (inside the AppImage's
bundled extractor for AppImage builds). This MIT notice belongs to Z3R,
not to the historical C#/WinForms launcher.
Third-party dependencies retain their respective licenses.
The external game, downloaded shader collection, ROM and Nintendo game assets
are not relicensed by this notice.

## Binary distribution

When distributing this GPL-covered launcher as an AppImage or Windows binary,
provide the corresponding source for that exact build in accordance with GPL
section 6, including build scripts and any modifications. A source archive
alongside the binary is one practical distribution method. Keep the GPL,
historical project credits and applicable third-party notices with the distribution.
