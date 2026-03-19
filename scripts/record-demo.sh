#!/usr/bin/env bash
set -euo pipefail

# Record a terminal demo GIF of equip in a clean macOS VM.
#
# Uses the equip-base Tart snapshot (pre-configured with brew, gh, asciinema,
# termsvg, node, claude code, and bradleydwyer/tap pre-tapped).
# Outputs a GIF to demos/equip-init.gif.
#
# Prerequisites:
#   - tart installed with equip-base snapshot
#   - agg installed on host (brew install agg)
#
# Usage:
#   ./scripts/record-demo.sh                  # record demo
#   ./scripts/record-demo.sh --clean-loadout  # delete bradleydwyer/loadout first

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

cleanup() {
    echo ""
    echo "==> Cleaning up..."
    tart stop "$VM_NAME" 2>/dev/null || true
    tart delete "$VM_NAME" 2>/dev/null || true

    # Delete the loadout repo created during the demo
    gh repo delete bradleydwyer/loadout --yes 2>/dev/null || true
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
    echo "==> Deleting bradleydwyer/loadout..."
    gh repo delete bradleydwyer/loadout --yes 2>/dev/null || true
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

# --- Copy recording script into VM ---
echo "==> Setting up recording script..."
cat <<'DEMO_SCRIPT' | ssh $SSH_OPTS "admin@${VM_IP}" "cat > /tmp/demo.sh && chmod +x /tmp/demo.sh"
#!/bin/bash
eval "$(/opt/homebrew/bin/brew shellenv)"
export HOMEBREW_NO_ENV_HINTS=1
mkdir -p ~/.claude
PROMPT="\033[32m❯\033[0m "

type_cmd() {
    printf "$PROMPT"
    local cmd="$1"
    for (( i=0; i<${#cmd}; i++ )); do
        printf '%s' "${cmd:$i:1}"
        sleep 0.02
    done
    sleep 0.3
    echo
}

sleep 0.3

type_cmd "brew install bradleydwyer/tap/equip"
brew install bradleydwyer/tap/equip 2>&1
sleep 1

type_cmd "equip init"
equip init 2>&1
sleep 1

type_cmd "equip install anthropics/skills/pdf"
equip install anthropics/skills/pdf 2>&1
sleep 1

type_cmd "equip list"
equip list 2>&1
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
    --speed 1 \
    --quiet
echo "    Done: $DEMOS_DIR/equip-init.gif"
echo ""

echo "==> Demo recorded successfully!"
ls -lh "$DEMOS_DIR/equip-init.gif"
