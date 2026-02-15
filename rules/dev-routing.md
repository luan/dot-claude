# Dev Server Routing

All local web projects use subdomain routing via Caddy + dnsmasq.
Infrastructure setup lives in ~/dotfiles. Per-project registration
is handled by `/bootstrap-caddy` (also called by `/bootstrap-web`).

## Convention

- Dev servers are at `http://<project-name>.localhost`
- Port registry: `~/.config/dev-routing/ports.json`
- Caddy configs: `~/.config/dev-routing/sites/<project>.caddy`
- Main Caddyfile: `~/.config/dev-routing/Caddyfile`

## For Claude

- **Never assume port 5173.** Always read
  `~/.config/dev-routing/ports.json` to find the port for a project.
- The project name key in ports.json matches the directory name under
  `~/src/`.
- If the project isn't in the registry, check `vite.config.ts` for
  `server.port`, then the Vite dev server output as fallback.
- Use `http://<project>.localhost` as the canonical dev URL.

## Infrastructure (dotfiles)

One-time machine setup:
- dnsmasq: `*.localhost` → 127.0.0.1 (`/etc/resolver/localhost`)
- Caddy: reverse proxy on port 80, imports `sites/*.caddy`
- Config dir: `~/.config/dev-routing/`
- Port registry: `ports.json` (`{"nextPort": 5200, "projects": {...}}`)
- Per-project: `sites/<project>.caddy`
