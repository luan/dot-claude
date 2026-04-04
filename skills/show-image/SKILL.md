---
name: show-image
description: Display images in the user's terminal using timg or chafa. Use this skill whenever the user asks to show, display, preview, or view image files in the terminal — even if they don't mention timg or chafa. Also use when the user wants to visually inspect screenshots, photos, icons, diagrams, or any image files from the command line. Works with single images or multiple images (grid view, side-by-side comparison).
---

# Show Image in Terminal

Claude's Bash tool output cannot render terminal graphics — escape sequences get stripped. Build the correct command and copy it to the user's clipboard so they can paste and run it.

## Step 1: Resolve paths and detect environment

Verify the image file(s) exist, then detect the terminal and available tools:

```bash
echo "TERM_PROGRAM=$TERM_PROGRAM TERM=$TERM TMUX=${TMUX:+yes}" && which timg chafa 2>/dev/null
```

Prefer `timg` over `chafa` — it auto-detects protocols better and has built-in grid layout.

## Step 2: Choose the protocol flag

| Terminal signal | timg flag | chafa flags |
|---|---|---|
| ghostty or kitty, no tmux | (none, auto-detect works) | `--format kitty` |
| ghostty or kitty, in tmux | `-pk` | `--format kitty --passthrough tmux` |
| `iTerm.app`, no tmux | (none) | `--format iterm` |
| `iTerm.app`, in tmux | `-pk` | `--format kitty --passthrough tmux` |
| unknown / other | `-pq` | `--format symbols` |

The tmux override matters because auto-detection often fails there, falling back to low-res output when the terminal actually supports kitty graphics through tmux passthrough.

Terminal signals: `TERM_PROGRAM=ghostty` or `TERM=xterm-ghostty` → ghostty. `TERM=xterm-kitty` → kitty. `TERM_PROGRAM=iTerm.app` → iTerm2. `TMUX` set → inside tmux.

## Step 3: Build the command

Always single-quote file paths (double-quote if path contains `'`).

**timg — single image:**
```bash
timg [-pk] '/path/to/image.png'
```

**timg — multiple images:**
```bash
timg [-pk] --grid=3 --title '/a.png' '/b.png' '/c.png'
# Or for a directory:
timg [-pk] --grid=4 --title '/path/to/images/*'
```

Useful timg options: `-W` (fit width), `-g WxH` (exact cell size), `--frames=1` (still frame for GIFs), `--center` (center in grid).

**chafa — single image (fallback only):**
```bash
chafa --format <fmt> [--passthrough tmux] '/path/to/image.png'
```

**chafa — multiple images (no grid support, use a loop):**
```bash
for f in '/a.png' '/b.png' '/c.png'; do echo "--- $f ---"; chafa --format <fmt> "$f"; done
```

### chafa pitfalls

- Never omit `--format` — auto-detection picks wrong protocols and produces garbage
- Never use `--format sixels` inside tmux — no sixel passthrough support
- Don't set `--size` with pixel formats (kitty/iterm/sixels) — it's character cells not pixels, easy to distort

## Step 4: Copy to clipboard

```bash
echo -n '<the command>' | pbcopy        # macOS
echo -n '<the command>' | xclip -sel c  # Linux
```

Tell the user: "Copied to clipboard — paste and run in your terminal." Include the command text in your message so they can see what they're running.
