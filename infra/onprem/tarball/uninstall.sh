#!/usr/bin/env bash
# uninstall.sh — paired with tarball/install-from-tarball.sh.
# install-from-tarball is a one-shot binary install with no persistent state
# besides /usr/local/bin/oya-ops-workspace-shell, which is owned by foundry/install.sh.
# So this uninstall is a no-op pointer to foundry/uninstall.sh.
set -uo pipefail
echo "tarball install has no persistent state beyond /usr/local/bin/oya-ops-workspace-shell."
echo "To remove the binary, run: sudo bash $(dirname "$0")/../foundry/uninstall.sh"
