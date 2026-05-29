#!/usr/bin/env bash
set -euo pipefail

echo "==> Installing system dependencies..."
sudo apt-get update -qq && sudo apt-get install -y --no-install-recommends \
  pkg-config \
  libssl-dev \
  sqlite3

echo "==> Installing sqlx-cli (offline, sqlite only)..."
cargo install sqlx-cli --no-default-features --features sqlite --locked

echo "==> Installing frontend dependencies..."
cd /workspace/web
bun install

echo "==> Creating data directory for SQLite..."
mkdir -p /workspace/data

echo "==> Setting up database..."
cd /workspace/api
cargo sqlx database setup

echo ""
echo "✅ Devcontainer ready!"
echo "   Frontend:  cd web  && bun dev"
echo "   Backend:   cd api  && cargo run"
echo "   (oder:     Launch 'API + Web' in VS Code)"