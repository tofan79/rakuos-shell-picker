# Fitur

Status: **selesai** = jalan & teruji · **sebagian** = UI ada, fungsional belum ·
**belum** = belum dikerjakan · **keluar scope** = sengaja tidak.

## A. Fondasi

| # | Fitur | Status |
|---|---|---|
| A1 | Binary `rakuos-shell-picker`, app ID `org.rakuos.ShellPicker` | selesai |
| A2 | Satu crate (`lib` + `bin`) dengan modul per compositor | selesai |
| A3 | Struktur modular, `ui/` terisolasi dari backend | selesai |
| A4 | 206 unit test (registry, logo, urutan, state, guard, dialog, logging) | selesai |
| A5 | Clippy bersih, `cargo test` hijau | selesai |

## B. Tampilan

| # | Fitur | Status |
|---|---|---|
| B1 | Window persegi 806×806, float, rule Hyprland | selesai |
| B2 | Tema gelap, panel semi-transparan, outline 1px | selesai |
| B3 | Tab compositor dari registry, tab awal = compositor yang jalan | selesai (Hyprland + Niri) |
| B4 | Grid 4 kolom, lebar kartu dipatok 150px | selesai |
| B5 | Grid scroll vertikal, lebar terkunci (aman 20+ shell) | selesai |
| B6 | Logo SVG langsung via GdkTexture, tanpa PNG | selesai |
| B7 | Logo & label center, nama panjang di-ellipsize | selesai |
| B8 | Panel log 100px di bawah, monospace, scroll sendiri | selesai |
| B13 | Status `running` diberi warna hijau | selesai |
| B14 | Label aksi berbunyi "switch", bukan "install" | selesai |
| B15 | Panel log hanya tampil saat ada operasi, hidden saat idle | selesai |
| B16 | Panel log gagal tetap tampil sampai ditutup | selesai (badge di card belum) |
| B17 | Halaman compositor yang tidak sedang jalan menampilkan banner + penolakan saat diklik | selesai (tab Niri kini nyata; halaman non-aktif tetap menolak) |
| B9 | Placeholder "Coming soon" untuk compositor tanpa shell | selesai |
| B10 | Kartu `unavailable` untuk shell yang belum terpasang | selesai |
| B11 | Dialog konfirmasi Yes/No sebelum apply | selesai |
| B12 | Tombol Refresh / Install | belum |
| B18 | Kanal log terpisah: file aplikasi + panel UI (bukan satu panel untuk semua) | selesai |

## C. Deteksi

| # | Fitur | Status |
|---|---|---|
| C1 | Binary ada di `PATH` | selesai |
| C2 | Paket terpasang (`rum paths` + `rpm --dbpath` overlay, bukan `rpm -q`) | selesai |
| C3 | Proses sedang jalan (scan `/proc/*/comm`) | selesai |
| C4 | Status `running` / `switch` / `not installed` di kartu | selesai |
| C5 | Health probe per shell (socket/API) | belum |
| C6 | Deteksi compositor aktif dari envvar (`HYPRLAND_INSTANCE_SIGNATURE`, `NIRI_SOCKET`) | selesai |
| C7 | Deteksi sesi Wayland (`WAYLAND_DISPLAY`, `XDG_RUNTIME_DIR`) | selesai |
| C8 | Fallback deteksi lewat `/proc/*/exe` | selesai |
| C9 | Tolak semua aksi kalau compositor tidak terdeteksi | selesai |

## D. State

| # | Fitur | Status |
|---|---|---|
| D1 | Simpan `active` / `previous` di XDG state | selesai |
| D2 | `record_switch`, lookup `active_shell` | selesai |
| D3 | Backups dir disiapkan | selesai |
| D4 | Tampilkan shell aktif dari state di UI | belum |
| D5 | Tulis state secara atomik (temp file + rename) | selesai — `State::save_to`, tulis ke `state.toml.tmp` lalu `rename` |
| D6 | Simpan compositor aktif, bukan cuma shell | belum |

## E. Backend

Inti dari proyek. `switch::prepare` (langkah 1–3) dan `apply::apply_with`
(langkah sisanya) adalah dua bagiannya; urutan persisnya ada di
[SAFETY.md](SAFETY.md).

| # | Fitur | Status |
|---|---|---|
| E1 | Trait `ShellBackend` (start/stop/status/backup) | dibatalkan — diganti `Identity` + `Port` + `StartMechanism`, jadi fakta per shell dideklarasikan bukan diimplementasikan |
| E2 | Backup konfigurasi sebelum switch | selesai — `switch::backup_sources` + `backup::create`, dibaca ulang dan dicocokkan dengan manifest |
| E3 | Rollback kalau switch gagal | selesai — `Apply::Failed { rolled_back }` |
| E4 | Backend Noctalia | selesai (registry, `StartMechanism::Autostart`) |
| E5 | Backend DMS | selesai (registry, `StartMechanism::SystemdUserUnit`) |
| E6 | Backend Caelestia | selesai — paket + install/uninstall plan + skel + probe proses `qs -c caelestia` |
| E7 | Backend Ambxst | selesai — paket + install/uninstall plan + skel + probe proses `ambxst`; runtime bersama DMS (`quickshell-git`, `matugen`) ditahan lewat `runtime_packages` |
| E8 | Stop shell lama dengan rapi (bukan `kill -9`) | selesai — SIGTERM, tunggu, batas waktu, `kill -KILL` hanya upaya terakhir |
| E9 | Start shell baru dengan mekanisme yang benar | selesai — `StartMechanism`, dibaca lagi oleh `verify` |
| E10 | Deteksi shell yang jalan tanpa lewat daftar kita | selesai — `process_running` lewat `/proc` |
| E11 | Registry shell untuk compositor kedua | selesai — Niri terdaftar dengan port DMS + Noctalia + Ambxst + iNiR yang lengkap (skel, config, probe) di `src/compositor/niri/shells/` |
| E12 | Cek environment sebelum switch (display, Wayland) | selesai — deteksi menolak kalau tidak ada sesi Wayland, `prepare` langkah 2 menolak target yang tidak bisa jalan |
| E13 | Guard di backend | selesai — `Refusal::Compositor` di `switch::prepare` langkah 1, arahnya sama dengan guard UI |
| E14 | Bersih setelah switch: `.pre-*`, config dan state shell keluar | selesai — `apply::clean`, hanya yang direproduksi backup byte untuk byte |
| E15 | Hapus paket shell yang ditinggalkan | selesai — `switch::uninstall` + `Identity::uninstall_plan`; paket base image tidak pernah dikirim ke package manager, runtime bersama tidak pernah disentuh |
| E16 | Unit systemd daemon shell mengikuti hidup/matinya | selesai — `Identity::autostart_systemd_user_units`; keluar: `stop`+`disable`+`mask`, masuk: `unmask`+`enable`+`start`; DMS mendeklarasikan `dsearch`, perlakuannya sama di kedua compositor |
| E17 | Kartu iNiR (Niri) + end4-pC (Hyprland) | selesai — end4-pC (plan, config_dir symlink ke payload RPM, skel dots ii + `qsConfig=end4-pC`, bootstrap venv + default tampilan di `custom/`, clean: settings shell + matugen + color-schemes + cache uv + venv); iNiR: plan install/uninstall, skel `defaults/niri` upstream (2 adaptasi: platform theme `qt6ct`, path watcher clipboard), config `.config/illogical-impulse`, state `.local/state/inir`, probe `qs -n -p /usr/share/quickshell/inir`, enable lewat `niri.service.wants` |
| E18 | Enable unit shell yang tidak punya `[Install]` | selesai — `Port::systemd_user_unit_wanted_by`; masuk: symlink `<unit>.service` ke `<target>.wants/` + `daemon-reload`, keluar: semua link itu dihapus (tanpa `disable`/`mask`, karena `disable` justru menolak unit tanpa `[Install]`); iNiR dengan `niri.service` satu-satunya pemakai |

## G. Logging

Spesifikasi lengkap di [LOGGING.md](LOGGING.md).

| # | Fitur | Status |
|---|---|---|
| G1 | Log aplikasi ke `XDG_RUNTIME_DIR`, 0600, dipotong tiap start | selesai — `logging::init`, `picker.log` mode 0600, `truncate` saat window start |
| G2 | Fallback `/tmp/rakuos-shell-picker-$UID/` mode 0700 | selesai — `logging::default_dir`, dir dibuat mode 0700 |
| G3 | Log gagal per waktu di `state/logs/<shell>/<timestamp>.log`, 0600 | belum |
| G4 | Tidak ada log file saat switch sukses | belum |
| G5 | Rotasi 10 kegagalan terakhir per shell | belum |
| G6 | `operation_id` mengorelasikan 3 saluran log | belum |
| G7 | `metadata.toml` per kegagalan | belum |
| G8 | Larangan: jangan pernah tulis isi config atau token ke log | belum |

## Keluar scope

| Item | Alasan |
|---|---|
| Mengganti compositor (Hyprland → compositor lain) | Berarti ganti image; urusan **"RakuOS system management"**. Ditolak oleh guard, lihat [SAFETY.md](SAFETY.md) |
| `rum autoremove` | Runtime yang jadi yatim setelah shell dibersihkan adalah kandidatnya, tapi berjalan tanpa izin shell dan bisa menjatuhkan sesuatu yang dipakai paket lain — pilihan terpisah, bukan efek samping switch |
| Mengedit isi konfigurasi | Picker memilih, bukan editor |
| Window manager-style tiling | Bukan WM, dan WM yang memegang layout |
| Config preset / generator | Berisiko menimpa file user |

## Rencana

Urutan pengerjaan:

1. **Kunci UI.** Setuju dengan tampilan; inisialisasi git; commit. Supaya semua
   perubahan backend nanti punya titik balik yang jelas. *(selesai)*
2. **Deteksi + guard.** `Compositor::is_running()`, `compositor::current()`,
   penolakan cross-compositor di UI **dan** di `preflight()`.
3. **Logging.** Tiga saluran sesuai [LOGGING.md](LOGGING.md), panel on-demand.
4. **Backup + rollback.** Ini duluan, sebelum ada switch sama sekali, supaya
   tidak ada kode yang bisa merusak sebelum ada jaring pengaman. Lihat
   [SAFETY.md](SAFETY.md).
5. **Switch end-to-end.** Stop → start → verifikasi → tulis state. Uji dengan
   DMS karena itu satu-satunya yang terpasang di mesin ini.
6. **Compositor kedua.** *(selesai)* Niri terdaftar dengan skel untuk DMS,
   Noctalia, dan Ambxst. Urutannya tetap: skel dulu, baru registrasi. Jangan
   mendaftarkan compositor yang skel-nya kosong —
   `a_skel_is_missing_only_where_the_shell_cannot_be_installed` akan gagal.

## Syarat "selesai" per fitur

Fitur dianggap selesai kalau:

- [ ] Ada test otomatisnya, atau manual step yang bisa diulang orang lain.
- [ ] Tidak melanggar satu pun aturan di [SAFETY.md](SAFETY.md).
- [ ] `cargo test` dan `cargo clippy --all-targets` hijau.
- [ ] [UI.md](UI.md) atau dokumen ini diperbarui kalau ada perilaku/angka yang
      berubah.
- [ ] Dicatat di sini kalau Status-nya berubah.
