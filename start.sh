#!/usr/bin/env bash
set -Eeuo pipefail

SESSION_NAME="pearlsr"
ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
RELEASE_TAG="${PEARLSR_RELEASE:-build-7}"
RELEASE_ROOT="${ROOT_DIR}/.runtime/${RELEASE_TAG}"
RELEASE_BASE_URL="https://github.com/Pape7rlane/dongdong-s-little-jiujiu-r/releases/download/${RELEASE_TAG}"

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
        "${ROOT_CMD[@]}" apt-get install -y tmux curl unzip
    elif command -v dnf >/dev/null 2>&1; then
        "${ROOT_CMD[@]}" dnf install -y tmux curl unzip
    elif command -v yum >/dev/null 2>&1; then
        "${ROOT_CMD[@]}" yum install -y tmux curl unzip
    elif command -v apk >/dev/null 2>&1; then
        "${ROOT_CMD[@]}" apk add --no-cache tmux curl unzip
    elif command -v pacman >/dev/null 2>&1; then
        "${ROOT_CMD[@]}" pacman -Sy --noconfirm tmux curl unzip
    elif command -v pkg >/dev/null 2>&1; then
        pkg update -y
        pkg install -y tmux curl unzip
    else
        echo "No supported package manager found; install tmux manually." >&2
        exit 1
    fi
}

install_tmux

if ! command -v curl >/dev/null 2>&1 || ! command -v unzip >/dev/null 2>&1; then
    echo "curl and unzip are required to download the release package." >&2
    echo "Install them with your system package manager, then run this script again." >&2
    exit 1
fi

case "$(uname -m)" in
    aarch64|arm64) RELEASE_ASSET="PearlSR-linux-arm64.zip" ;;
    x86_64|amd64) RELEASE_ASSET="PearlSR-linux-x64.zip" ;;
    *)
        echo "Unsupported CPU architecture: $(uname -m)" >&2
        exit 1
        ;;
esac

RELEASE_DIR="${RELEASE_ROOT}/${RELEASE_ASSET%.zip}"
ARCHIVE_PATH="${RELEASE_ROOT}/${RELEASE_ASSET}"
mkdir -p "$RELEASE_DIR"

if [[ ! -x "${RELEASE_DIR}/dispatch" || ! -x "${RELEASE_DIR}/gameserver" ]]; then
    echo "Downloading ${RELEASE_TAG}/${RELEASE_ASSET}..."
    temporary_archive="${ARCHIVE_PATH}.part"
    curl -fL --retry 3 -o "$temporary_archive" "${RELEASE_BASE_URL}/${RELEASE_ASSET}"
    mv -f "$temporary_archive" "$ARCHIVE_PATH"
    unzip -q -o "$ARCHIVE_PATH" -d "$RELEASE_DIR"
    chmod +x "${RELEASE_DIR}/dispatch" "${RELEASE_DIR}/gameserver"
fi

if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    exec tmux attach-session -t "$SESSION_NAME"
fi

tmux new-session -d -s "$SESSION_NAME" -n server -c "$RELEASE_DIR" \
    "./dispatch"
tmux split-window -v -t "$SESSION_NAME:server" -c "$RELEASE_DIR" \
    "./gameserver"
tmux select-layout -t "$SESSION_NAME:server" even-vertical
tmux select-pane -t "$SESSION_NAME:server.0"

exec tmux attach-session -t "$SESSION_NAME"
