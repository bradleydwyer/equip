#!/usr/bin/env bash
set -euo pipefail

# Record a terminal demo GIF of equip in a clean macOS VM.
#
# Uses the equip-base Tart snapshot (pre-configured with brew, gh, asciinema,
# termsvg, node, claude code, and bradleydwyer/tap pre-tapped).
# Equip is installed before recording starts — the demo shows usage, not setup.
# Outputs a GIF to demos/equip-init.gif.
#
# Prerequisites:
#   - tart installed with equip-base snapshot
#   - agg installed on host (brew install agg)
#
# Usage:
#   ./scripts/record-demo.sh                  # record demo
#   ./scripts/record-demo.sh --clean-loadout  # delete bradleydwyer/equip-loadout first

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DEMOS_DIR="$PROJECT_DIR/demos"

VM_NAME="equip-demo"
BASE_IMAGE="equip-base"
CLEAN_LOADOUT=false

for arg in "$@"; do
    case "$arg" in
        --clean-loadout) CLEAN_LOADOUT=true ;;
    esac
done

SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR"
VM_IP=""

safe_delete_loadout() {
    local actual_name
    actual_name=$(gh api repos/bradleydwyer/equip-loadout --jq '.name' 2>/dev/null || echo "")
    if [[ "$actual_name" == "equip-loadout" ]]; then
        gh repo delete bradleydwyer/equip-loadout --yes 2>/dev/null || true
    fi
}

cleanup() {
    echo ""
    echo "==> Cleaning up..."
    tart stop "$VM_NAME" 2>/dev/null || true
    tart delete "$VM_NAME" 2>/dev/null || true

    # Delete the loadout repo created during the demo
    safe_delete_loadout
    echo "  Done."
}
trap cleanup EXIT

run_ssh() {
    ssh $SSH_OPTS "admin@${VM_IP}" "$1"
}

echo "==> Recording equip demo"
echo ""

# --- Prepare output directory ---
mkdir -p "$DEMOS_DIR"

# --- Optionally delete loadout repo for a clean demo ---
if [[ "$CLEAN_LOADOUT" == true ]]; then
    echo "==> Deleting bradleydwyer/equip-loadout..."
    safe_delete_loadout
    echo "    Clean."
    echo ""
fi

# --- Clone & start VM ---
echo "==> Cloning VM from $BASE_IMAGE..."
tart delete "$VM_NAME" 2>/dev/null || true
tart clone "$BASE_IMAGE" "$VM_NAME"

echo "==> Starting VM (headless)..."
tart run --no-graphics "$VM_NAME" &

echo "==> Waiting for VM to boot..."
VM_IP=$(tart ip "$VM_NAME" --wait 60)
echo "    IP: $VM_IP"

for i in $(seq 1 30); do
    if ssh $SSH_OPTS -o ConnectTimeout=5 "admin@${VM_IP}" "true" 2>/dev/null; then
        break
    fi
    sleep 1
done
echo "    SSH ready."
echo ""

# --- Pre-install equip (not part of the recording) ---
echo "==> Installing equip..."
run_ssh 'mkdir -p ~/bin'
scp $SSH_OPTS "$PROJECT_DIR/target/release/equip" "admin@${VM_IP}:bin/equip"
run_ssh 'chmod +x ~/bin/equip && export PATH="$HOME/bin:$PATH" && equip --version'
run_ssh 'mkdir -p ~/.claude ~/.codex'
echo "    Installed."
echo ""

# --- Copy recording script into VM ---
echo "==> Setting up recording script..."
cat <<'DEMO_SCRIPT' | ssh $SSH_OPTS "admin@${VM_IP}" "cat > /tmp/demo.sh && chmod +x /tmp/demo.sh"
#!/bin/bash
eval "$(/opt/homebrew/bin/brew shellenv)"
export PATH="$HOME/bin:$PATH"
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
export HOMEBREW_NO_ENV_HINTS=1
export HOMEBREW_NO_AUTO_UPDATE=1
PROMPT="\033[32m❯\033[0m "

type_cmd() {
    printf "$PROMPT"
    local cmd="$1"
    for (( i=0; i<${#cmd}; i++ )); do
        printf '%s' "${cmd:$i:1}"
        sleep 0.05
    done
    sleep 0.3
    echo
}

sleep 0.3

type_cmd "equip init"
equip init 2>&1
sleep 1

type_cmd "equip install michaelneale/megamind"
equip install michaelneale/megamind 2>&1
sleep 1

type_cmd "equip install bradleydwyer/skills"
equip install bradleydwyer/skills 2>&1
sleep 1

type_cmd "equip remove remember"
equip remove remember 2>&1
sleep 1

type_cmd "equip list --short"
equip list --short 2>&1
sleep 1

printf "$PROMPT"
sleep 1
DEMO_SCRIPT
echo "    Ready."
echo ""

# --- Record ---
echo "==> Recording session..."
run_ssh 'eval "$(/opt/homebrew/bin/brew shellenv)" && asciinema rec /tmp/demo.cast --cols 80 --rows 24 --idle-time-limit 1 -c "bash /tmp/demo.sh" --overwrite'
echo "    Recorded."
echo ""

# --- Copy cast file back ---
echo "==> Copying cast file..."
scp $SSH_OPTS "admin@${VM_IP}:/tmp/demo.cast" "$DEMOS_DIR/equip-init.cast"
echo "    Copied."
echo ""

# --- Convert to GIF ---
echo "==> Converting to GIF..."
agg "$DEMOS_DIR/equip-init.cast" "$DEMOS_DIR/equip-init.gif" \
    --theme dracula \
    --font-size 16 \
    --font-dir /System/Library/Fonts \
    --speed 1.5 \
    --quiet
echo "    Done: $DEMOS_DIR/equip-init.gif"
echo ""

echo "==> Demo recorded successfully!"
ls -lh "$DEMOS_DIR/equip-init.gif"
