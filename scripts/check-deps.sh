#!/usr/bin/env bash
# Verifikasi toolchain & dependency rakuos-shell-picker.
#
# Jalankan sebelum mulai coding:
#   ./scripts/check-deps.sh
#
# Keluar dengan kode != 0 kalau ada yang wajib hilang.

set -uo pipefail

cd "$(dirname "$0")/.." || exit 2

pass=0
warn=0
fail=0

ok()   { printf '  \033[32mOK\033[0m    %s\n' "$1"; pass=$((pass+1)); }
warn() { printf '  \033[33mWARN\033[0m  %s\n' "$1"; warn=$((warn+1)); }
bad()  { printf '  \033[31mFAIL\033[0m  %s\n' "$1"; fail=$((fail+1)); }

have() { command -v "$1" >/dev/null 2>&1; }

echo "== toolchain rust =="
for tool in rustc cargo rustfmt cargo-clippy; do
  if have "$tool"; then
    ok "$tool — $("$tool" --version 2>&1 | head -1)"
  else
    bad "$tool tidak ada di PATH"
  fi
done

echo
echo "== linker =="
# Wajib: `.cargo/config.toml` memakai linker clang dan `-fuse-ld=lld`.
for tool in clang ld.lld; do
  if have "$tool"; then
    ok "$tool — $("$tool" --version 2>&1 | head -1)"
  else
    bad "$tool tidak ada — pasang clang / lld (dipakai .cargo/config.toml)"
  fi
done

echo
echo "== library development (pkg-config) =="
for mod in gtk4 libadwaita-1 glib-2.0 gio-2.0 gobject-2.0 cairo pango; do
  if ver=$(pkg-config --modversion "$mod" 2>/dev/null); then
    ok "$mod $ver"
  else
    bad "$mod tidak ditemukan — pasang gtk4-devel / libadwaita-devel"
  fi
done

echo
echo "== library runtime =="
for lib in libgtk-4.so.1 libadwaita-1.so.0 librsvg-2.so.2; do
  if ldconfig -p 2>/dev/null | grep -q "$lib"; then
    ok "$lib"
  else
    bad "$lib tidak ada"
  fi
done

echo
echo "== tool runtime =="
for tool in rpm git; do
  if have "$tool"; then
    ok "$tool"
  else
    bad "$tool tidak ada"
  fi
done

echo
echo "== konsistensi rpmdb =="
# Diketahui: paket diinstal via dnf, tapi rpmdb tidak lagi mencatatnya.
# File ada = aman untuk sekarang. rpmdb kosong = hilang saat ganti image.
orphan=()
for pkg in rust cargo rustfmt clippy clang lld gtk4-devel libadwaita-devel; do
  file=""
  case "$pkg" in
    rust)    file=/usr/bin/rustc      ;;
    cargo)   file=/usr/bin/cargo      ;;
    rustfmt) file=/usr/bin/rustfmt    ;;
    clippy)  file=/usr/bin/cargo-clippy ;;
    clang)   file=/usr/bin/clang      ;;
    lld)     file=/usr/bin/ld.lld     ;;
    gtk4-devel)         file=/usr/lib64/pkgconfig/gtk4.pc         ;;
    libadwaita-devel)  file=/usr/lib64/pkgconfig/libadwaita-1.pc ;;
  esac
  if ! rpm -q "$pkg" >/dev/null 2>&1 && [ -f "$file" ]; then
    orphan+=("$pkg")
  fi
done

if [ ${#orphan[@]} -eq 0 ]; then
  ok "semua paket tercatat di rpmdb"
else
  warn "terpasang tapi tidak tercatat rpmdb: ${orphan[*]}"
  echo "         file masih ada, build aman. tapi image baru bisa kehilangan"
  echo "         toolchain ini. pastikan Containerfile menyebutkannya."
fi

echo
echo "== aset & konfigurasi =="
missing_assets=0
for svg in noctalia danklogo2 caelestia ambxst-logo-color; do
  [ -f "assets/$svg.svg" ] || { bad "assets/$svg.svg hilang"; missing_assets=1; }
done
[ "$missing_assets" -eq 0 ] && ok "4 logo SVG lengkap"

RULE_FILE="${XDG_CONFIG_HOME:-$HOME/.config}/hypr/config/windowrules.lua"
if [ -f "$RULE_FILE" ] && grep -qF 'ShellPicker' "$RULE_FILE"; then
  ok "rule window ShellPicker ada di windowrules.lua"
else
  warn "rule window ShellPicker tidak ditemukan — picker akan ikut di-tile"
  echo "         tambahkan:"
  echo '           hl.window_rule({ match = { class = "^(org\\.rakuos\\.ShellPicker)$" },'
  echo '                               float = true, size = { "806", "806" } })'
fi

echo
echo "== build & test =="
if have cargo; then
  work=$(mktemp -d "${TMPDIR:-/tmp}/rakuos-shell-picker-check.XXXXXX") || work=""
  build_err="$work/build.err"
  test_out="$work/test.out"

  if cargo build --quiet 2>"$build_err"; then
    ok "cargo build"
  else
    bad "cargo build gagal:"
    sed 's/^/         /' "$build_err" | head -15
  fi

  if cargo test --quiet >"$test_out" 2>&1; then
    ok "cargo test — $(grep -oE '[0-9]+ passed' "$test_out" | head -1)"
  else
    bad "cargo test gagal:"
    sed 's/^/         /' "$test_out" | grep -E 'error|FAILED|panicked' | head -10
  fi

  n=$(cargo clippy --all-targets 2>&1 | grep -cE '^(error|warning:)')
  if [ "$n" -eq 0 ]; then
    ok "cargo clippy bersih"
  else
    warn "cargo clippy: $n temuan"
  fi
  [ -n "$work" ] && rm -rf "$work"
fi

echo
echo "==============================="
printf 'lulus %d   peringatan %d   gagal %d\n' "$pass" "$warn" "$fail"
[ "$fail" -eq 0 ] || exit 1
