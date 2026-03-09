#!/usr/bin/env bash
set -euo pipefail

VERSION="0.1.0"
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

try_prebuilt() {
  local target="$1"
  local artifact="crosswind-v${VERSION}-${target}.tar.gz"
  local url="https://github.com/${REPO}/releases/download/v${VERSION}/${artifact}"
  local tmpdir

  say "Downloading Crosswind v${VERSION} for ${target}..."

  tmpdir="$(mktemp -d)"
  trap "rm -rf '$tmpdir'" EXIT

  if ! curl -fsSL "$url" -o "${tmpdir}/${artifact}"; then
    return 1
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
  return 0
}

try_cargo_install() {
  if command -v cargo >/dev/null 2>&1; then
    say "Installing via cargo..."
    cargo install --git "https://github.com/${REPO}" --bin crosswind
    return 0
  fi
  return 1
}

try_rustup_then_cargo() {
  if command -v rustup >/dev/null 2>&1; then
    say "Rust toolchain manager found but cargo is missing."
    say "Running: rustup install stable"
    rustup install stable
    if command -v cargo >/dev/null 2>&1; then
      try_cargo_install
      return 0
    fi
  fi
  return 1
}

print_no_install_options() {
  err "Could not install Crosswind.

No prebuilt binary is available for your platform, and neither cargo nor rustup were found.

To install manually:
  1. Install Rust: https://rustup.rs
  2. Run: cargo install --git https://github.com/${REPO} --bin crosswind"
}

check_path() {
  case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) return 0 ;;
  esac

  say ""
  say "Add Crosswind to your PATH by adding this to your shell profile:"
  say ""
  say "  export PATH=\"${INSTALL_DIR}:\$PATH\""
  say ""
}

main() {
  say "Crosswind installer"
  say ""

  local target
  target="$(detect_target)"

  if try_prebuilt "$target"; then
    say "Installed Crosswind to ${INSTALL_DIR}/crosswind"
  elif try_cargo_install; then
    say "Installed Crosswind via cargo"
  elif try_rustup_then_cargo; then
    say "Installed Crosswind via cargo (after rustup)"
  else
    print_no_install_options
  fi

  # Verify
  local cw_bin
  if [ -x "${INSTALL_DIR}/crosswind" ]; then
    cw_bin="${INSTALL_DIR}/crosswind"
  elif command -v crosswind >/dev/null 2>&1; then
    cw_bin="$(command -v crosswind)"
  else
    check_path
    say "Installation complete. Restart your shell or update PATH, then run: crosswind --version"
    exit 0
  fi

  say ""
  if "$cw_bin" --version 2>/dev/null; then
    true
  fi

  check_path
  say ""
  say "Installation complete. Run 'crosswind --help' to get started."
}

main
