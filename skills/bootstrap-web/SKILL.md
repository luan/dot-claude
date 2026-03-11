---
name: bootstrap-web
description: "Bootstrap a new web project with the preferred stack. Triggers: 'bootstrap web', 'new webapp', 'scaffold project', 'new web project'."
argument-hint: "<project-name> [description] [--auto]"
user-invocable: true
disable-model-invocation: true
---

# Bootstrap Web

Scaffold a new SvelteKit web app. Researches current ecosystem state before scaffolding — stale templates break on install.

## Arguments

First word = project name (directory under `$HOME/src/`). Remaining = description. No name → AskUserQuestion.

## Fixed Stack

- **Framework:** SvelteKit + Svelte 5 (runes), TypeScript strict
- **Tooling:** bun (never npm/pnpm), Vite, Vitest
- **Styling:** Tailwind CSS via vite plugin (no `tailwind.config.*`), OKLCH colors, CSS custom properties
- **Dark mode:** class-based (`.dark`), dark-first
- **Deploy:** Cloudflare Pages (`@sveltejs/adapter-cloudflare`), D1 SQLite
- **Auth:** WebAuthn passkeys

## Dev Routing

Invoke `Skill(bootstrap-caddy, "<project-name>")` for port + `https://<project>.localhost`. Use returned port in vite.config.ts, URL in `.env` (`WEBAUTHN_ORIGIN`). Missing infrastructure → stop and tell user.

## Research Phase

Research current state via WebSearch/context7 before writing any files. Present choices only when multiple strong contenders exist:

1. **SvelteKit scaffold** — `sv create` or manual? Non-interactive support?
2. **DB layer** — Drizzle ORM vs raw SQL vs alternatives for D1
3. **UI components** — shadcn-svelte, bits-ui, or newer Svelte 5-compatible lib
4. **Icons** — unplugin-icons, lucide-svelte, phosphor, etc.
5. **WebAuthn** — `@simplewebauthn` still best?
6. **CSS utilities** — clsx + tailwind-merge + tailwind-variants still right?
7. **Animation** — lightweight Tailwind animation approach
8. **Testing** — Vitest + `@testing-library/svelte` + jsdom still needed?

Clear winners → proceed. `--auto` → pick leading option for close calls. Without `--auto` → AskUserQuestion on close calls.

## Design Interview

`--auto` → defaults: minimal tone, cool+muted colors, sans+formal typography.

Without `--auto`, AskUserQuestion:

1. **Tone** — minimal, bold, playful, editorial, brutalist, etc.
2. **Color direction** — warm/cool, muted/vibrant, monochrome/colorful
3. **Typography** — serif, sans, mono, mixed; formal vs casual

Use answers to select Google Fonts, build OKLCH palette, shape layout per `/frontend-design`.

## Scaffold

Create project at `$HOME/src/<project-name>` using research results.

**Constraints:**
- Never hardcode versions — `bun add <package>` resolves latest
- Config from current docs, not stale templates
- vite.config.ts: `server: { port: Number(process.env.DEV_PORT) || undefined }`
- Dark-first: `<html class="dark">`
- WebAuthn: `rpID = localhost`, origin from `$env/dynamic/private`, HMAC-signed challenges
- `.env.example` committed, `.env` gitignored

**Build:** config files, app shell, lib (cn helper, DB schema, auth, sessions), routes (hooks, layout, home, auth flow), UI components, static assets.

## Quality Gate

```bash
cd $HOME/src/<project-name> && bun run prepare && bun run check
```

Both must pass. Fix type errors before returning. 2 failed attempts → report specific errors.

## Completion

Report: project location, dev URL, `bun dev` to start, research decisions. Remind user: create D1 database (`wrangler d1 create`) and set `CHALLENGE_SECRET` before deploy.
