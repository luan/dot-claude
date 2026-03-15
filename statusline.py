#!/usr/bin/env uv run
"""Two-line Claude Code statusline with dot progress bars and quota tracking."""

import json, os, re, sys, subprocess, time
from datetime import datetime, timezone

# Ayu Dark palette
GREEN = "\033[38;5;114m"
ORANGE = "\033[38;5;215m"
RED = "\033[38;5;203m"
DIM = "\033[38;5;242m"
LGRAY = "\033[38;5;250m"
BLUE = "\033[38;5;75m"
CYAN = "\033[38;5;111m"
PURPLE = "\033[38;5;183m"
YELLOW = "\033[38;5;228m"
RESET = "\033[0m"

SEP = f" {DIM}|{RESET} "

GIT_CACHE = "/tmp/claude-statusline-git"
GIT_TTL = 5
USAGE_CACHE = "/tmp/claude-statusline-usage.json"
USAGE_TTL = 120
VERSION_CACHE = "/tmp/claude-statusline-version"
VERSION_TTL = 3600
UPDATE_ICON = "\U000f0047"


CTX_CHARS = ("□", "◧", "■")  # empty, half, filled
CTX_THRESHOLDS = (4, 9)  # 0-3: green, 4-8: orange, 9-11: red
DIM_GREEN = "\033[38;5;65m"
DIM_ORANGE = "\033[38;5;130m"
DIM_RED = "\033[38;5;131m"
DIM_CYAN = "\033[38;5;67m"
SEG_DIGITS = "🯰🯱🯲🯳🯴🯵🯶🯷🯸🯹"


def seg_pct(n, col):
    """Format a number as segmented digit characters with color."""
    return f"{col}{''.join(SEG_DIGITS[int(d)] for d in str(int(n)))}٪{RESET}"


PLUS = "\uf067"
MINUS = "\uf068"

USAGE_CHARS = ("○", "◎", "◉", "●")  # empty, starting, half, filled
USAGE_PACE = "◌"  # empty but within pace window


def context_bar(pct, width=12):
    """Context bar with position-based gradient coloring."""
    pct = max(0, min(100, int(pct or 0)))
    fill = pct * width / 100
    full = int(fill)
    frac = fill - full

    bar = ""
    for i in range(width):
        if i < full:
            level = 1.0
        elif i == full and frac > 0:
            level = frac
        else:
            level = 0

        if level > 0:
            if i < CTX_THRESHOLDS[0]:
                col = DIM_GREEN if i < CTX_THRESHOLDS[0] // 2 else GREEN
            elif i < CTX_THRESHOLDS[1]:
                mid = CTX_THRESHOLDS[0] + (CTX_THRESHOLDS[1] - CTX_THRESHOLDS[0]) // 2
                col = DIM_ORANGE if i < mid else ORANGE
            else:
                mid = CTX_THRESHOLDS[1] + (12 - CTX_THRESHOLDS[1]) // 2
                col = DIM_RED if i < mid else RED
        else:
            col = DIM

        if level <= 0:
            char = CTX_CHARS[0]
        elif level < 0.5:
            char = CTX_CHARS[1]
        else:
            char = CTX_CHARS[2]
        bar += f"{col}{char}"

    label_col = RED if full >= 7 else (ORANGE if full >= 3 else GREEN)
    return f"{bar}{RESET}", label_col


def usage_bar(pct, width=12, col=GREEN, pace_pct=None):
    """Usage bar with single color and optional pace marker."""
    pct = max(0, min(100, int(pct or 0)))
    fill = pct * width / 100
    full = int(fill)
    frac = fill - full

    pace_seg = None
    if pace_pct is not None:
        pace_pos = max(0.0, min(100.0, pace_pct)) * width / 100
        pace_seg = min(int(round(pace_pos)), width - 1)

    ahead = pace_seg is not None and full > pace_seg

    bar = ""
    for i in range(width):
        if i < full:
            level = 1.0
        elif i == full and frac > 0:
            level = frac
        else:
            level = 0

        if level > 0:
            if level < 0.33:
                char = USAGE_CHARS[1]  # ◎
            elif level < 0.66:
                char = USAGE_CHARS[2]  # ◉
            else:
                char = USAGE_CHARS[3]  # ●
            # Ahead of pace: dim the expected portion (left of pace), bright the bonus
            dim = "\033[2m" if ahead and i < pace_seg else ""
            bar += f"{dim}{col}{char}{RESET}"
        elif pace_seg is not None and i <= pace_seg:
            # Empty within expected remaining — should still be filled
            ratio = pct / pace_pct if pace_pct > 0 else 0
            p_col = RED if ratio < 0.8 else (ORANGE if ratio < 1.0 else DIM)
            bar += f"{p_col}{USAGE_PACE}"
        else:
            bar += f"{DIM}{USAGE_CHARS[0]}"

    return f"{bar}{RESET}", col


def fmt_tokens(n):
    if not n:
        return "0"
    if n >= 1_000_000:
        return f"{n / 1_000_000:.1f}m"
    if n >= 1000:
        return f"{n // 1000}k"
    return str(n)


def fmt_cost(usd):
    if not usd:
        return "$0"
    return f"${usd:.2f}" if usd >= 0.01 else f"${usd:.4f}"


def _fmt_duration(secs, show_seconds=False):
    """Format seconds as Xd Yh / Xh YYm / Xm (or Xs / XmYYs with show_seconds)."""
    a = abs(int(secs))
    if a >= 86400:
        return f"{a // 86400}d{(a % 86400) // 3600}h"
    if a >= 3600:
        return f"{a // 3600}h{(a % 3600) // 60:02d}m"
    if show_seconds:
        if a < 60:
            return f"{a}s"
        return f"{a // 60}m{a % 60:02d}s"
    return f"{a // 60}m"


def fmt_duration(ms):
    if not ms:
        return "0s"
    return _fmt_duration(int(ms) // 1000, show_seconds=True)


def fmt_reset(iso_str):
    if not iso_str:
        return "", 0
    try:
        dt = datetime.fromisoformat(iso_str).astimezone()
        now = datetime.now(timezone.utc).astimezone()
        total_secs = max(0, (dt - now).total_seconds())
        return _fmt_duration(total_secs), total_secs
    except Exception:
        return "", 0


def pace_balance_secs(used, remaining_secs, window_secs):
    """Surplus/deficit in seconds vs even pace. Positive = ahead."""
    elapsed = window_secs - remaining_secs
    if elapsed < 60:
        return None
    balance_pct = (100 - used) - (remaining_secs / window_secs * 100)
    return int(round(balance_pct * window_secs / 100))


def fmt_pace(secs, window_secs):
    """Format pace balance as signed time with dim graded color."""
    if secs == 0:
        return ""
    sign = MINUS if secs < 0 else PLUS
    if secs > 0:
        col = DIM_CYAN
    else:
        deficit_pct = abs(secs) / window_secs * 100
        col = DIM_RED if deficit_pct >= 15 else DIM_ORANGE
    ul = "\033[4m" if secs < 0 else ""
    return f"\033[3m{ul}{col}{sign}{_fmt_duration(secs)}{RESET}"


def quota_color(utilization, remaining_secs, window_secs):
    """Color based on remaining quota per remaining time unit."""
    if not window_secs or remaining_secs <= 0:
        if utilization >= 80:
            return RED
        if utilization >= 50:
            return ORANGE
        return CYAN
    is_daily = window_secs > 86400
    unit_secs = 86400 if is_daily else 3600
    total_units = window_secs / unit_secs
    remaining_units = max(remaining_secs / unit_secs, 0.1)
    remaining_pct = 100 - utilization
    per_unit = remaining_pct / remaining_units
    even_pace = 100 / total_units
    if per_unit >= even_pace * 0.7:
        return CYAN
    if per_unit >= even_pace * 0.35:
        return ORANGE
    return RED


# -- Cache helpers --


def read_cache(path, ttl):
    try:
        if time.time() - os.stat(path).st_mtime < ttl:
            with open(path) as f:
                return f.read()
    except OSError:
        pass
    return None


def write_cache(path, value):
    try:
        with open(path, "w") as f:
            f.write(value)
    except OSError:
        pass


def latest_version():
    cached = read_cache(VERSION_CACHE, VERSION_TTL)
    if cached is not None:
        return cached or None
    try:
        raw = subprocess.check_output(
            [
                "curl",
                "-s",
                "--max-time",
                "3",
                "https://registry.npmjs.org/@anthropic-ai/claude-code/latest",
            ],
            stderr=subprocess.DEVNULL,
            text=True,
            timeout=5,
        ).strip()
        ver = json.loads(raw).get("version", "")
        write_cache(VERSION_CACHE, ver)
        return ver
    except Exception:
        write_cache(VERSION_CACHE, "")
        return None


def _run(cmd, cwd=None):
    return subprocess.check_output(
        cmd, cwd=cwd, stderr=subprocess.DEVNULL, text=True, timeout=3
    ).strip()


# -- Data fetchers --


def git_info(cwd):
    if not cwd:
        return None
    import hashlib

    cache_path = GIT_CACHE + "-" + hashlib.md5(cwd.encode()).hexdigest()[:8]
    cached = read_cache(cache_path, GIT_TTL)
    if cached is not None:
        return cached or None

    try:
        branch = _run(["git", "branch", "--show-current"], cwd)
        if not branch:
            branch = _run(["git", "rev-parse", "--short", "HEAD"], cwd)[:7]

        lines = _run(["git", "status", "--porcelain"], cwd).splitlines()
        staged = modified = 0
        for ln in lines:
            if len(ln) < 2:
                continue
            if ln[0] in "AMDRC":
                staged += 1
            if ln[1] in "MD":
                modified += 1

        parts = [f"{LGRAY}󰘬 {branch}{RESET}"]
        if staged:
            parts.append(f"{GREEN}•{staged}{RESET}")
        if modified:
            parts.append(f"{ORANGE}+{modified}{RESET}")

        try:
            raw = _run(["git", "remote", "get-url", "origin"], cwd)
            url = raw
            if url.startswith("git@"):
                url = url.replace(":", "/", 1).replace("git@", "https://")
            if url.endswith(".git"):
                url = url[:-4]
            name = url.rsplit("/", 1)[-1]
            parts.append(f"\033]8;;{url}\a{CYAN}{name}{RESET}\033]8;;\a")
        except Exception:
            parts.append(f"{CYAN}{os.path.basename(cwd)}{RESET}")

        result = " ".join(parts)
    except Exception:
        result = ""

    write_cache(cache_path, result)
    return result or None


CODEXBAR_SNAPSHOT = os.path.expanduser(
    "~/Library/Group Containers/group.com.steipete.codexbar/widget-snapshot.json"
)


def _usage_from_codexbar():
    """Read usage from CodexBar's widget snapshot (no API call needed)."""
    try:
        with open(CODEXBAR_SNAPSHOT) as f:
            snap = json.load(f)
        for entry in snap.get("entries", []):
            if entry.get("provider") != "claude":
                continue
            primary = entry.get("primary") or {}
            secondary = entry.get("secondary") or {}
            result = {}
            if "usedPercent" in primary:
                result["five_hour"] = {
                    "utilization": primary["usedPercent"],
                    "resets_at": primary.get("resetsAt"),
                }
            if "usedPercent" in secondary:
                result["seven_day"] = {
                    "utilization": secondary["usedPercent"],
                    "resets_at": secondary.get("resetsAt"),
                }
            if result:
                return result
    except Exception:
        pass
    return None


def _usage_from_oauth(version=""):
    """Fetch usage from Anthropic OAuth API."""
    raw = _run(
        ["security", "find-generic-password", "-s", "Claude Code-credentials", "-w"]
    )
    try:
        creds = json.loads(raw)
        token = creds.get("claudeAiOauth", {}).get("accessToken")
    except (json.JSONDecodeError, ValueError):
        decoded = bytes.fromhex(raw).decode("utf-8", errors="replace")
        m = re.search(r'"accessToken"\s*:\s*"(sk-ant-[^"]+)"', decoded)
        token = m.group(1) if m else None
    if not token:
        return None

    result = subprocess.check_output(
        [
            "curl",
            "-s",
            "--max-time",
            "5",
            "-H",
            "Accept: application/json",
            "-H",
            f"Authorization: Bearer {token}",
            "-H",
            "anthropic-beta: oauth-2025-04-20",
            "-H",
            f"User-Agent: claude-code/{version or '0.0.0'}",
            "https://api.anthropic.com/api/oauth/usage",
        ],
        stderr=subprocess.DEVNULL,
        text=True,
        timeout=8,
    ).strip()

    if result:
        parsed = json.loads(result)
        if "five_hour" in parsed or "seven_day" in parsed:
            return parsed
    return None


def fetch_usage(version=""):
    cached = read_cache(USAGE_CACHE, USAGE_TTL)
    if cached is not None:
        try:
            return json.loads(cached) if cached else None
        except Exception:
            pass

    # OAuth is authoritative (has reset times); CodexBar is fallback
    try:
        usage = _usage_from_oauth(version)
    except Exception:
        usage = None

    if not usage:
        usage = _usage_from_codexbar()

    if usage:
        write_cache(USAGE_CACHE, json.dumps(usage))
        return usage

    # Preserve last good cache on failure
    cached = read_cache(USAGE_CACHE, USAGE_TTL * 10)
    if cached:
        try:
            parsed = json.loads(cached)
            if "five_hour" in parsed or "seven_day" in parsed:
                return parsed
        except Exception:
            pass
    return None


# -- Main --


def main():
    try:
        data = json.load(sys.stdin)
    except Exception:
        sys.stdout.write(f"{DIM}statusline: waiting{RESET}")
        return

    model_name = (data.get("model") or {}).get("display_name", "")
    cw = data.get("context_window") or {}
    pct = int(cw.get("used_percentage") or 0)
    ctx_size = int(cw.get("context_window_size") or 200000)
    usage = cw.get("current_usage") or {}
    input_tokens = (
        (usage.get("input_tokens") or 0)
        + (usage.get("cache_creation_input_tokens") or 0)
        + (usage.get("cache_read_input_tokens") or 0)
    )

    cost_data = data.get("cost") or {}
    cwd = (data.get("workspace") or {}).get("current_dir", "")
    vim = data.get("vim")
    agent = data.get("agent")

    # Bridge context % to hooks via per-session temp file
    sid = data.get("session_id", "")
    if sid:
        try:
            with open(f"/tmp/claude-context-pct-{sid}", "w") as f:
                f.write(str(pct))
        except OSError:
            pass

    # === LINE 1: Version | Model | context bar pct tokens | cost | duration ===
    parts1 = []
    version = data.get("version", "")
    if version or model_name:
        update_prefix = ""
        if version:
            latest = latest_version()
            if latest and latest != version:
                update_prefix = f"{ORANGE}{UPDATE_ICON} {RESET}"
        small = f"{DIM}{version}{RESET}" if version else ""
        sub = f" {PURPLE}{model_name.lower()}{RESET}" if model_name else ""
        parts1.append(f"{update_prefix}{small}{sub}")
    bar, bar_col = context_bar(pct)
    parts1.append(
        f"{bar} {seg_pct(pct, bar_col)} {DIM}{fmt_tokens(input_tokens)}/{fmt_tokens(ctx_size)}{RESET}"
    )
    parts1.append(f"{LGRAY}󰅐 {fmt_duration(cost_data.get('total_duration_ms'))}{RESET}")
    parts1.append(f"{DIM}󰇁 {fmt_cost(cost_data.get('total_cost_usd'))}{RESET}")
    sys.stdout.write(SEP.join(parts1))

    # === LINE 2: git | 5h quota bar | weekly quota bar | vim | agent ===
    parts2 = []

    gi = git_info(cwd)
    if gi:
        parts2.append(gi)

    quota = fetch_usage(version)
    if quota:
        FIVE_HOURS = 5 * 3600
        SEVEN_DAYS = 7 * 24 * 3600

        fh = quota.get("five_hour") or {}
        fh_used = int(fh.get("utilization") or 0)
        fh_rem = 100 - fh_used
        fh_reset_str, fh_remaining = fmt_reset(fh.get("resets_at"))
        fh_col = quota_color(fh_used, fh_remaining, FIVE_HOURS)
        fh_pace = (fh_remaining / FIVE_HOURS * 100) if fh.get("resets_at") else None
        fh_bar, _ = usage_bar(fh_rem, width=12, col=fh_col, pace_pct=fh_pace)
        fh_label = f"5h: {fh_bar} {seg_pct(fh_rem, fh_col)}"
        if fh_reset_str:
            fh_label += f" {DIM}{fh_reset_str}{RESET}"
        parts2.append(fh_label)

        sd = quota.get("seven_day") or {}
        sd_used = int(sd.get("utilization") or 0)
        sd_rem = 100 - sd_used
        sd_reset_str, sd_remaining = fmt_reset(sd.get("resets_at"))
        sd_col = quota_color(sd_used, sd_remaining, SEVEN_DAYS)
        sd_pace = (sd_remaining / SEVEN_DAYS * 100) if sd.get("resets_at") else None
        sd_bar, _ = usage_bar(sd_rem, width=12, col=sd_col, pace_pct=sd_pace)
        sd_bal = (
            pace_balance_secs(sd_used, sd_remaining, SEVEN_DAYS)
            if sd.get("resets_at")
            else None
        )
        if sd_bal:
            sd_label = f"7d: {sd_bar} {seg_pct(sd_rem, sd_col)} {fmt_pace(sd_bal, SEVEN_DAYS)}"
        else:
            sd_label = f"7d: {sd_bar} {seg_pct(sd_rem, sd_col)}"
        if sd_reset_str:
            sd_label += f" {DIM}{sd_reset_str}{RESET}"
        parts2.append(sd_label)

    if vim:
        parts2.append(f"{PURPLE} {vim['mode']}{RESET}")
    if agent:
        parts2.append(f"{ORANGE}{agent['name']}{RESET}")

    if parts2:
        sys.stdout.write("\n" + SEP.join(parts2))


if __name__ == "__main__":
    main()
