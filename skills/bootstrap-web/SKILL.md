---
name: bootstrap-web
description: "Bootstrap a new web project with the preferred stack. Triggers: 'bootstrap web', 'new webapp', 'scaffold project', 'new web project'."
argument-hint: "<project-name> [description]"
user-invocable: true
disable-model-invocation: true
---

# Bootstrap Web

Scaffold a new SvelteKit web app. Researches current ecosystem state before scaffolding to avoid stale choices.

## Arguments

First word = project name (directory under `$HOME/src/`). Remaining = description. If no name, ask with AskUserQuestion.

## Fixed Stack (non-negotiable)

- **Framework:** SvelteKit + Svelte 5 (runes), TypeScript strict
- **Tooling:** bun (never npm/pnpm), Vite, Vitest
- **Styling:** Tailwind CSS via vite plugin (no `tailwind.config.*`), OKLCH colors, CSS custom properties
- **Dark mode:** class-based (`.dark`), dark-first
- **Deploy:** Cloudflare Pages (`@sveltejs/adapter-cloudflare`), D1 SQLite
- **Auth:** WebAuthn passkeys

## Dev Routing

Invoke `Skill(bootstrap-caddy, "<project-name>")` to assign a port and create `https://<project>.localhost`. Use returned port in vite.config.ts, URL in `.env` (`WEBAUTHN_ORIGIN`). If infrastructure missing, stop and tell user.

## Research Phase

Before writing files, research current state via WebSearch/context7. For each topic, identify the current best option — present choices to user only when multiple strong contenders exist:

1. **SvelteKit scaffold** — `sv create` or manual? Non-interactive mode support?
2. **DB layer** — Drizzle ORM vs raw SQL vs alternatives for D1
3. **UI components** — shadcn-svelte, bits-ui, or newer Svelte 5-compatible library
4. **Icons** — unplugin-icons, lucide-svelte, phosphor, or other
5. **WebAuthn** — `@simplewebauthn` still best?
6. **CSS utilities** — clsx + tailwind-merge + tailwind-variants still right combo?
7. **Animation** — lightweight Tailwind animation approach
8. **Testing** — Vitest + `@testing-library/svelte` + jsdom still needed?

Summarize findings as a decision list. Clear winners → proceed. Close calls → AskUserQuestion.

## Design Interview

After research, interview user via AskUserQuestion:

1. **Tone** — minimal, bold, playful, editorial, brutalist, etc. (or own words)
2. **Color direction** — warm/cool, muted/vibrant, monochrome/colorful, or "surprise me"
3. **Typography** — serif, sans, mono, mixed; formal vs casual

Use answers to select fonts (Google Fonts), build OKLCH palette, shape layout per `/frontend-design` principles.

## Scaffold

Create project at `$HOME/src/<project-name>` using research results for exact packages/APIs.

**Constraints:**
- Never hardcode versions — `bun add <package>` resolves latest
- Config from current docs, not stale templates
- vite.config.ts: `server: { port: Number(process.env.DEV_PORT) || undefined }`
- Dark-first: `<html class="dark">`
- WebAuthn: `rpID = localhost`, origin from `$env/dynamic/private`, HMAC-signed challenges
- `.env.example` committed, `.env` gitignored

**Build:** config files, app shell (html/css/d.ts), lib (cn helper, DB schema, auth, sessions), routes (hooks, layout, home, auth flow), UI components, static assets.

## Quality Gate

```bash
cd $HOME/src/<project-name> && bun run prepare && bun run check
```

Both must pass. Fix type errors before returning — never ship a broken scaffold. If failures persist after 2 fix attempts, report the specific errors.

## Completion

Report: project location, dev URL, how to start (`bun dev`), and research decisions summary. Remind user to create D1 database (`wrangler d1 create`) and set real `CHALLENGE_SECRET` before deploy.
