#!/usr/bin/env bash
set -euo pipefail

VERSION="0.2.0"
REPO="dzmbs/crosswind"
INSTALL_DIR="${CROSSWIND_INSTALL_DIR:-$HOME/.local/bin}"

say() {
  printf '[crosswind] %s\n' "$*"
}

err() {
  printf '[crosswind] ERROR: %s\n' "$*" >&2
  exit 1
}

detect_target() {
  local os arch

  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)  os="linux" ;;
    Darwin) os="macos" ;;
    *)      err "Unsupported operating system: $os" ;;
  esac

  case "$arch" in
    x86_64)          arch="x86_64" ;;
    arm64|aarch64)   arch="aarch64" ;;
    *)               err "Unsupported architecture: $arch" ;;
  esac

  echo "${os}-${arch}"
}

check_existing() {
  local existing
  if existing="$(command -v crosswind 2>/dev/null)"; then
    local current
    current="$("$existing" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "")"
    if [ "$current" = "$VERSION" ]; then
      say "Crosswind v${VERSION} is already installed at ${existing}"
      say "Nothing to do."
      exit 0
    elif [ -n "$current" ]; then
      say "Upgrading Crosswind v${current} to v${VERSION}"
    fi
  fi
}

download() {
  local target="$1"
  local artifact="crosswind-v${VERSION}-${target}.tar.gz"
  local url="https://github.com/${REPO}/releases/download/v${VERSION}/${artifact}"
  local tmpdir

  say "Downloading Crosswind v${VERSION} for ${target}..."

  tmpdir="$(mktemp -d)"
  trap "rm -rf '$tmpdir'" EXIT

  if ! curl -fsSL "$url" -o "${tmpdir}/${artifact}"; then
    err "Download failed. No prebuilt binary for ${target}.

To install from source:
  1. Install Rust: https://rustup.rs
  2. Run: cargo install --git https://github.com/${REPO} --bin crosswind"
  fi

  say "Extracting..."
  tar xzf "${tmpdir}/${artifact}" -C "$tmpdir"

  mkdir -p "$INSTALL_DIR"

  if [ -f "${tmpdir}/crosswind" ]; then
    mv "${tmpdir}/crosswind" "${INSTALL_DIR}/crosswind"
  elif [ -f "${tmpdir}/crosswind-v${VERSION}-${target}/crosswind" ]; then
    mv "${tmpdir}/crosswind-v${VERSION}-${target}/crosswind" "${INSTALL_DIR}/crosswind"
  else
    err "Could not find crosswind binary in archive"
  fi

  chmod +x "${INSTALL_DIR}/crosswind"
}

check_path() {
  case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) return 0 ;;
  esac

  say ""
  say "Add crosswind to your PATH:"
  say ""
  say "  export PATH=\"${INSTALL_DIR}:\$PATH\""
  say ""
}

main() {
  say "Crosswind installer"
  say ""

  check_existing

  local target
  target="$(detect_target)"
  download "$target"

  say "Installed crosswind to ${INSTALL_DIR}/crosswind"

  if "${INSTALL_DIR}/crosswind" --version 2>/dev/null; then
    true
  fi

  check_path
  say "Run 'crosswind --help' to get started."
}

main
