---
name: bootstrap:caddy
description: "Register a project in the local dev routing system. Triggers: 'register project', 'add to caddy', 'bootstrap caddy', 'dev routing'."
argument-hint: "<project-name> [port]"
user-invocable: true
disable-model-invocation: true
---

# Bootstrap Caddy

Register a project in the local subdomain routing system (`https://<project>.localhost` via Caddy + dnsmasq).

## Step 1: Parse arguments

The argument string is everything after `/bootstrap:caddy `.
- First word = project name
- Optional second word = port number override

**If the argument string is empty or missing, you MUST ask:**

```
AskUserQuestion: "What project name should be registered?"
```

Do NOT infer the project name from conversation context, working directory, or any other source. Only use the explicit argument.

## Step 2: Check prerequisites

Run this exact command:

```bash
test -f ~/.config/dev-routing/Caddyfile && test -f ~/.config/dev-routing/ports.json && echo "OK" || echo "MISSING"
```

If output is "MISSING", stop immediately and say:

> Dev routing infrastructure not found at `~/.config/dev-routing/`.
> Set it up via dotfiles first (dnsmasq + Caddy).

Do NOT continue. Do NOT attempt to create the infrastructure.

## Step 3: Read port registry

Read `~/.config/dev-routing/ports.json` with the Read tool. Parse the JSON. It has this shape:

```json
{"nextPort": 5200, "projects": {"name": 5200, ...}}
```

Check if the project name from Step 1 exists as a key in `projects`.

- If YES → report existing registration and stop:
  > Already registered: https://<project>.localhost → localhost:<port>
- If NO → continue to Step 4

## Step 4: Assign port

- If user provided a port number in args → use that. Check no other project in `projects` already has that port.
- If no port specified → use the `nextPort` value from ports.json.

## Step 5: Update port registry

Add the project to `projects` with the assigned port. If using auto-assigned port, increment `nextPort`. Write the updated JSON back to `~/.config/dev-routing/ports.json` using the Write tool.

## Step 6: Create Caddy site config

Write `~/.config/dev-routing/sites/<project>.caddy`:

```
<project>.localhost {
    reverse_proxy localhost:<port>
}
```

## Step 7: Configure project dev server

Check if `~/src/<project>` exists.

If the project directory exists:

1. **vite.config.ts** — ensure it reads port from env. The server config should use `Number(process.env.DEV_PORT) || undefined` so it falls back to Vite's default when DEV_PORT is not set. Read the file first. If it already has this pattern, skip. If it has a hardcoded port, replace with the env var pattern. If it has no server block, add one. Example:
   ```typescript
   server: { port: Number(process.env.DEV_PORT) || undefined }
   ```

2. **.env** (gitignored) — append `DEV_PORT=<port>` if not already present. If `.env` doesn't exist, create it with just this line. Do NOT overwrite existing content.

3. **.env.example** (committed) — append `# DEV_PORT=5173` if not already present, so team members know the variable exists.

This keeps the port mapping in gitignored `.env` — nothing local leaks into the repo. Bun auto-loads `.env`, so `process.env.DEV_PORT` is available in vite.config.ts.

If the project directory doesn't exist, skip — report the port so bootstrap:web can set it up during scaffolding.

## Step 8: Reload Caddy

```bash
caddy reload --config ~/.config/dev-routing/Caddyfile 2>&1 || caddy start --config ~/.config/dev-routing/Caddyfile 2>&1
```

## Port Lookup (for other skills)

When any skill needs a project's dev port or URL:
1. Read `~/.config/dev-routing/ports.json` — project key matches directory name under `~/src/`
2. Fallback: check `vite.config.ts` for `server.port`, then Vite dev server output
3. Never assume port 5173
4. Canonical URL: `https://<project>.localhost`

## Step 9: Report

Output exactly:

- Dev URL: `https://<project>.localhost`
- Backend port: `<port>`
- Project configured: yes / no (project not found)
- For .env: `DEV_PORT=<port>`
