---
name: bootstrap-caddy
description: "Register a project in the local dev routing system. Triggers: 'register project', 'add to caddy', 'bootstrap caddy', 'dev routing'."
argument-hint: "<project-name> [port]"
user-invocable: true
disable-model-invocation: true
---

# Bootstrap Caddy

Register a project in the local subdomain routing system (`https://<project>.localhost` via Caddy + dnsmasq).

## Step 1: Parse arguments

First word = project name. Optional second = port override. Empty → AskUserQuestion (don't infer from context).

## Step 2: Check prerequisites

```bash
test -f $HOME/.config/dev-routing/Caddyfile && test -f $HOME/.config/dev-routing/ports.json && echo "OK" || echo "MISSING"
```

MISSING → stop: "Dev routing infrastructure not found at `$HOME/.config/dev-routing/`. Set it up via dotfiles first."

## Step 3: Read port registry

Read `$HOME/.config/dev-routing/ports.json`:

```json
{"nextPort": 5200, "projects": {"name": 5200, ...}}
```

Project already exists → report `https://<project>.localhost → localhost:<port>` and stop.

## Step 4: Assign port

User-provided → verify no collision. No port → use `nextPort`.

## Step 5: Update port registry

Add project + port. If auto-assigned, increment `nextPort`. Write back.

## Step 6: Create Caddy site config

Write `$HOME/.config/dev-routing/sites/<project>.caddy`:

```
<project>.localhost {
    reverse_proxy localhost:<port>
}
```

## Step 7: Configure project dev server

Skip if `$HOME/src/<project>` doesn't exist — just report port for later.

If exists:

1. **vite.config.ts** — ensure `server: { port: Number(process.env.DEV_PORT) || undefined }`. Skip if present, replace if hardcoded.
2. **.env** (gitignored) — append `DEV_PORT=<port>` if missing.
3. **.env.example** (committed) — append `# DEV_PORT=5173` if missing.

Bun auto-loads `.env` — no extra setup needed.

## Step 8: Reload Caddy

```bash
caddy reload --config $HOME/.config/dev-routing/Caddyfile 2>&1 || caddy start --config $HOME/.config/dev-routing/Caddyfile 2>&1
```

## Step 9: Report

- **Dev URL:** `https://<project>.localhost`
- **Backend port:** `<port>`
- **Project configured:** yes / no (not found)
- **For .env:** `DEV_PORT=<port>`
