#!/usr/bin/env bash
set -euo pipefail

REPO="proto-cli/proto-cli"
RELEASES_API="https://api.github.com/repos/${REPO}/releases"
DOWNLOAD_BASE="https://github.com/${REPO}/releases/download"
BIN_NAME="proto"

BOLD="\033[1m"
CYAN="\033[0;36m"
BLUE="\033[0;34m"
WHITE="\033[0;37m"
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
RED="\033[0;31m"
NC="\033[0m"
DIV="──────────────────────────────────────────"

PREFIX="${PREFIX:-$HOME/.local/bin}"
VERSION=""
DRY_RUN=0
PRINT_TARGET=0
ASSUME_YES=0

usage() {
    cat <<EOF
Proto CLI installer

Usage:
  curl -fsSL https://raw.githubusercontent.com/${REPO}/master/install.sh | sh
  curl -fsSL https://raw.githubusercontent.com/${REPO}/master/install.sh | sh -s -- --prefix /usr/local/bin

Options:
  --prefix DIR       Install directory (default: \$HOME/.local/bin)
  --version TAG      Install a specific release tag (default: latest)
  --print-target     Print the detected target triple and exit
  --dry-run          Download and verify but do not install
  -y, --yes          Skip confirmation prompts
  -h, --help         Show this help
EOF
}

info()    { echo -e "${CYAN}  ◆${NC} $1"; }
success() { echo -e "${GREEN}  ✔${NC} $1"; }
warn()    { echo -e "${YELLOW}  ⚠${NC}  $1"; }
err()     { echo -e "${RED}  ✗${NC} $1"; }
sep()     { echo -e "\n  ${BLUE}${DIV}${NC}\n"; }

detect_target() {
    local os arch target
    case "$(uname -s)" in
        Linux)  os="linux" ;;
        Darwin) os="darwin" ;;
        MSYS*|MINGW*|CYGWIN*) os="windows" ;;
        *) os="$(uname -s | tr '[:upper:]' '[:lower:]')" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) arch="$(uname -m)" ;;
    esac

    case "${os}-${arch}" in
        linux-x86_64)  target="x86_64-unknown-linux-gnu" ;;
        linux-aarch64) target="aarch64-unknown-linux-gnu" ;;
        darwin-x86_64) target="x86_64-apple-darwin" ;;
        darwin-aarch64) target="aarch64-apple-darwin" ;;
        windows-x86_64) target="x86_64-pc-windows-msvc.exe" ;;
        windows-aarch64) target="aarch64-pc-windows-msvc.exe" ;;
        *)
            err "Unsupported platform: ${os}-${arch}"
            err "No prebuilt binary is available for your system."
            exit 1
            ;;
    esac

    echo "$target"
}

resolve_version() {
    if [[ -n "$VERSION" ]]; then
        echo "$VERSION"
        return
    fi

    local tags
    tags="$(curl -fsSL "$RELEASES_API" 2>/dev/null \
        | grep -o '"tag_name": *"[^"]*"' \
        | sed 's/.*"\(.*\)"/\1/' \
        | sort -V \
        | tail -n 1 || true)"

    if [[ -z "$tags" ]]; then
        err "Could not determine the latest release version."
        err "Check your network connection, or pass one explicitly with --version."
        exit 1
    fi
    echo "$tags"
}

sha256_of() {
    if command -v sha256sum &>/dev/null; then
        sha256sum "$1" | awk '{print $1}'
    elif command -v shasum &>/dev/null; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        err "No sha256 utility found (needs sha256sum or shasum)."
        exit 1
    fi
}

install_binary() {
    local target="$1" version="$2"
    local asset="proto-${target}"
    local url="${DOWNLOAD_BASE}/${version}/${asset}"

    info "Downloading ${asset} (${version})..."
    TMP_DIR="$(mktemp -d)"
    trap 'rm -rf "$TMP_DIR"' EXIT

    local binary="${TMP_DIR}/${BIN_NAME}"
    if ! curl -fsSL "$url" -o "$binary"; then
        err "Failed to download ${url}"
        err "The binary for your platform may not be published for this release yet."
        exit 1
    fi

    local expected actual
    expected="$(curl -fsSL "${url}.sha256" 2>/dev/null | awk '{print $1}' | tr '[:upper:]' '[:lower:]' || true)"
    actual="$(sha256_of "$binary")"
    if [[ -n "$expected" ]]; then
        if [[ "$expected" != "$actual" ]]; then
            err "Checksum verification failed."
            err "  expected: ${expected}"
            err "  actual:   ${actual}"
            err "Installation aborted for security reasons."
            exit 1
        fi
        success "Checksum verified"
    else
        warn "No published checksum found; skipping verification."
    fi

    chmod +x "$binary"

    if [[ "$DRY_RUN" -eq 1 ]]; then
        success "Dry-run: verified binary ready at ${binary}"
        info "Would install to ${PREFIX}/${BIN_NAME}"
        exit 0
    fi

    mkdir -p "$PREFIX"
    if [[ -w "$PREFIX" ]]; then
        cp "$binary" "$PREFIX/$BIN_NAME"
    elif command -v sudo &>/dev/null && [[ "$(id -u)" -ne 0 ]]; then
        warn "Installing to ${PREFIX} requires elevated permissions."
        sudo cp "$binary" "$PREFIX/$BIN_NAME"
    else
        err "Cannot write to ${PREFIX}"
        err "Re-run with: sh -s -- --prefix \$(pwd)/bin"
        exit 1
    fi

    success "Installed: ${PREFIX}/${BIN_NAME}"

    if [[ "$PREFIX" != "/usr/local/bin" ]] && ! echo ":$PATH:" | grep -q ":${PREFIX}:"; then
        warn "${PREFIX} is not in your PATH."
        echo "  Add it to your shell config, e.g.:"
        echo -e "    ${CYAN}export PATH=\"\$PATH:${PREFIX}\"${NC}"
    fi
}

main() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --prefix)
                PREFIX="$2"; shift 2 ;;
            --prefix=*)
                PREFIX="${1#*=}"; shift ;;
            --version)
                VERSION="$2"; shift 2 ;;
            --version=*)
                VERSION="${1#*=}"; shift ;;
            --print-target)
                PRINT_TARGET=1; shift ;;
            --dry-run)
                DRY_RUN=1; shift ;;
            -y|--yes)
                ASSUME_YES=1; shift ;;
            -h|--help)
                usage; exit 0 ;;
            *)
                err "Unknown option: $1"
                usage; exit 1 ;;
        esac
    done

    if [[ "$PRINT_TARGET" -eq 1 ]]; then
        echo "$(detect_target)"
        exit 0
    fi

    echo ""
    echo -e "${CYAN}    ⣀⡀    ${NC}"
    echo -e "${CYAN}⢠⣤⡀⣾⣿⣿⠀⣤⣤⡄${NC}"
    echo -e "${CYAN}⢿⣿⡇⠘⠛⠁⢸⣿⣿⠃${NC}"
    echo -e "${CYAN}⠈⣉⣤⣾⣿⣿⡆⠉⣴⣶⣶${NC}"
    echo -e "${CYAN}⣾⣿⣿⣿⣿⣿⣿⡀⠻⠟⠃${NC}"
    echo -e "${CYAN}⠙⠛⠻⢿⣿⣿⣿⡇  ${NC}"
    echo -e "${CYAN}    ⠈⠙⠋⠁  ${NC}"
    echo ""
    echo -e "${BOLD}${CYAN}Proto CLI ${WHITE}installer${NC}"
    echo -e "${BLUE}Your friendly protogen CLI companion${NC}"

    sep

    local target
    target="$(detect_target)"
    local version
    version="$(resolve_version)"
    info "Latest release: ${version}"
    info "Target: ${target}"

    if [[ "$ASSUME_YES" -eq 0 ]] && [[ ! -t 0 ]]; then
        info "Piping? Installing automatically (add --yes to silence)."
    fi

    sep

    install_binary "$target" "$version"

    sep
    echo -e "  ${GREEN}✦ Proto CLI installed successfully ✦${NC}"
    echo ""
    echo "        proto help             Show all commands"
    echo "        proto system           View system information"
    echo "        proto completions bash --install   Shell completions"
    echo ""
    echo "  Reinstall / update anytime:"
    echo -e "    ${CYAN}curl -fsSL https://raw.githubusercontent.com/${REPO}/master/install.sh | sh${NC}"
    echo ""
}

main "$@"