# rakuos-shell-picker

Pilih **desktop shell** (panel/widgets/status layer) yang berjalan di atas
sebuah compositor. Compositor sendiri — **Hyprland atau Niri** — **tidak**
berganti: selama window picker terbuka, compositor yang sedang jalan tetap
jalan.

```
Hyprland / Niri        <- compositor, selalu hidup, tidak pernah di-restart
   └── shell           <- yang dipilih user
```

Shell yang tersedia berbeda per compositor:

| Compositor | Shell | Jumlah |
|---|---|---|
| Hyprland | Noctalia, DMS, Caelestia, Ambxst, end4-pC | 5 |
| Niri | DMS, Noctalia, Ambxst, iNiR | 4 |

DMS, Noctalia, dan Ambxst tersedia di kedua compositor. **Caelestia** dan
**end4-pC** hanya ada di Hyprland, sementara **iNiR** hanya ada di Niri.

## Status

Ready to use. UI dan switch backend sudah terimplementasi, aman dengan backup+rollback. Lihat [docs/FEATURES.md](docs/FEATURES.md) untuk daftar
lengkap fitur beserta statusnya.

## Build & run

```bash
# jalankan dari root repo
cargo run -p rakuos-shell-picker   # atau: cargo run
```

Butuh GTK 4.22 dan libadwaita 1.9 — logo dibaca langsung sebagai SVG, tidak ada
aset PNG yang perlu dikonversi.

```bash
cargo test      # registry + session state
cargo clippy --all-targets
```

## Layout project

```
src/main.rs      entry point
src/app.rs       aplikasi GTK & window
src/ui/          widget, satu file per bagian
src/compositor/  satu folder per compositor
  hyprland/shells/   satu folder per shell, isinya skel/ dan Port-nya
  niri/shells/       satu folder per shell (dms, noctalia, ambxst, inir), isinya skel/ dan Port-nya
src/shell.rs     tipe Shell & cara memeriksa satu shell
src/session.rs   state active/previous
assets/          logo SVG shell
data/            ikon aplikasi & desktop entry
docs/            spesifikasi & rencana
```

Aturan singkat: **menambah shell tidak boleh menyentuh `ui/`**. Detailnya di
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Dokumentasi

| Dokumen | Isi |
|---|---|
| [docs/README.md](docs/README.md) | indeks dokumen |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | struktur modul & aturan dependensi |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | toolchain, dependency, pemeriksaan environment |
| [docs/FEATURES.md](docs/FEATURES.md) | fitur, status, syarat selesai |
| [docs/LOGGING.md](docs/LOGGING.md) | tiga saluran log, lokasi, format, rotasi |
| [docs/SAFETY.md](docs/SAFETY.md) | jaminan backup & rollback |
| [docs/SWITCH-FLOW.md](docs/SWITCH-FLOW.md) | urutan langkah switch Noctalia ⇄ DMS |
| [docs/UI.md](docs/UI.md) | spesifikasi layout & angka tetap |

Sebelum mulai coding, cek environment:

```bash
./scripts/check-deps.sh
```

## Catatan integrasi

- App ID: `org.rakuos.ShellPicker`
- Binary: `rakuos-shell-picker`
- Window harus float + ukuran tetap agar tidak ikut tile: rule Hyprland ada di
  `~/.config/hypr/config/windowrules.lua`.
- State: `~/.local/state/rakuos/shell-picker/state.toml`

## Alur kerja

1. Mendeteksi compositor yang sedang jalan (Hyprland atau Niri). Kalau tidak dikenali, semua klik ditolak.
2. Menampilkan daftar shell yang tersedia untuk compositor tersebut.
3. Saat user memilih shell: backup konfigurasi yang relevan dengan shell, stop shell lama, terapkan konfigurasi target (dari skel/backup logic), start shell baru.
4. Verifikasi: bus notifikasi berpindah ke shell baru.
5. Logout sesi (default) atau biarkan live (--no-logout/opsi UI) — logout memastikan sesi fresh.

Semua perubahan hanya menyentuh konfigurasi yang berkaitan dengan shell (lihat [docs/SAFETY.md](docs/SAFETY.md)). Compositor tidak pernah di-restart.
