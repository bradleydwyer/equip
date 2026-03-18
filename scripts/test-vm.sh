#!/usr/bin/env bash
set -euo pipefail

# Test equip in a clean macOS Tahoe VM via Tart.
#
# Builds equip locally (release), copies the binary into the VM via SCP,
# so you can iterate without releasing to Homebrew.
#
# Prerequisites:
#   - tart installed (brew install cirruslabs/cli/tart)
#   - tahoe-base image pulled (tart clone ghcr.io/cirruslabs/macos-tahoe-base:latest tahoe-base)
#   - gh CLI authenticated on host (token is forwarded to VM)
#   - Rust toolchain installed on host
#
# Usage:
#   ./scripts/test-vm.sh                # full test
#   ./scripts/test-vm.sh --keep         # don't delete VM after (for debugging)
#   ./scripts/test-vm.sh --no-clone     # reuse existing equip-test VM
#   ./scripts/test-vm.sh --no-build     # skip local build, use last binary

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

VM_NAME="equip-test"
BASE_IMAGE="tahoe-base"
KEEP=false
CLONE=true
BUILD=true
TEST_REPO=""  # created dynamically

for arg in "$@"; do
    case "$arg" in
        --keep) KEEP=true ;;
        --no-clone) CLONE=false ;;
        --no-build) BUILD=false ;;
    esac
done

cleanup() {
    echo ""
    echo "==> Cleaning up..."

    # Delete temp test repo if created
    if [[ -n "$TEST_REPO" ]]; then
        echo "  Deleting test repo $TEST_REPO..."
        gh repo delete "$TEST_REPO" --yes 2>/dev/null || true
    fi

    if [[ "$KEEP" == true ]]; then
        echo "  --keep: VM '$VM_NAME' preserved. SSH: ssh admin@\$(tart ip $VM_NAME)"
        tart stop "$VM_NAME" 2>/dev/null || true
        return
    fi

    tart stop "$VM_NAME" 2>/dev/null || true
    tart delete "$VM_NAME" 2>/dev/null || true
    echo "  VM deleted."
}
trap cleanup EXIT

# Get a gh token to forward to the VM
GH_TOKEN=$(gh auth token)
GH_USER=$(gh api user --jq .login)
TEST_REPO="${GH_USER}/equip-test-loadout"

SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR"

run_ssh() {
    ssh $SSH_OPTS "admin@${VM_IP}" "$1"
}

echo "==> equip VM integration test"
echo "    VM: $VM_NAME (from $BASE_IMAGE)"
echo "    User: $GH_USER"
echo ""

# --- Build locally ---
if [[ "$BUILD" == true ]]; then
    echo "==> Building equip (release)..."
    cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml" 2>&1 | tail -1
    echo "    Built: $PROJECT_DIR/target/release/equip"
    echo ""
fi

# --- Clone & start VM ---
if [[ "$CLONE" == true ]]; then
    echo "==> Cloning VM from $BASE_IMAGE..."
    tart delete "$VM_NAME" 2>/dev/null || true
    tart clone "$BASE_IMAGE" "$VM_NAME"
fi

echo "==> Starting VM (headless)..."
tart run --no-graphics "$VM_NAME" &
VM_PID=$!

echo "==> Waiting for VM to boot..."
VM_IP=$(tart ip "$VM_NAME" --wait 60)
echo "    IP: $VM_IP"

# Wait for SSH to be ready
for i in $(seq 1 30); do
    if ssh $SSH_OPTS -o ConnectTimeout=5 "admin@${VM_IP}" "true" 2>/dev/null; then
        break
    fi
    sleep 2
done

echo "    SSH ready."
echo ""

# --- Copy binary into VM ---
echo "==> Copying equip binary to VM..."
scp -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR \
    "$PROJECT_DIR/target/release/equip" "admin@${VM_IP}:/tmp/equip"
run_ssh 'mkdir -p ~/bin && mv /tmp/equip ~/bin/equip && chmod +x ~/bin/equip'
run_ssh 'export PATH="$HOME/bin:$PATH" && equip --version'
echo "    equip installed."
echo ""

# --- Install Homebrew (needed for gh) ---
echo "==> Installing Homebrew..."
run_ssh 'command -v brew >/dev/null 2>&1 || NONINTERACTIVE=1 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"'
run_ssh 'echo "eval \"\$(/opt/homebrew/bin/brew shellenv)\"" >> ~/.zprofile'
echo "    Homebrew installed."
echo ""

# --- Set up gh CLI ---
echo "==> Installing and authenticating gh CLI..."
run_ssh 'eval "$(/opt/homebrew/bin/brew shellenv)" && brew install gh'
# Persist the token via gh auth and set gh as the git credential helper
run_ssh "eval \"\$(/opt/homebrew/bin/brew shellenv)\" && echo '${GH_TOKEN}' | gh auth login --with-token"
run_ssh 'eval "$(/opt/homebrew/bin/brew shellenv)" && gh auth setup-git'
run_ssh 'eval "$(/opt/homebrew/bin/brew shellenv)" && gh auth status'
echo "    gh authenticated."
echo ""

# --- Create temp test repo for sync ---
echo "==> Creating temp sync repo: $TEST_REPO..."
gh repo delete "$TEST_REPO" --yes 2>/dev/null || true
gh repo create "$TEST_REPO" --public --description "equip integration test (auto-delete)"
echo "    Repo created."
echo ""

# --- Run equip commands ---
PASS=0
FAIL=0

check() {
    local desc="$1"
    local cmd="$2"
    printf "  %-50s " "$desc"
    local output
    if output=$(run_ssh "export PATH=\"\$HOME/bin:\$PATH\" && eval \"\$(/opt/homebrew/bin/brew shellenv)\" && $cmd" 2>&1); then
        echo "PASS"
        PASS=$((PASS + 1))
    else
        echo "FAIL"
        echo "    $output" | head -5
        FAIL=$((FAIL + 1))
    fi
}

echo "==> Running integration tests..."
echo ""

check "equip --version" \
    "equip --version"

check "equip init (temp repo)" \
    "equip init ${TEST_REPO}"

check "equip install bradleydwyer/skills" \
    "equip install bradleydwyer/skills --all"

check "equip list shows skills" \
    "equip list --json | grep -q '\"name\"'"

check "equip list includes direct skills" \
    "equip list --json | grep -q console2svg"

check "equip list includes included skills" \
    "equip list --json | grep -q available"

check "equip status shows synced" \
    "equip status --json | grep -q synced"

check "equip outdated runs" \
    "equip outdated"

check "equip survey runs" \
    "equip survey"

check "equip install local path" \
    "mkdir -p /tmp/test-skill && printf -- '---\nname: test-skill\ndescription: test\n---\n# Test' > /tmp/test-skill/SKILL.md && equip install /tmp/test-skill --all"

check "equip remove skill" \
    "equip remove test-skill"

check "equip list --json valid JSON" \
    "equip list --json | python3 -c 'import sys,json; json.load(sys.stdin)'"

check "equip export to file" \
    "equip export --output /tmp/equip-export.json"

check "equip restore --dry-run from file" \
    "equip restore --from /tmp/equip-export.json --dry-run"

echo ""
echo "==> Results: $PASS passed, $FAIL failed"

if [[ "$FAIL" -gt 0 ]]; then
    exit 1
fi
