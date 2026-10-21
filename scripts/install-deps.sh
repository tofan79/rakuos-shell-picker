#!/usr/bin/env bash
# Pasang toolchain Rust + library dev yang dibutuhkan rakuos-shell-picker.
#
#   ./scripts/install-deps.sh
#
# Di rakuos semua paket datang lewat `rum`, dan `rum` butuh root: di sini
# dipanggil dengan `pkexec`, jadi jalankan sebagai user biasa dan polkit yang
# meminta password sekali. Idempoten — yang sudah ada (dilihat dari efeknya,
# bukan dari rpmdb) akan dilewati.

set -uo pipefail

cd "$(dirname "$0")/.." || exit 2

# Nama paket rum. Deteksinya lewat efek, karena `rpm -q` tidak melihat paket
# overlay (lihat docs/SAFETY.md: "rpm -q tidak bisa dipercaya").
# clang + lld bukan opsional: `.cargo/config.toml` menyetel `linker = "clang"`
# dan `-fuse-ld=lld`, jadi build gagal di tahap link tanpa keduanya.
PKGS=(rust cargo rustfmt clippy clang lld gtk4-devel libadwaita-devel)

have() { command -v "$1" >/dev/null 2>&1; }

is_installed() {
  case "$1" in
    rust)             have rustc ;;
    cargo)            have cargo ;;
    rustfmt)          have rustfmt ;;
    clippy)           have cargo-clippy ;;
    clang)            have clang ;;
    lld)              have ld.lld ;;
    gtk4-devel)       pkg-config --exists gtk4 2>/dev/null ;;
    libadwaita-devel) pkg-config --exists libadwaita-1 2>/dev/null ;;
    *) return 1 ;;
  esac
}

echo "== memeriksa yang sudah ada =="
missing=()
for pkg in "${PKGS[@]}"; do
  if is_installed "$pkg"; then
    printf '  \033[32mOK\033[0m    %s\n' "$pkg"
  else
    printf '  \033[33m----\033[0m  %s belum ada\n' "$pkg"
    missing+=("$pkg")
  fi
done

if [ ${#missing[@]} -eq 0 ]; then
  echo
  echo "Semua dependency sudah ada."
else
  echo
  echo "== memasang: ${missing[*]} =="
  if ! have rum; then
    echo "rum tidak ada di PATH — mesin ini bukan rakuos?" >&2
    exit 1
  fi
  if ! have pkexec; then
    echo "pkexec tidak ada — jalankan script ini sebagai root, atau pasang polkit" >&2
    exit 1
  fi
  pkexec rum install -y "${missing[@]}" || {
    echo "rum install gagal" >&2
    exit 1
  }
fi

echo
echo "== verifikasi akhir =="
exec "$(dirname "$0")/check-deps.sh"
