#!/usr/bin/env bash
# Focus Ghostty and switch tmux to the given session.
# Called by grrr --execute on notification click.
# Args: <session> <tmux-binary>
# tmux path is passed in because notification handlers run with minimal PATH.
session="$1"
tmux="$2"
open -a Ghostty &
client=$("$tmux" list-clients -F '#{client_tty}' | head -1)
[[ -n "$client" ]] && "$tmux" switch-client -c "$client" -t "$session"
