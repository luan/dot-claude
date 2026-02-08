---
paths:
  - "**/*.svelte"
  - "**/*.svelte.ts"
---

# Svelte 5 Conventions

## Deprecated Patterns (never use)
- `svelte:component` → use dynamic component syntax
- `on:click`, `on:change`, etc. → use `onclick`, `onchange`
- `<slot>` / `<slot name="x">` → use snippets (`{#snippet}`)
- `$:` reactive statements → use `$derived()`, `$effect()`
- Svelte stores (`writable`, `readable`) → use `$state()`

## Placement Rules
- `{@const}` only inside `{#each}`, `{#if}`, or component
  boundaries — never directly inside HTML elements
  (`<button>`, `<span>`, `<div>`, etc.)

## Runes
- `$state()` for reactive state (replaces `let x = ...` reactive)
- `$derived()` for computed values (replaces `$: x = ...`)
- `$effect()` for side effects (replaces `$: { ... }`)
- `$props()` for component props (replaces `export let`)
- `$bindable()` for two-way bindable props

## Verification
Post-edit hooks run `svelte-check --threshold error` automatically.
If no hook available, run `npx svelte-check` manually after edits.
