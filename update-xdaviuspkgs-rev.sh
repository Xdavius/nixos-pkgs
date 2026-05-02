#!/usr/bin/env bash
set -euo pipefail

REPO="https://github.com/Xdavius/nixos-pkgs"
STATE_FILE="./xdaviuspkgs-rev.nix"

echo "🔄 Fetch latest commit..."
REV=$(git ls-remote "$REPO" HEAD | awk '{print $1}')

echo "📌 $REV"

sudo tee "$STATE_FILE" >/dev/null <<EOF
{
  rev = "$REV";
}
EOF

echo "✅ Updated in /etc/nixos"
