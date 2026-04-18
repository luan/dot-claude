# ct notify

Cross-platform notification handler for Claude Code hooks.
Reads a JSON payload from stdin and sends a desktop notification with per-type icons, sounds, and click-to-focus.

## Setup

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "Notification": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "ct notify"
          }
        ]
      }
    ]
  }
}
```

## Input

JSON on stdin with optional fields:

| Field               | Description                                                        |
|---------------------|--------------------------------------------------------------------|
| `notification_type` | `permission_prompt`, `idle_prompt`, `elicitation_dialog`, or other |
| `title`             | Notification title (falls back to tmux session name)               |
| `message`           | Notification subtitle                                              |

## Behavior by type

| Type                  | Icon   | Color   | Sound           |
|-----------------------|--------|---------|-----------------|
| `permission_prompt`   | lock   | red     | warning         |
| `idle_prompt`         | chat   | blue    | warning         |
| `elicitation_dialog`  | question | amber | warning         |
| other / none          | check  | green   | completion      |

## Features

- **Per-session colored icons** -- uses tmux `@session_color` if set, otherwise falls back to type-based colors
- **Click-to-focus** -- clicking the notification raises the terminal and switches to the correct tmux session + window
- **Notification dedup** -- replaces the previous notification for the same tmux session (Linux)
- **Focus suppression** -- skips sound/bell when the terminal is focused and viewing the active session
- **Tmux attention flag** -- sets `@attention` on the session for status bar integration

## Platform notes

- **Linux**: uses `notify-send`, `paplay`, `xdotool`. GNOME Wayland desaturates custom icons and blocks programmatic window activation.
- **macOS**: uses `grrr` for notifications with sound and click actions.

## Environment variables

| Variable       | Default    | Description                        |
|----------------|------------|------------------------------------|
| `CT_TERMINAL`  | `ghostty`  | Terminal app name for focus detection |

## Testing

Run `scripts/ct-notify-test.sh` to send test notifications:

```bash
# Send all 4 notification types (3s apart)
./scripts/ct-notify-test.sh

# Test with a tmux session color override
./scripts/ct-notify-test.sh '#ff6600'

# Test click-to-focus (sends one notification, wait for click)
./scripts/ct-notify-test.sh focus
```

The test script clears the icon cache before each run so fresh icons are generated.
To verify tmux tab switching, open two tmux sessions side by side and run the focus test from one of them.
