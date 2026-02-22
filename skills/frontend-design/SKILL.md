---
name: frontend-design
description: "Create distinctive, production-grade frontend interfaces with high design quality. Triggers: 'build UI', 'design component', 'create page', 'frontend', 'make it look good'. Works with any framework or vanilla HTML/CSS/JS."
user-invocable: true
---

# Frontend Design

Create distinctive, production-grade interfaces that avoid generic "AI slop" aesthetics.

## Workflow

1. **Detect stack**: Read `package.json`/equivalent, check existing component patterns (`src/components/`), match project conventions (file structure, naming, styling approach). Greenfield → ask user or infer.
2. **Design direction**: Before coding, commit to a clear aesthetic — see below.
3. **Implement**: Working code in whatever stack fits (React, Vue, Svelte, vanilla HTML/CSS/JS, Flutter, SwiftUI, terminal UI — anything).
4. **Verify**: Check the result matches the chosen direction, not generic defaults.

## Design Thinking

Before coding, commit to a specific aesthetic direction:

1. **Purpose** — What problem does this solve? Who uses it?
2. **Tone** — Pick a concrete aesthetic (brutalist, editorial, retro-futuristic, luxury, playful, etc.) and commit fully. A strong point of view looks intentional; a lukewarm mix of styles looks accidental.
3. **Differentiation** — What's the one thing someone will remember about this interface?

## Banned Patterns

These create the "AI-generated website" look — sameness across every output:

- **Fonts:** Inter, Roboto, Arial, Space Grotesk, system fonts — these are the default fallbacks every AI reaches for, making output instantly recognizable as generated. Pick fonts that reinforce the chosen aesthetic (Google Fonts has thousands).
- **Colors:** Purple gradients on white backgrounds — the canonical AI palette. Build palettes from the brand/purpose instead. Dominant color + sharp accents > evenly-distributed rainbow.
- **Layouts:** Cookie-cutter symmetry, predictable card grids — these telegraph "template." Break the grid with asymmetry, overlap, or generous negative space.

Every generation must vary: different fonts, palettes, themes, aesthetic directions. Never converge across sessions.

## Execution

- **Match complexity to vision**: maximalist → elaborate animations/effects; minimalist → precision in spacing, typography, subtlety
- **Typography**: pair a distinctive display font with a refined body font
- **Color**: CSS variables/design tokens for consistency
- **Motion**: CSS-first; libraries (Motion, GSAP) when stack supports. One orchestrated page load > scattered micro-interactions
- **Spatial**: asymmetry, overlap, grid-breaking, generous negative space OR controlled density — pick one and commit
- **Backgrounds**: atmosphere and depth over flat solid colors (gradients, noise, patterns, layered transparencies)
