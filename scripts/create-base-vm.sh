#!/usr/bin/env bash
set -euo pipefail

# Create the equip-base Tart VM snapshot for demo recordings.
#
# Installs: brew, gh, asciinema, termsvg, node, claude code.
# Pre-taps bradleydwyer/tap for fast brew install during demos.
#
# After this script, you must SSH in and run `gh auth login` manually
# to authenticate your GitHub account.
#
# Prerequisites:
#   - tart installed (brew install cirruslabs/cli/tart)
#   - macOS Tahoe base image (tart pull ghcr.io/cirruslabs/macos-tahoe-base:latest)
#
# Usage:
#   ./scripts/create-base-vm.sh
#   ssh admin@$(tart ip equip-base)   # then run: gh auth login
#   tart stop equip-base              # save the snapshot

VM_NAME="equip-base"
OCI_IMAGE="ghcr.io/cirruslabs/macos-tahoe-base:latest"

SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR"
VM_IP=""

run_ssh() {
    ssh $SSH_OPTS "admin@${VM_IP}" "$1"
}

run_sshpass() {
    sshpass -p admin ssh $SSH_OPTS -o PubkeyAuthentication=no "admin@${VM_IP}" "$1"
}

echo "==> Creating equip-base VM"
echo ""

# --- Check prerequisites ---
if ! command -v tart &>/dev/null; then
    echo "Error: tart not installed. Run: brew install cirruslabs/cli/tart"
    exit 1
fi

if ! command -v sshpass &>/dev/null; then
    echo "Error: sshpass not installed. Run: brew install sshpass"
    exit 1
fi

# --- Clone from OCI image ---
echo "==> Cloning VM from $OCI_IMAGE..."
tart delete "$VM_NAME" 2>/dev/null || true
tart clone "$OCI_IMAGE" "$VM_NAME"

echo "==> Starting VM (headless)..."
tart run --no-graphics "$VM_NAME" &

echo "==> Waiting for VM to boot..."
VM_IP=$(tart ip "$VM_NAME" --wait 60)
echo "    IP: $VM_IP"

for i in $(seq 1 30); do
    if sshpass -p admin ssh $SSH_OPTS -o PubkeyAuthentication=no -o ConnectTimeout=5 "admin@${VM_IP}" "true" 2>/dev/null; then
        break
    fi
    sleep 2
done
echo "    SSH ready."
echo ""

# --- Copy SSH key ---
echo "==> Copying SSH key..."
sshpass -p admin ssh-copy-id $SSH_OPTS -o PubkeyAuthentication=no "admin@${VM_IP}" 2>&1
echo "    Key installed."
echo ""

# --- Install Homebrew ---
echo "==> Installing Homebrew..."
run_ssh 'NONINTERACTIVE=1 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"'
run_ssh 'cat >> ~/.zshrc <<EOF
eval "\$(/opt/homebrew/bin/brew shellenv zsh)"
export HOMEBREW_NO_ENV_HINTS=1
EOF'
echo "    Homebrew installed."
echo ""

# --- Install tools ---
echo "==> Installing gh, asciinema, termsvg, node..."
run_ssh 'eval "$(/opt/homebrew/bin/brew shellenv)" && brew install gh asciinema termsvg node'
echo "    Tools installed."
echo ""

# --- Install Claude Code ---
echo "==> Installing Claude Code..."
run_ssh 'eval "$(/opt/homebrew/bin/brew shellenv)" && npm install -g @anthropic-ai/claude-code'
run_ssh 'eval "$(/opt/homebrew/bin/brew shellenv)" && claude --version'
echo "    Claude Code installed."
echo ""

# --- Pre-tap equip ---
echo "==> Pre-tapping bradleydwyer/tap..."
run_ssh 'eval "$(/opt/homebrew/bin/brew shellenv)" && brew tap bradleydwyer/tap'
echo "    Tapped."
echo ""

echo "==> Base VM ready!"
echo ""
echo "    Next steps:"
echo "      1. SSH in:  ssh admin@${VM_IP}"
echo "      2. Run:     gh auth login"
echo "      3. Stop:    tart stop $VM_NAME"
echo ""
echo "    Optional backup:"
echo "      tart export $VM_NAME ~/equip-base.tvm"
