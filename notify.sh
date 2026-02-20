#!/usr/bin/env bash
# Claude Code notification hook — bell + macOS notification with context.
# Requires: brew tap moltenbits/tap && brew install growlrrr
# Receives JSON on stdin from the Notification hook.

# Skip subagent processes
[[ -n "$CLAUDE_CODE_SUBAGENT" ]] && exit 0

# Read hook JSON from stdin
hook_json=$(cat)
hook_title=$(echo "$hook_json" | jq -r '.title // empty' 2>/dev/null)
hook_message=$(echo "$hook_json" | jq -r '.message // empty' 2>/dev/null)
hook_type=$(echo "$hook_json" | jq -r '.notification_type // empty' 2>/dev/null)

# Detect if user is in Ghostty
frontapp=$(lsappinfo info -only name "$(lsappinfo front)" 2>/dev/null | sed -n 's/.*="\(.*\)"/\1/p')

# Skip entirely if Ghostty is focused and the client is viewing this session
if [[ -n "$TMUX" && "$frontapp" == "Ghostty" ]]; then
  my_session=$(tmux display-message -p '#S' 2>/dev/null)
  client_session=$(tmux display-message -p -t "$(tmux display-message -p '#{client_tty}')" '#{client_session}' 2>/dev/null)
  [[ "$my_session" == "$client_session" ]] && exit 0
fi

# Auto-register Claude app icon on first run
if [[ ! -d ~/.growlrrr/apps/Claude.app ]]; then
  grrr apps add --appId Claude --appIcon ~/.claude/claude.png 2>/dev/null
fi

# Map notification type to sound and symbol
case "$hook_type" in
  permission_prompt) sound="Frog";  symbol="lock" ;;
  idle_prompt)            sound="Frog";  symbol="chat" ;;
  elicitation_dialog)    sound="Frog";  symbol="question" ;;
  *)                     sound="Hero";  symbol="check" ;;
esac

# Capture tmux context
tmux_session=""
session_image=""
if [[ -n "$TMUX" ]]; then
  tmux_session=$(tmux display-message -p '#S' 2>/dev/null)

  # Per-session + per-event colored icon as notification thumbnail
  session_color=$(tmux show-option -t "$tmux_session" -qv @session_color 2>/dev/null)
  if [[ -n "$session_color" ]]; then
    session_image="$HOME/.claude/icons/${tmux_session}-${symbol}.png"
    if [[ ! -f "$session_image" ]]; then
      ~/.claude/gen-circle "$session_color" "$session_image" "$symbol" 2>/dev/null
    fi
  fi
fi

title="${tmux_session:-Claude Code}"
subtitle="${hook_title:-$hook_message}"
message="${hook_message:-Claude Code}"
# Don't repeat subtitle in message; grrr requires a message arg
if [[ "$subtitle" == "$message" ]]; then
  message=" "
fi

# Dock bounce
printf '\a' > /dev/tty 2>/dev/null

# Build click-to-focus command
execute_cmd="open -a Ghostty"

# Flag tmux session for attention indicator in status bar and refresh
if [[ -n "$tmux_session" ]]; then
  tmux set-option -t "$tmux_session" @attention 1 2>/dev/null
  list=$(~/.config/tmux/scripts/session-list.sh 2>/dev/null)
  tmux set -g status-left " $list " \; refresh-client -S 2>/dev/null
fi

# macOS notification (no sound if already in Ghostty)
sound_flag=(--sound "$sound")
[[ "$frontapp" == "Ghostty" ]] && sound_flag=(--sound none)

image_flag=()
[[ -n "$session_image" && -f "$session_image" ]] && image_flag=(--image "$session_image")

grrr --appId Claude --title "$title" --subtitle "$subtitle" "${sound_flag[@]}" "${image_flag[@]}" --execute "$execute_cmd" ${message:+"$message"} >/dev/null 2>&1

exit 0
