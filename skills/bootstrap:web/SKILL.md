---
name: bootstrap:web
description: "Bootstrap a new web project with the preferred stack. Triggers: 'bootstrap web', 'new webapp', 'scaffold project', 'new web project'."
argument-hint: "<project-name> [description]"
user-invocable: true
disable-model-invocation: true
---

# Bootstrap Web

Scaffold a new SvelteKit web app. Researches current ecosystem state before scaffolding to avoid stale choices.

## Arguments

- First word: project name (required, directory name under `~/src/`)
- Remaining words: project description/purpose (optional)

If no project name, ask with AskUserQuestion.

## Fixed Preferences

These are non-negotiable — do not research alternatives:

| Layer           | Choice                                                                  |
| --------------- | ----------------------------------------------------------------------- |
| Framework       | SvelteKit + Svelte 5 (runes: `$state`, `$derived`, `$props`, `$effect`) |
| Language        | TypeScript (strict mode)                                                |
| Package manager | bun (never npm/pnpm)                                                    |
| Styling         | Tailwind CSS via vite plugin (NO `tailwind.config.*`)                   |
| Colors          | OKLCH color space, CSS custom properties                                |
| Dark mode       | Class-based (`.dark`), dark-first design                                |
| Deploy          | Cloudflare Pages (`@sveltejs/adapter-cloudflare`)                       |
| Database        | D1 SQLite                                                               |
| Auth            | WebAuthn passkeys                                                       |
| Testing         | Vitest                                                                  |
| Build           | Vite                                                                    |

## Dev Routing

Register the project in the local subdomain routing system:

```
Skill tool: bootstrap:caddy, args: "<project-name>"
```

This assigns a port and creates `https://<project>.localhost`. Use the returned port in vite.config.ts (`server.port`) and the URL in .env (`WEBAUTHN_ORIGIN`).

If it fails because infrastructure is missing, stop and tell the user to set up dev routing via dotfiles first.

## Research Phase

Before writing any files, research the current state of each evolving choice. Use WebSearch, context7, and current docs.

### What to research

1. **SvelteKit scaffold** — Is there an official `sv create` or `create-svelte` CLI? What's the current recommended way to init a project? Use it if it supports non-interactive mode with our preferences (TypeScript, Tailwind, no demo app). Otherwise scaffold manually.

2. **DB layer** — What's the current best way to use D1 SQLite from SvelteKit? Options include Drizzle ORM, raw SQL, or alternatives. Check what works well with D1 today.

3. **UI components** — What's the current best Svelte 5 component library? shadcn-svelte, bits-ui, melt-ui, or something newer? Check compatibility with latest Svelte and Tailwind.

4. **Icons** — What's the current best icon solution for SvelteKit? unplugin-icons, lucide-svelte, @phosphor-icons/svelte, or other?

5. **WebAuthn library** — Is `@simplewebauthn` still the recommended choice? Any better alternatives?

6. **CSS utilities** — Are `clsx` + `tailwind-merge` +
   `tailwind-variants` still the right combo? Or has the ecosystem consolidated?

7. **Animation** — Best lightweight animation approach for Tailwind? `tw-animate-css`, or something else?

8. **Testing** — Does Vitest still need `@testing-library/svelte` and `jsdom`? Or has the testing story changed?

### Research output

After researching, summarize findings as a brief decision list and present to the user via AskUserQuestion for any choices with multiple good options. For clear winners, just proceed.

## Design Interview

After research decisions are settled, interview the user about visual direction using AskUserQuestion. Ask about:

1. **Tone/personality** — What feeling should the app convey? Options like: minimal/clean, bold/expressive, playful, editorial, brutalist, luxury, retro, organic, industrial. Let the user describe in their own words too.

2. **Color direction** — Any colors in mind? Warm vs cool? Muted vs vibrant? Monochrome vs colorful? Or "surprise me."

3. **Typography feel** — Serif, sans-serif, mono, mixed? Formal vs casual? Or "you pick something distinctive."

Use their answers to select fonts (Google Fonts), build the OKLCH palette, and shape the layout. Apply `/frontend-design` principles.

## Scaffold Phase

After research + design interview, create the project at `~/src/<project-name>`.

### Constraints

- NEVER hardcode package versions — `bun add <package>` resolves latest
- Write config files based on current docs from research phase, not templates
- vite.config.ts MUST use `server: { port: Number(process.env.DEV_PORT) || undefined }` (port from gitignored `.env`)
- Dark-first: `<html class="dark">`
- WebAuthn: `rpID` = `localhost` (valid for `*.localhost`), origin from `$env/dynamic/private`
- HMAC-signed challenges for WebAuthn (no DB challenge storage)
- `.env.example` committed, `.env` gitignored

### What to Build

Use research results to determine exact packages and APIs. Build:

1. **Config** — vite, svelte, typescript, vitest, wrangler, tailwind, DB ORM
2. **App shell** — app.html, app.css (Tailwind v4 + OKLCH design tokens), app.d.ts
3. **Lib** — cn() helper, DB schema+client, auth (WebAuthn), sessions
4. **Routes** — hooks.server.ts, layout, home page, auth flow (register/login/logout)
5. **UI** — at minimum a Button component with variant/size props
6. **Static** — favicon, .gitignore, .env.example, initial migration

## Quality Gate

```bash
cd ~/src/<project-name>
bun run prepare
bun run check
```

Both must pass. Fix any type errors. Do NOT return with a broken scaffold.

## Completion

After quality gate passes, report:

- Project location: `~/src/<project-name>`
- Dev URL: `https://<project-name>.localhost`
- How to start: `cd ~/src/<project-name> && bun dev`
- Reminder: create D1 database with `wrangler d1 create <name>` and update wrangler.toml
- Reminder: set real CHALLENGE_SECRET before deploying
- Summary of research decisions made (which packages chosen and why)
