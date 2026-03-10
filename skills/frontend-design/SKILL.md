---
name: frontend-design
description: "Create distinctive, production-grade frontend interfaces with high design quality. Triggers: 'build UI', 'design component', 'create page', 'frontend', 'make it look good'. Works with any framework or vanilla HTML/CSS/JS."
user-invocable: true
---

# Frontend Design

Create distinctive, production-grade interfaces that avoid the generic "AI slop" aesthetic.

## Workflow

1. **Detect stack**: Read `package.json`/equivalent, check existing component patterns, match project conventions. Greenfield → ask user or infer.
2. **Design direction**: Commit to a clear aesthetic before writing code — see below.
3. **Implement**: Working code in whatever stack fits (React, Vue, Svelte, vanilla HTML/CSS/JS, Flutter, SwiftUI, terminal UI).
4. **Verify**: Check the result matches the chosen direction, not generic defaults.

## Design Thinking

Before coding, commit to a specific aesthetic direction:

1. **Purpose** — What problem does this solve? Who uses it?
2. **Tone** — Pick a concrete aesthetic (brutalist, editorial, retro-futuristic, luxury, playful, etc.) and commit fully. A strong point of view looks intentional; mixing styles looks accidental.
3. **Differentiation** — What's the one thing someone will remember about this interface?

## Banned Patterns

These produce the "AI-generated" look — sameness across every output:

- **Fonts:** Inter, Roboto, Arial, Space Grotesk, system fonts — the defaults every AI reaches for. Pick fonts that reinforce the aesthetic (Google Fonts has thousands).
- **Colors:** Purple gradients on white — the canonical AI palette. Build palettes from brand/purpose. Dominant color + sharp accents > rainbow.
- **Layouts:** Cookie-cutter symmetry, predictable card grids — telegraph "template." Break the grid.

Every generation must vary. Never converge on the same fonts, palettes, or layout patterns across sessions.

## Execution

- **Match complexity to vision**: maximalist → elaborate animations; minimalist → precision in spacing and typography
- **Typography**: pair a distinctive display font with a refined body font
- **Color**: CSS variables/design tokens for consistency
- **Motion**: CSS-first; libraries (Motion, GSAP) when stack supports. One orchestrated page load > scattered micro-interactions
- **Spatial**: asymmetry, overlap, grid-breaking, generous negative space OR controlled density — pick one
- **Backgrounds**: atmosphere and depth over flat solid colors (gradients, noise, patterns, layered transparencies)
