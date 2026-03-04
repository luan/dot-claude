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

First word after `/bootstrap-caddy` = project name. Optional second word = port override.

**If empty, ask via AskUserQuestion.** Do NOT infer from context or working directory.

## Step 2: Check prerequisites

```bash
test -f $HOME/.config/dev-routing/Caddyfile && test -f $HOME/.config/dev-routing/ports.json && echo "OK" || echo "MISSING"
```

If "MISSING", stop: "Dev routing infrastructure not found at `$HOME/.config/dev-routing/`. Set it up via dotfiles first."

## Step 3: Read port registry

Read `$HOME/.config/dev-routing/ports.json`:

```json
{"nextPort": 5200, "projects": {"name": 5200, ...}}
```

If project exists → report `https://<project>.localhost → localhost:<port>` and stop.

## Step 4: Assign port

- User-provided port → use it (verify no collision with existing projects)
- No port → use `nextPort` from ports.json

## Step 5: Update port registry

Add project to `projects` with assigned port. If auto-assigned, increment `nextPort`. Write back to `$HOME/.config/dev-routing/ports.json`.

## Step 6: Create Caddy site config

Write `$HOME/.config/dev-routing/sites/<project>.caddy`:

```
<project>.localhost {
    reverse_proxy localhost:<port>
}
```

## Step 7: Configure project dev server

Check if `$HOME/src/<project>` exists. If not, skip — report the port for later setup.

If exists:

1. **vite.config.ts** — ensure `server: { port: Number(process.env.DEV_PORT) || undefined }`. Read first; skip if pattern exists, replace if hardcoded.
2. **.env** (gitignored) — append `DEV_PORT=<port>` if missing. Don't overwrite existing content.
3. **.env.example** (committed) — append `# DEV_PORT=5173` if missing, so team knows the variable exists.

Bun auto-loads `.env`, so `process.env.DEV_PORT` works in vite.config.ts without extra setup.

## Step 8: Reload Caddy

```bash
caddy reload --config $HOME/.config/dev-routing/Caddyfile 2>&1 || caddy start --config $HOME/.config/dev-routing/Caddyfile 2>&1
```

## Step 9: Report

- **Dev URL:** `https://<project>.localhost`
- **Backend port:** `<port>`
- **Project configured:** yes / no (not found)
- **For .env:** `DEV_PORT=<port>`
