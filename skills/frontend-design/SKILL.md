---
name: frontend-design
description: "Create distinctive, production-grade frontend interfaces with high design quality. Triggers: 'build UI', 'design component', 'create page', 'frontend', 'make it look good'. Works with any framework or vanilla HTML/CSS/JS."
user-invocable: true
---

This skill guides creation of distinctive, production-grade frontend interfaces that avoid generic "AI slop" aesthetics. Implement real working code with exceptional attention to aesthetic details and creative choices.

The user provides frontend requirements: a component, page, application, or interface to build. They may include context about the purpose, audience, or technical constraints.

## Stack Detection

Before writing code, determine what the project uses:

1. Read `package.json`, `Cargo.toml`, `pubspec.yaml`, or equivalent
2. Check for existing component patterns (`src/components/`, etc.)
3. Match the project's conventions — file structure, naming, styling approach (CSS modules, Tailwind, styled-components, etc.)
4. If greenfield, ask the user or infer from requirements

Then implement working code in whatever stack fits the project (HTML/CSS/JS, React, Vue, Svelte, Angular, Solid, Flutter, SwiftUI, terminal UI — anything).

## Design Thinking

Before coding, commit to a clear aesthetic direction:

1. **Purpose** — What problem does this solve? Who uses it?
2. **Tone** — Pick a specific aesthetic (brutalist, editorial, retro-futuristic, luxury, playful, etc.) and commit fully. Intentionality over intensity.
3. **Differentiation** — What's the one thing someone will remember?

## Banned Patterns

These signal generic AI output — avoid unconditionally:

- **Fonts:** Inter, Roboto, Arial, Space Grotesk, system fonts
- **Colors:** Purple gradients on white backgrounds, cliched color schemes
- **Layouts:** Cookie-cutter component patterns, predictable symmetry

Every generation must vary: different fonts, different palettes, different themes (light/dark), different aesthetic directions. Never converge across sessions.

## Execution

- Match complexity to vision: maximalist → elaborate animations/effects; minimalist → precision in spacing/typography/subtlety
- Typography: pair a distinctive display font with a refined body font (Google Fonts)
- Color: CSS variables/design tokens. Dominant color + sharp accents > evenly-distributed palettes
- Motion: CSS-first, libraries (Motion, GSAP) when stack supports. One orchestrated page load > scattered micro-interactions
- Spatial: asymmetry, overlap, grid-breaking, generous negative space OR controlled density
- Backgrounds: atmosphere and depth over solid colors (gradients, noise, patterns, layered transparencies)
