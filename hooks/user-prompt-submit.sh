#!/bin/bash

# UserPromptSubmit hook - auto-discover Neovim socket

# Check if we've already found a socket this session
SESSION_SOCKET_FILE="/tmp/claude_nvim_session_$$"

# Only run on first prompt of session (when no socket stored yet)
if [[ ! -f "$SESSION_SOCKET_FILE" ]]; then
  # Check if Neovim created a socket with hashed project path
  PROJECT_HASH=$(pwd | shasum -a 256 | cut -d' ' -f1)
  NVIM_SOCKET_PATH="/tmp/claude_nvim_socket_${PROJECT_HASH}"

  if [[ -S "$NVIM_SOCKET_PATH" ]] && timeout 2 nvim --server "$NVIM_SOCKET_PATH" --remote-expr "1" >/dev/null 2>&1; then
    # Mark that we've found a socket this session
    echo "$NVIM_SOCKET_PATH" >"$SESSION_SOCKET_FILE"

    # Output socket path directly for Claude to use
    echo "CLAUDE_NVIM_SOCKET=$NVIM_SOCKET_PATH"

    # Instruct Claude to acknowledge the socket discovery
    echo "CLAUDE_INSTRUCTION: Acknowledge that Neovim socket has been discovered and is ready for use"
    exit 0
  fi

fi

# Always allow the prompt to continue
exit 0
