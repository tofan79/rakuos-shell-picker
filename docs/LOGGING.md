# Spesifikasi Logging

Tiga saluran log dengan aturan berbeda. Memilih kanal mana untuk apa bukan bebas —
satu Channel untuk satu tujuan.

| # | Saluran | Tujuan | Umur | Lihat user? |
|---|---|---|---|---|
| 1 | **Log aplikasi** | rekaman operasional selama app jalan | dibuang saat logout | tidak, hanya developer |
| 2 | **Panel UI** | progres saat ada operasi | hanya saat ada operasi | ya |
| 3 | **Log gagal** | detail kegagalan yang bisa dibaca user | permanen, rotasi 10 | ya, lewat file |

Aturan dasar: **yang sukses tidak ditulis sebagai log.** Yang berhasil tidak
perlu bukti; yang gagal harus bisa dibaca dan ditindaklanjuti.

## Satu sistem log, bukan dua

Semua log keluar lewat satu pintu: `src/logging.rs`, lewat
`logging::info` / `logging::warn` / `logging::error`. Tidak ada crate logging
lain di `Cargo.toml`.

`log` dan `env_logger` pernah ada di sana dan sudah dibuang, karena keduanya
melanggar aturan di halaman ini: `env_logger` menulis ke stderr, bukan ke file
`0600` di `$XDG_RUNTIME_DIR`, dan ia tidak ikut format RFC 3339 + level di
bawah. Satu kasus nyata dari punyanya: `env_logger::init()` ada di `main()` dan
satu `log::warn!` ada di `ui/logo.rs`, tapi filter defaultnya `error` — jadi
baris `warn` itu tidak pernah muncul, dan tidak pernah ada yang sadar karena
tidak ada yang membacanya.

Kalau nanti butuh level verbose untuk development, tambahkan levelnya di
`logging.rs` dan filternya di sana juga. Membawa crate kedua berarti log terpecah
di dua tempat dengan dua format.

## 1. Log aplikasi

```
$XDG_RUNTIME_DIR/rakuos-shell-picker/picker.log     mode 0600
$TMPDIR/rakuos-shell-picker-$UID/picker.log          fallback, dir 0700
```

Jangan pakai `/tmp/rakuos-shell-picker.log` polos. `/tmp` mode 1777, file di
 dalamnya bisa dibaca user lain di mesin yang sama, dan isinya berisi nama path
konfigurasi. Kalau `XDG_RUNTIME_DIR` tidak ada, buat subfolder sendiri dengan
nama UID dan mode 0700 — bukan file telanjang di `/tmp`.

Isi:

```
2026-10-02T18:30:45Z INFO  start rakuos-shell-picker 1.0.2
2026-10-02T18:30:45Z INFO  compositor detected: hyprland
2026-10-02T18:30:45Z INFO  inventory: noctalia=installed dms=available
2026-10-02T18:31:02Z INFO  switch begin op=7f3a1c2e from=noctalia to=dms
2026-10-02T18:31:09Z INFO  switch end   op=7f3a1c2e result=ok
```

- **Diotong (truncate) saat window start.** Window memulai file kosong.
- **Jalur `--switch` dan `--dry-run` menambah (append), bukan memotong.** Keduanya
  membuka file yang sama dengan mode append dan menulis penanda
  `--- command line session ---`. Window bisa saja sedang berjalan — bahkan
  sedang switch — dan mengosongkan file pada saat itu akan membuang seluruh
  catatannya, termasuk operasi yang sedang berjalan. Dua sesi dalam satu file
  dipisah oleh penanda di atas.
- **Satu langkah = satu baris.** `progress` sudah menulis ke log lewat panel UI
  dan lewat closure CLI; memanggil `logging::info` lagi untuk langkah yang sama
  akan mencetaknya dua kali. Kalau sebuah baris muncul dobel di log, penyebabnya
  hampir selalu dua jalur itu.
- **Penolakan ikut tercatat.** `refused:` dari `--dry-run`/`--switch` ditulis ke
  log sebagai `ERROR`, jadi percobaan yang berhenti sebelum langkah pertama pun
  meninggalkan jejak.
- Format: RFC 3339 UTC + level + pesan.
- Setiap baris operasi diberi `op=<id>` supaya bisa dikorelasikan dengan log
  gagal dan panel UI.

## 2. Panel log di UI

Tidak pernah menampilkan inventaris atau versi aplikasi. Itu urusan log aplikasi.

Cuma ada kalau sedang ada operasi:

| Keadaan | Panel |
|---|---|
| Idle | **tersembunyi** |
| Operasi berjalan | tampil, isi progres per langkah |
| Operasi sukses | **tersembunyi lagi** |
| Operasi gagal | **tetap tampil** sampai user menutupnya, + badge di card yang gagal |

Alasan gagal tidak boleh hilang sendiri: kalau panel hilang begitu selesai,
user tidak pernah tahu switch-nya tidak berhasil. Karena itu ada tombol
tutup, dan badge di card supaya kelihatan dari mana pun user sedang melihat.

Panel menampilkan ringkas saja (langkah demi langkah). Detail lengkap tetap di
log aplikasi.

## 3. Log gagal

> **Status: belum diimplementasikan.** Saluran yang berjalan sekarang baru 1 dan
> 2. Bagian ini adalah spesifikasi tujuannya, dan tiap barisnya tercatat sebagai
> G3–G8 di [FEATURES.md](FEATURES.md).

```
~/.local/state/rakuos-shell-picker/logs/<shell-id>/<timestamp>.log   mode 0600
```

Contoh:

```
~/.local/state/rakuos-shell-picker/logs/dms/20261002T183102Z.log
```

- Timestamp format `YYYYMMDDTHHMMSSZ` (aman untuk nama file).
- Mode `0600` — isi log bisa menyebut path konfigurasi, dan konfigurasi shell
  kadang memuat token.
- **Hanya ditulis saat gagal.** Switch sukses tidak pernah membuat file.
- Rotasi: simpan 10 kegagalan terakhir per shell, lalu pangkas yang tertua.

Rotasi log boleh otomatis karena log itu artefak yang bisa digenerate ulang.
Ini **berbeda** dari backup konfigurasi: penghapusan backup selalu butuh
persetujuan user, lihat [SAFETY.md](SAFETY.md).

Isi log gagal:

```toml
operation_id = "7f3a1c2e"
shell = "dms"
compositor = "hyprland"
started_at = "2026-10-02T18:31:02Z"
finished_at = "2026-10-02T18:31:04Z"
result = "failed"
stage = "start"          # preflight | backup | stop | start | verify | state
error = "exit status 1"
```

Isi file `.log`: output lengkap perintah yang gagal, stderr, dan jejak
langkah. Timestamp penanda file.log dibungkus di `.toml` terpisah supaya file
log tetap murni teks.

## Yang tidak boleh masuk log

- Isi file konfigurasi
- Nilai token, password, API key
- Seluruh isi environment
- Lokasi home lengkap tanpa keperluan

Yang boleh: nama path, nama executable, exit code, pesan error dari perkakas,
timestamp, id operasi.

## Jenis kegagalan yang dicatat

| Jenis | Kirim log gagal? | Alasan |
|---|---|---|
| Guard ditolak (compositor beda) | tidak | bukan kegagalan, memang dicegah sejak awal |
| Shell target belum terpasang | tidak | hanya ditolak user, bukan error sistem |
| Backup gagal | **ya** | data berisiko, user harus tahu |
| Stop gagal | **ya** | bisa menyisakan sesi setengah |
| Start gagal | **ya** | ini yang paling sering terjadi |
| Verifikasi gagal | **ya** | shell hidup tapi tidak berguna |
| Tulis state gagal | **ya** | state jadi tidak konsisten |

Guard dan "belum terpasang" cukup ditulis ke log aplikasi, karena user sudah
melihat langsung lewat badge di card-nya.

## Contoh isi log gagal

```
=== rakuos-shell-picker 1.0.2 ===
operation : 7f3a1c2e
shell     : dms
compositor: hyprland
started   : 2026-10-02T18:31:02Z

[18:31:02] preflight  ok
[18:31:02] backup     ok -> ~/.local/share/backups/rakuos-shell-picker/20261002-hyprland-noctalia
[18:31:03] stop       ok (pid 4821, noctalia)
[18:31:03] start      FAILED
  cmd: dms run
  exit: 1
  stderr: Failed to load config: /home/user/.config/dms/config.json
[18:31:04] result     failed at stage=start
[18:31:04] rollback   ok, noctalia restarted (pid 4913)
```
