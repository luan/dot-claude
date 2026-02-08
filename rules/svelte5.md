---
paths:
  - "**/*.svelte"
  - "**/*.svelte.ts"
---

# Svelte 5

## Deprecated (never use)
- `svelte:component` → dynamic component syntax
- `on:click`, `on:change` → `onclick`, `onchange`
- `<slot>` / `<slot name="x">` → snippets (`{#snippet}`)
- `$:` reactive → `$derived()`, `$effect()`
- Svelte stores (`writable`, `readable`) → `$state()`

## Placement
- `{@const}` only inside `{#each}`, `{#if}`, or component
  boundaries — never in HTML elements (`<button>`, `<div>`, etc.)

## Runes
- `$state()` — reactive state (replaces reactive `let`)
- `$derived()` — computed (replaces `$: x = ...`)
- `$effect()` — side effects (replaces `$: { ... }`)
- `$props()` — component props (replaces `export let`)
- `$bindable()` — two-way bindable props

## Verification
Post-edit hooks run `svelte-check --threshold error`.
No hook → `npx svelte-check` manually.
