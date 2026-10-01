#!/usr/bin/env bash
set -Eeuo pipefail

SESSION_NAME="pearlsr"
ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

install_tmux() {
    if command -v tmux >/dev/null 2>&1; then
        return
    fi

    local -a ROOT_CMD=()
    if [[ "${EUID}" -ne 0 ]]; then
        if ! command -v sudo >/dev/null 2>&1; then
            echo "tmux is not installed and sudo is unavailable; install tmux manually." >&2
            exit 1
        fi
        ROOT_CMD=(sudo)
    fi

    if command -v apt-get >/dev/null 2>&1; then
        "${ROOT_CMD[@]}" apt-get update
        "${ROOT_CMD[@]}" apt-get install -y tmux
    elif command -v dnf >/dev/null 2>&1; then
        "${ROOT_CMD[@]}" dnf install -y tmux
    elif command -v yum >/dev/null 2>&1; then
        "${ROOT_CMD[@]}" yum install -y tmux
    elif command -v apk >/dev/null 2>&1; then
        "${ROOT_CMD[@]}" apk add --no-cache tmux
    elif command -v pacman >/dev/null 2>&1; then
        "${ROOT_CMD[@]}" pacman -Sy --noconfirm tmux
    elif command -v pkg >/dev/null 2>&1; then
        pkg update -y
        pkg install -y tmux
    else
        echo "No supported package manager found; install tmux manually." >&2
        exit 1
    fi
}

install_tmux

cd "$ROOT_DIR"

if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    exec tmux attach-session -t "$SESSION_NAME"
fi

tmux new-session -d -s "$SESSION_NAME" -n server -c "$ROOT_DIR" \
    "zig build run-dispatch"
tmux split-window -v -t "$SESSION_NAME:server" -c "$ROOT_DIR" \
    "zig build run-gameserver"
tmux select-layout -t "$SESSION_NAME:server" even-vertical
tmux select-pane -t "$SESSION_NAME:server.0"

exec tmux attach-session -t "$SESSION_NAME"
