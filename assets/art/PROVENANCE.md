# Original Parkbound artwork

Created 2026-11-16 using the built-in image generation tool for the user-selected detailed classic isometric direction. The images are committed source assets. `assets/manifest.json` records dimensions and SHA256 hashes. Runtime embeds the files. The three preceding style comparisons were concept previews.

## Title background

File `title-gardens.png`. Generated file identifier `exec-3bc7a923-6f26-4aa6-a86c-61739652a69b.png`. Transparency disabled.

Prompt used verbatim follows.

> Create a production-ready full-width 16:9 2560x1440 title background for PARKBOUND, an original isometric theme-park simulation. NO text, lettering, logos, menus, buttons, UI, frames or watermarks anywhere. This is the background behind real game menu controls. Premium classic high-resolution pre-rendered isometric art, rich saturated emerald grass, coral-red and gold steel roller coaster sweeping high across the upper right, detailed gold and cream Ferris wheel far right, ornate red cream carousel at lower right, teal lake with rock edging and fountain, striped cream and green shopfronts, flowerbeds full of small pink yellow white flowers, real miniature guests strolling stone paths, layered rich trees. Strong architectural details on station roofs, exposed beams, rails, supports, ride seats. Warm golden afternoon sunlight, crisp realistic materials at miniature scale, restrained soft natural shadows, orthographic isometric viewpoint, detailed sharp scenery edge to edge. Composition important: LEFTMOST 35 percent is darker shaded dense forest and quiet deep green lawn with subtle park paths, sufficient low-contrast dark background to place readable cream menu controls over. The vivid detailed park action occupies the RIGHT 65 percent. No drastic black seam, let forest transition naturally into park. Original coherent park scene with correct depth ordering and grounded supports. Festive and welcoming, polished commercial game title art. No mobile game imagery or currencies.

The generated dimensions are 1672×941. The renderer fits the image using its near-16:9 aspect ratio.

## Oak foliage

File `oak.png`. Generated file identifier `exec-a1836cf1-2759-48a1-8b76-e1690991d512.png`. Transparency enabled. Actual dimensions 1199×1312.

Prompt used verbatim follows.

> A single original isometric deciduous oak tree sprite for the classic PC theme-park game PARKBOUND, beautifully rendered high-resolution miniature game art. Full tree centered on a truly TRANSPARENT background, from root to crown fully visible, generous transparent margin, NO ground plane, NO shadow patch, NO text or UI. Orthographic isometric game angle, dense rounded but natural crown with many richly detailed individually visible small leaves and layered clusters. Saturated emerald and spring green foliage, soft yellow-green sunlight from upper left, dark cool green lower right shadows, convincing brown trunk and several branching limbs, small exposed roots. Tree is compact and broad with a readable trunk at bottom, slightly taller than wide, composed as one reusable sprite. Fine painted material detail with crisp edges, classic pre-rendered theme park simulation style, premium rich game texture, natural and vivid, not pixel art or a simple cartoon. Preserve alpha transparency around all leaves and branches. One whole tree only.

## Application icon

Created 2026-11-18 using the built-in ImageGen tool, in generate mode with transparency enabled. The user approved the resulting artwork. Source file `parkbound-icon.png` is 1254×1254 RGBA. Generated file identifier `exec-a51c0564-8e10-4044-b317-ad89d271227a.png`.

Prompt used verbatim follows.

> Create one polished Windows game application icon for Parkbound, an original classic isometric amusement-park building game. Square 1:1 icon asset. A bold scarlet-red roller-coaster train with three small cream seats sweeps across the foreground on gleaming golden coaster rails. Behind it, one elegant golden vertical coaster loop forms a strong readable silhouette. A small lush green park island with two rounded trees anchors the composition. Rich forest-green rounded-square enamel badge, delicate warm-gold beveled rim, saturated teal and leaf-green accents, warm ivory highlights. Premium hand-painted isometric game art, charming detailed miniature, crisp silhouettes, directional upper-left lighting, restrained soft shadows. The dominant red train and gold loop must remain immediately recognizable at 32 by 32 pixels, with the fine detail concentrated at large icon sizes. Centered, fills about 88 percent of the canvas with even margins. Transparent pixels outside the rounded-square badge. Clean standalone artwork, entirely original, pictorial icon without typography.

`scripts/make-icon.ps1` performs technical downscaling and ICO packaging with Windows System.Drawing. `parkbound.ico` contains 32-bit alpha PNG frames at 16, 24, 32, 48, 64, 128 and 256 pixels. The source artwork is retained at its generated resolution. SHA256 identities for both files are recorded in `assets/manifest.json`.

## Fonts

The unmodified Barlow Medium and Gloock Regular fonts are distributed under the SIL Open Font License 1.1. The matching copyright and full license text is retained in `assets/fonts/Barlow-OFL.txt` and `assets/fonts/Gloock-OFL.txt`. The packaging script includes the files and appends the notices to `THIRD_PARTY_NOTICES.txt`.

- [Barlow source](https://github.com/google/fonts/tree/main/ofl/barlow)
- [Gloock source](https://github.com/google/fonts/tree/main/ofl/gloock)
