#!/usr/bin/env uv run
"""Two-line Claude Code statusline with dot progress bars and quota tracking."""

import io, json, os, re, sqlite3, sys, subprocess, tempfile, time

# Force UTF-8 stdout on Windows
if sys.platform == "win32" and hasattr(sys.stdout, "buffer"):
    sys.stdout = io.TextIOWrapper(
        sys.stdout.buffer, encoding="utf-8", errors="replace", line_buffering=True
    )
from datetime import datetime, timezone
from pathlib import Path

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

ANSI_RE = re.compile(r"\033\[[^m]*m|\033\]8;;[^\a]*\a")


def term_width():
    try:
        return os.get_terminal_size().columns
    except OSError:
        return 120


def visible_len(s):
    return len(ANSI_RE.sub("", s))


def join_segments(segments, max_width):
    """Join segments with SEP, wrapping to new lines when they exceed max_width."""
    if not segments:
        return ""
    sep_w = visible_len(SEP)
    lines = []
    cur = segments[0]
    cur_w = visible_len(cur)
    for seg in segments[1:]:
        seg_w = visible_len(seg)
        if cur_w + sep_w + seg_w <= max_width:
            cur += SEP + seg
            cur_w += sep_w + seg_w
        else:
            lines.append(cur)
            cur = seg
            cur_w = seg_w
    lines.append(cur)
    return "\n".join(lines)

_TMPDIR = Path(tempfile.gettempdir())
GIT_CACHE = str(_TMPDIR / "claude-statusline-git")
GIT_TTL = 5
SID_CACHE = str(_TMPDIR / "claude-statusline-sids")
SID_TTL = 30
VERSION_CACHE = str(_TMPDIR / "claude-statusline-version")
VERSION_TTL = 3600
UPDATE_ICON = "\U000f0047"


CTX_CHARS = ("□", "◧", "■")  # empty, half, filled
CTX_THRESHOLDS = (4, 9)  # 0-3: green, 4-8: orange, 9-11: red
DIM_GREEN = "\033[38;5;65m"
DIM_YELLOW = "\033[38;5;136m"
DIM_ORANGE = "\033[38;5;130m"
DIM_RED = "\033[38;5;131m"
DIM_CYAN = "\033[38;5;67m"
DIM_PINK = "\033[38;5;175m"
SEG_DIGITS = "🯰🯱🯲🯳🯴🯵🯶🯷🯸🯹"


def seg_pct(n, col):
    """Format a number as segmented digit characters with color."""
    v = max(0, int(n))
    return f"{col}{''.join(SEG_DIGITS[int(d)] for d in str(v))}٪{RESET}"


PLUS = "+"
MINUS = "-"

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


def fmt_reset(resets_at):
    if not resets_at:
        return "", 0
    try:
        if isinstance(resets_at, (int, float)):
            total_secs = max(0, resets_at - time.time())
        else:
            dt = datetime.fromisoformat(resets_at).astimezone()
            total_secs = max(0, (dt - datetime.now(timezone.utc)).total_seconds())
        return _fmt_duration(total_secs), total_secs
    except Exception:
        return "", 0


_STATE_DIR = Path(
    os.environ.get("XDG_STATE_HOME") or (Path.home() / ".local" / "state")
) / "claude-statusline"
USAGE_DB = str(_STATE_DIR / "usage.db")
OVERAGE_JSON_LEGACY = str(_STATE_DIR / "overage.json")
MAX_DB_BYTES = 10 * 1024 * 1024

_SCHEMA = """
CREATE TABLE IF NOT EXISTS events (
  ts            INTEGER NOT NULL,
  sid           TEXT    NOT NULL,
  model         TEXT,
  cost          REAL    NOT NULL,
  cost_delta    REAL    NOT NULL,
  overage_delta REAL    NOT NULL,
  fh_used       INTEGER NOT NULL,
  sd_used       INTEGER NOT NULL,
  ctx_pct       INTEGER,
  input_tokens  INTEGER,
  duration_ms   INTEGER,
  cwd           TEXT,
  agent         TEXT,
  PRIMARY KEY (ts, sid)
);
CREATE INDEX IF NOT EXISTS events_sid_ts ON events(sid, ts);
CREATE INDEX IF NOT EXISTS events_ts     ON events(ts);

CREATE TABLE IF NOT EXISTS sessions (
  sid           TEXT PRIMARY KEY,
  first_seen    INTEGER NOT NULL,
  last_seen     INTEGER NOT NULL,
  last_cost     REAL    NOT NULL,
  total_cost    REAL    NOT NULL,
  total_overage REAL    NOT NULL DEFAULT 0,
  model         TEXT,
  cwd           TEXT
);

CREATE TABLE IF NOT EXISTS windows (
  kind     TEXT NOT NULL,
  reset_ts INTEGER NOT NULL,
  overage  REAL NOT NULL DEFAULT 0,
  PRIMARY KEY (kind, reset_ts)
);

CREATE TABLE IF NOT EXISTS usage_samples (
  ts       INTEGER PRIMARY KEY,
  fh_used  INTEGER NOT NULL,
  fh_reset INTEGER NOT NULL,
  sd_used  INTEGER NOT NULL,
  sd_reset INTEGER NOT NULL
);
"""


def _db():
    _STATE_DIR.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(USAGE_DB, timeout=5.0, isolation_level=None)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA synchronous=NORMAL")
    conn.execute("PRAGMA busy_timeout=3000")
    conn.executescript(_SCHEMA)
    _migrate_json(conn)
    return conn


def _migrate_json(conn):
    """One-shot import of overage.json if present. Renames to .migrated after."""
    src = Path(OVERAGE_JSON_LEGACY)
    if not src.exists():
        return
    try:
        state = json.loads(src.read_text())
    except (OSError, ValueError):
        return
    now = int(time.time())
    for sid, sess in (state.get("sessions") or {}).items():
        if isinstance(sess, dict):
            last_cost = float(sess.get("last_cost", 0))
            overage = float(sess.get("overage", 0))
        else:
            last_cost = float(sess or 0)
            overage = 0.0
        conn.execute(
            """INSERT OR IGNORE INTO sessions(sid, first_seen, last_seen, last_cost, total_cost, total_overage)
               VALUES(?,?,?,?,?,?)""",
            (sid, now, now, last_cost, last_cost, overage),
        )
    for kind, reset_key in (("5h", "window_5h_reset"), ("7d", "window_7d_reset")):
        reset = int(state.get(reset_key) or 0)
        overage = float(state.get(f"overage_{kind}") or 0)
        if overage > 0:
            conn.execute(
                """INSERT OR IGNORE INTO windows(kind, reset_ts, overage) VALUES(?,?,?)""",
                (kind, reset, overage),
            )
    month_key = state.get("month")
    month_overage = float(state.get("overage_month") or 0)
    if month_key and month_overage > 0:
        try:
            month_reset = int(datetime.strptime(month_key + "-01", "%Y-%m-%d").timestamp())
            conn.execute(
                """INSERT OR IGNORE INTO windows(kind, reset_ts, overage) VALUES(?,?,?)""",
                ("month", month_reset, month_overage),
            )
        except ValueError:
            pass
    total_overage = float(state.get("overage_total") or 0)
    if total_overage > 0:
        conn.execute(
            """INSERT OR IGNORE INTO windows(kind, reset_ts, overage) VALUES(?,?,?)""",
            ("total", 0, total_overage),
        )
    try:
        src.rename(str(src) + ".migrated")
    except OSError:
        pass


def _db_footprint():
    """Total disk footprint including WAL + SHM sidecars."""
    total = 0
    for suffix in ("", "-wal", "-shm"):
        try:
            total += os.path.getsize(USAGE_DB + suffix)
        except OSError:
            pass
    return total


def _maybe_prune(conn):
    """Keep DB under MAX_DB_BYTES by dropping oldest events. Freed pages get reused."""
    if _db_footprint() <= MAX_DB_BYTES:
        return
    # Drop oldest 20% from both high-volume tables
    for tbl in ("events", "usage_samples"):
        n = conn.execute(f"SELECT COUNT(*) FROM {tbl}").fetchone()[0]
        if n < 100:
            continue
        conn.execute(
            f"DELETE FROM {tbl} WHERE rowid IN (SELECT rowid FROM {tbl} ORDER BY ts LIMIT ?)",
            (n // 5,),
        )
    # Reclaim freelist pages so the file actually shrinks on disk
    try:
        conn.execute("VACUUM")
    except sqlite3.OperationalError:
        pass
    conn.execute("PRAGMA wal_checkpoint(TRUNCATE)")


def _month_reset_ts():
    now = datetime.now()
    return int(datetime(now.year, now.month, 1).timestamp())


def record_tick(data):
    """Record one statusline tick. Returns dict of overage buckets + session overage."""
    empty = {"overage_5h": 0.0, "overage_7d": 0.0, "overage_month": 0.0, "overage_total": 0.0, "overage_session": 0.0}
    sid = data.get("session_id") or ""
    cost_data = data.get("cost") or {}
    cost = cost_data.get("total_cost_usd")
    if not sid or cost is None:
        return empty

    quota = data.get("rate_limits") or {}
    fh = quota.get("five_hour") or {}
    sd = quota.get("seven_day") or {}
    fh_used = int(fh.get("used_percentage") or 0)
    sd_used = int(sd.get("used_percentage") or 0)
    fh_reset = _to_ts(fh.get("resets_at"))
    sd_reset = _to_ts(sd.get("resets_at"))
    month_reset = _month_reset_ts()

    model_field = data.get("model") or ""
    model = model_field.get("display_name") if isinstance(model_field, dict) else str(model_field)
    cw = data.get("context_window") or {}
    ctx_pct = int(cw.get("used_percentage") or 0)
    usage = cw.get("current_usage") or {}
    input_tokens = (
        (usage.get("input_tokens") or 0)
        + (usage.get("cache_creation_input_tokens") or 0)
        + (usage.get("cache_read_input_tokens") or 0)
    )
    duration_ms = int(cost_data.get("total_duration_ms") or 0)
    cwd = (data.get("workspace") or {}).get("current_dir", "")
    agent_name = (data.get("agent") or {}).get("name") if isinstance(data.get("agent"), dict) else None
    capped = fh_used >= 100 or sd_used >= 100
    now = int(time.time())

    conn = _db()
    try:
        conn.execute("BEGIN IMMEDIATE")
        row = conn.execute("SELECT last_cost FROM sessions WHERE sid=?", (sid,)).fetchone()
        last_cost = row[0] if row else cost
        delta = max(0.0, cost - last_cost)
        overage_delta = delta if capped else 0.0

        if row:
            conn.execute(
                """UPDATE sessions SET last_seen=?, last_cost=?, total_cost=?,
                   total_overage=total_overage+?, model=COALESCE(?, model), cwd=COALESCE(?, cwd)
                   WHERE sid=?""",
                (now, cost, cost, overage_delta, model or None, cwd or None, sid),
            )
        else:
            conn.execute(
                """INSERT INTO sessions(sid, first_seen, last_seen, last_cost, total_cost, total_overage, model, cwd)
                   VALUES(?,?,?,?,?,?,?,?)""",
                (sid, now, now, cost, cost, overage_delta, model or None, cwd or None),
            )

        # Skip samples whose window has already expired — stale rate_limit data
        # from a session that hasn't made an API call this window.
        stale = fh_reset <= now or sd_reset <= now
        if not stale:
            # Dedupe within current window: only write when quota values actually change.
            last = conn.execute(
                """SELECT fh_used, sd_used FROM usage_samples
                   WHERE fh_reset=? AND sd_reset=? ORDER BY ts DESC LIMIT 1""",
                (fh_reset, sd_reset),
            ).fetchone()
            if last != (fh_used, sd_used):
                conn.execute(
                    """INSERT OR REPLACE INTO usage_samples(ts, fh_used, fh_reset, sd_used, sd_reset)
                       VALUES(?,?,?,?,?)""",
                    (now, fh_used, fh_reset, sd_used, sd_reset),
                )

        if delta > 0:
            conn.execute(
                """INSERT OR REPLACE INTO events
                   (ts, sid, model, cost, cost_delta, overage_delta, fh_used, sd_used,
                    ctx_pct, input_tokens, duration_ms, cwd, agent)
                   VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)""",
                (now, sid, model or None, cost, delta, overage_delta, fh_used, sd_used,
                 ctx_pct, input_tokens, duration_ms, cwd or None, agent_name),
            )

        if overage_delta > 0:
            for kind, reset in (("5h", fh_reset), ("7d", sd_reset),
                                ("month", month_reset), ("total", 0)):
                conn.execute(
                    """INSERT INTO windows(kind, reset_ts, overage) VALUES(?,?,?)
                       ON CONFLICT(kind, reset_ts) DO UPDATE SET overage=overage+excluded.overage""",
                    (kind, reset, overage_delta),
                )

        buckets = dict(empty)
        for kind, reset, key in (("5h", fh_reset, "overage_5h"),
                                 ("7d", sd_reset, "overage_7d"),
                                 ("month", month_reset, "overage_month"),
                                 ("total", 0, "overage_total")):
            r = conn.execute(
                "SELECT overage FROM windows WHERE kind=? AND reset_ts=?",
                (kind, reset),
            ).fetchone()
            if r:
                buckets[key] = r[0]
        r = conn.execute("SELECT total_overage FROM sessions WHERE sid=?", (sid,)).fetchone()
        if r:
            buckets["overage_session"] = r[0]
        conn.execute("COMMIT")
        _maybe_prune(conn)
        return buckets
    except sqlite3.Error:
        try:
            conn.execute("ROLLBACK")
        except sqlite3.Error:
            pass
        return empty
    finally:
        conn.close()


def _to_ts(v):
    if v is None:
        return 0
    if isinstance(v, (int, float)):
        return int(v)
    try:
        return int(datetime.fromisoformat(v).timestamp())
    except Exception:
        return 0


def pace_balance_secs(used, remaining_secs, window_secs):
    """Surplus/deficit in seconds vs even pace. Positive = ahead."""
    elapsed = window_secs - remaining_secs
    if elapsed < 60:
        return None
    balance_pct = (100 - used) - (remaining_secs / window_secs * 100)
    return int(round(balance_pct * window_secs / 100))


def fmt_pace(secs, window_secs, **_):
    """Format pace balance as integer hours."""
    if secs == 0:
        return ""
    sign = MINUS if secs < 0 else PLUS
    if secs > 0:
        col = DIM_CYAN
    else:
        deficit_pct = abs(secs) / window_secs * 100
        if deficit_pct >= 15:
            col = DIM_RED
        elif deficit_pct >= 8:
            col = DIM_ORANGE
        else:
            col = DIM_YELLOW
    a = abs(secs)
    hours = round(a / 3600)
    txt = f"{hours}h"
    seg = "".join(SEG_DIGITS[int(c)] if c.isdigit() else c for c in txt)
    return f"\033[3m{col}{sign}{seg}{RESET}"


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


def unique_sid_prefix(sid):
    """Shortest prefix of sid that's unique among all transcript session IDs."""
    cached = read_cache(SID_CACHE, SID_TTL)
    if cached is not None:
        all_sids = cached.splitlines()
    else:
        claude_dir = str(Path.home() / ".claude" / "projects")
        all_sids = []
        try:
            for root, _, files in os.walk(claude_dir):
                for f in files:
                    if f.endswith(".jsonl"):
                        all_sids.append(f[:-6])  # strip .jsonl
        except OSError:
            pass
        write_cache(SID_CACHE, "\n".join(all_sids))

    others = [s for s in all_sids if s != sid]
    for length in range(3, len(sid) + 1):
        prefix = sid[:length]
        if not any(o.startswith(prefix) for o in others):
            return prefix
    return sid[:8]


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


def _trunc_wt(name, max_len=6):
    if len(name) <= max_len:
        return name
    return name[:2] + ".." + name[-2:]


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

        try:
            wt_lines = [l for l in _run(["git", "worktree", "list"], cwd).splitlines() if l.strip()]
            if len(wt_lines) > 1:
                toplevel = _run(["git", "rev-parse", "--show-toplevel"], cwd)
                wt_name = os.path.basename(toplevel)
                parts.append(f"\033[3m{DIM_PINK}{_trunc_wt(wt_name)}\033[23m{RESET}")
        except Exception:
            pass

        result = " ".join(parts)
    except Exception:
        result = ""

    write_cache(cache_path, result)
    return result or None


# -- Main --


def main():
    try:
        data = json.load(sys.stdin)
    except Exception:
        sys.stdout.write(f"{DIM}statusline: waiting{RESET}")
        return

    model_field = data.get("model") or ""
    model_raw = model_field.get("display_name", "") if isinstance(model_field, dict) else str(model_field)
    model_name = re.sub(r"\s*\((\d+\w*)\s+context\)", r" \1", model_raw)
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
    quota = data.get("rate_limits") or {}

    # Bridge context % to hooks via per-session temp file
    sid = data.get("session_id", "")
    if sid:
        try:
            with open(str(_TMPDIR / f"claude-context-pct-{sid}"), "w") as f:
                f.write(str(pct))
        except OSError:
            pass

    try:
        overage = record_tick(data)
    except Exception:
        overage = {"overage_5h": 0.0, "overage_7d": 0.0, "overage_month": 0.0, "overage_total": 0.0, "overage_session": 0.0}

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
    if overage["overage_session"] > 0:
        parts1.append(f"{RED}󰇁 +{fmt_cost(overage['overage_session'])[1:]}{RESET}")
    else:
        parts1.append(f"{DIM}󰇁 {fmt_cost(cost_data.get('total_cost_usd'))}{RESET}")
    width = term_width()
    line1 = join_segments(parts1, width)
    if sid:
        short = unique_sid_prefix(sid)
        line1 += f" \033[38;5;239m\uf120 {short}{RESET}"
    sys.stdout.write(line1)

    # === LINE 2: git | 5h quota bar | weekly quota bar | vim | agent ===
    parts2 = []

    gi = git_info(cwd)
    if gi:
        parts2.append(gi)

    suffix_parts = []
    if vim:
        suffix_parts.append(f"{PURPLE} {vim['mode']}{RESET}")
    if agent:
        suffix_parts.append(f"{ORANGE}{agent['name']}{RESET}")

    if quota:
        FIVE_HOURS = 5 * 3600
        SEVEN_DAYS = 7 * 24 * 3600

        fh = quota.get("five_hour") or {}
        fh_used = int(fh.get("used_percentage") or 0)
        fh_rem = 100 - fh_used
        fh_reset_str, fh_remaining = fmt_reset(fh.get("resets_at"))
        fh_col = quota_color(fh_used, fh_remaining, FIVE_HOURS)
        fh_pace = (fh_remaining / FIVE_HOURS * 100) if fh.get("resets_at") else None

        sd = quota.get("seven_day") or {}
        sd_used = int(sd.get("used_percentage") or 0)
        sd_rem = 100 - sd_used
        sd_reset_str, sd_remaining = fmt_reset(sd.get("resets_at"))
        sd_col = quota_color(sd_used, sd_remaining, SEVEN_DAYS)
        sd_pace = (sd_remaining / SEVEN_DAYS * 100) if sd.get("resets_at") else None
        sd_bal = (
            pace_balance_secs(sd_used, sd_remaining, SEVEN_DAYS)
            if sd.get("resets_at")
            else None
        )

        usage_seg = ""
        for bw in range(12, 3, -1):
            fh_bar, _ = usage_bar(fh_rem, width=bw, col=fh_col, pace_pct=fh_pace)
            fh_label = f"5h: {fh_bar} {seg_pct(fh_rem, fh_col)}"
            if fh_reset_str:
                fh_label += f" {DIM}{fh_reset_str}{RESET}"

            sd_bar, _ = usage_bar(sd_rem, width=bw, col=sd_col, pace_pct=sd_pace)
            if sd_bal:
                sd_label = f"7d: {sd_bar} {seg_pct(sd_rem, sd_col)} {fmt_pace(sd_bal, SEVEN_DAYS)}"
            else:
                sd_label = f"7d: {sd_bar} {seg_pct(sd_rem, sd_col)}"
            if sd_reset_str:
                sd_label += f" {DIM}{sd_reset_str}{RESET}"

            usage_seg = fh_label + SEP + sd_label
            if visible_len(usage_seg) <= width:
                break

        parts2.append(usage_seg)

    parts2.extend(suffix_parts)

    if parts2:
        sys.stdout.write("\n" + join_segments(parts2, width))


if __name__ == "__main__":
    main()
