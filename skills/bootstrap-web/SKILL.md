---
name: bootstrap-web
description: "Bootstrap a new web project with the preferred stack. Triggers: 'bootstrap web', 'new webapp', 'scaffold project', 'new web project'."
argument-hint: "<project-name> [description]"
user-invocable: true
disable-model-invocation: true
---

# Bootstrap Web

Scaffold a new SvelteKit web app. Researches current ecosystem state
before scaffolding to avoid stale choices.

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
Skill tool: bootstrap-caddy, args: "<project-name>"
```

This assigns a port and creates `http://<project>.localhost`. Use the
returned port in vite.config.ts (`server.port`) and the URL in .env
(`WEBAUTHN_ORIGIN`).

If it fails because infrastructure is missing, stop and tell the user
to set up dev routing via dotfiles first.

## Research Phase

Before writing any files, research the current state of each evolving
choice. Use WebSearch, context7, and current docs.

### What to research

1. **SvelteKit scaffold** — Is there an official `sv create` or
   `create-svelte` CLI? What's the current recommended way to init a
   project? Use it if it supports non-interactive mode with our
   preferences (TypeScript, Tailwind, no demo app). Otherwise scaffold
   manually.

2. **DB layer** — What's the current best way to use D1 SQLite from
   SvelteKit? Options include Drizzle ORM, raw SQL, or alternatives.
   Check what works well with D1 today.

3. **UI components** — What's the current best Svelte 5 component
   library? shadcn-svelte, bits-ui, melt-ui, or something newer?
   Check compatibility with latest Svelte and Tailwind.

4. **Icons** — What's the current best icon solution for SvelteKit?
   unplugin-icons, lucide-svelte, @phosphor-icons/svelte, or other?

5. **WebAuthn library** — Is `@simplewebauthn` still the recommended
   choice? Any better alternatives?

6. **CSS utilities** — Are `clsx` + `tailwind-merge` +
   `tailwind-variants` still the right combo? Or has the ecosystem
   consolidated?

7. **Animation** — Best lightweight animation approach for Tailwind?
   `tw-animate-css`, or something else?

8. **Testing** — Does Vitest still need `@testing-library/svelte` and
   `jsdom`? Or has the testing story changed?

### Research output

After researching, summarize findings as a brief decision list and
present to the user via AskUserQuestion for any choices with multiple
good options. For clear winners, just proceed.

## Design Interview

After research decisions are settled, interview the user about visual
direction using AskUserQuestion. Ask about:

1. **Tone/personality** — What feeling should the app convey?
   Options like: minimal/clean, bold/expressive, playful, editorial,
   brutalist, luxury, retro, organic, industrial. Let the user describe
   in their own words too.

2. **Color direction** — Any colors in mind? Warm vs cool? Muted vs
   vibrant? Monochrome vs colorful? Or "surprise me."

3. **Typography feel** — Serif, sans-serif, mono, mixed? Formal vs
   casual? Or "you pick something distinctive."

Use their answers to select fonts (Google Fonts), build the OKLCH
palette, and shape the layout. Apply frontend-design principles:

- Never Inter, Roboto, Arial, or system fonts
- Cohesive OKLCH palette — dominant direction + sharp accents
- Generous negative space, no cookie-cutter layouts
- Dark-first: `<html class="dark">`
- Match complexity to vision: maximalist needs elaborate code,
  minimalist needs precision and restraint

## Scaffold Phase

After research + design interview, create the project. NEVER hardcode
package versions — always `bun add <package>` and let bun resolve
latest.

### 1. Create project directory

```bash
mkdir -p ~/src/<project-name>
```

### 2. Initialize

Either use the official SvelteKit CLI (if it supports non-interactive
setup) or create package.json manually:

```json
{
  "name": "<project-name>",
  "private": true,
  "version": "0.0.1",
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "prepare": "svelte-kit sync || echo ''",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "test": "vitest run",
    "test:watch": "vitest"
  }
}
```

Add DB scripts if using an ORM (e.g. `db:generate`, `db:push`).
Add `"deploy": "bun run build && wrangler pages deploy"`.

### 3. Install dependencies

Use `bun add` and `bun add -d` without version specifiers. Group by
purpose:

```bash
cd ~/src/<project-name>

# Core (always)
bun add -d svelte @sveltejs/kit @sveltejs/vite-plugin-svelte \
  @sveltejs/adapter-cloudflare vite typescript svelte-check wrangler

# Tailwind (always)
bun add -d @tailwindcss/vite tailwindcss

# Testing (always)
bun add -d vitest <testing-library-if-needed> <dom-env-if-needed>

# Icons (from research)
bun add -d <icon-packages>

# DB layer (from research)
bun add <db-packages>

# Auth (from research)
bun add <auth-packages>

# UI components (from research)
bun add <ui-packages>

# Utilities (from research)
bun add <utility-packages>
```

### 4. Write config files

Write these based on current docs (not hardcoded templates):

- **vite.config.ts** — SvelteKit + Tailwind vite plugins + icon plugin.
  MUST include `server: { port: Number(process.env.DEV_PORT) || undefined }`
  so the port comes from the gitignored `.env` file, not hardcoded.
- **svelte.config.js** — Cloudflare adapter, minimal CSP
- **tsconfig.json** — strict, extends `.svelte-kit/tsconfig.json`
- **vitest.config.ts** — jsdom/happy-dom, test include pattern
- **wrangler.toml** — project name, D1 binding, placeholder DB id
- **DB config** — if using ORM (schema location, sqlite dialect)
- **components.json** — if using shadcn-svelte

Write based on current docs for each package.

### 5. Write app shell

- **src/app.html** — dark class on html, viewport meta, favicon,
  sveltekit placeholders
- **src/app.css** — Tailwind v4 pattern:
  1. `@import "tailwindcss"` + animation import if applicable
  2. `@custom-variant dark (&:is(.dark *));`
  3. `:root` + `.dark` with OKLCH design tokens (background, foreground,
     primary, secondary, muted, accent, destructive, border, input, ring,
     card, popover)
  4. `@theme inline` mapping CSS vars to Tailwind color namespace
  5. `@layer base` with border-border and bg-background defaults
- **src/app.d.ts** — App.Locals (user, db), App.Platform (D1 env)

### 6. Write lib files

- **src/lib/utils.ts** — `cn()` helper (clsx + tailwind-merge)
- **src/lib/db/schema** — users, passkeys, sessions tables
- **src/lib/db/client** — dual-mode client (D1 prod, local dev)
- **src/lib/server/auth** — WebAuthn registration + authentication
  (HMAC-signed challenges, no DB challenge storage).
  Set `rpID` to `localhost` (valid for all `*.localhost` subdomains).
  Set origin from `$env/dynamic/private` WEBAUTHN_ORIGIN.
- **src/lib/server/session** — create/validate/destroy sessions,
  cookie helpers, 30-day duration

### 7. Write UI primitives

At minimum, a Button component with variant/size props using the
chosen component approach (tailwind-variants, shadcn pattern, etc.).

### 8. Write routes

- **src/hooks.server.ts** — DB init, session validation, public path
  allowlist, 401 for API / redirect for pages
- **src/routes/+layout.svelte** — root layout, import app.css, header
- **src/routes/+layout.server.ts** — pass user from locals
- **src/routes/+page.svelte** — home page placeholder
- **src/routes/auth/+page.svelte** — login/register with passkeys
- **src/routes/auth/login/+server.ts** — GET options, POST verify
- **src/routes/auth/register/+server.ts** — GET options, POST create
- **src/routes/auth/logout/+server.ts** — POST destroy session

### 9. Remaining files

- **static/favicon.svg** — simple distinctive SVG for the project
- **.gitignore** — node*modules, .svelte-kit, build, .wrangler, data,
  .DS_Store, *.db, .env/.env.\_ (keep .env.example)
- **.env.example** (committed):
  ```
  # DEV_PORT=5173
  DATABASE_URL=data/<project-name>.db
  WEBAUTHN_RP_ID=localhost
  WEBAUTHN_ORIGIN=http://<project-name>.localhost
  CHALLENGE_SECRET=change-me-to-a-random-secret
  ```
- **.env** (gitignored) — copy from .env.example, then set actual
  values. The bootstrap-caddy skill will have already written
  `DEV_PORT=<port>` here.
- **Initial migration** — generate from ORM or write SQL manually

## Quality Gate

```bash
cd ~/src/<project-name>
bun run prepare
bun run check
```

Both must pass. Fix any type errors. Do NOT return with a broken
scaffold.

## Completion

After quality gate passes, report:

- Project location: `~/src/<project-name>`
- Dev URL: `http://<project-name>.localhost`
- How to start: `cd ~/src/<project-name> && bun dev`
- Reminder: create D1 database with `wrangler d1 create <name>` and
  update wrangler.toml
- Reminder: set real CHALLENGE_SECRET before deploying
- Summary of research decisions made (which packages chosen and why)
