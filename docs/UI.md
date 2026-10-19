# Spesifikasi Tampilan

Semua angka di sini adalah nilai yang benar-benar dipakai di `src/ui/`.
Kalau kamu ubah angka, ubah juga file yang disebutnya.

## Window

| Properti | Nilai | Sumber |
|---|---|---|
| Rasio | bebas | ukuran diatur compositor (Hyprland float / Niri tiling) |
| Default app | 800×800 | `app.rs` `DEFAULT_SIZE` |
| Isi | menyesuaikan lebar | `FlowBox`, maks 5 kartu per baris, wrap otomatis |
| Scroll | vertikal + horizontal | muncul hanya bila isi lebih besar dari window |

Ukuran window ditentukan compositor (float di Hyprland, tiling di Niri), jadi isi
menyesuaikan lebarnya sendiri dan tidak lagi bergantung pada rule ukuran tetap.

## Susunan

Vertikal, dari atas (jumlah kolom mengikuti lebar window, maks 5):

```
+--------------------------------------------------+
|                                                  |
|              hyprland       Niri                 |   tabbar
|                                                  |
|     +-------+  +-------+  +-------+  +------+   |
|     | logo | | | logo | | | logo | | | logo |   |   grid 4 kolom
|     | 100x | | |      | | |      | | |      |   |
|     +------+ +-------+ +-------+ +-------+   |
|     Nama    Nama      Nama     Nama           |
|                                                  |
|     +------+  +------+  +------+  +------+    |
|     ...  (scroll ke bawah, window tidak melar)  |
|                                                  |
|  +--------------------------------------------+  |
|  | log, 400px, monospace                      |  |
|  +--------------------------------------------+  |
+--------------------------------------------------+
```

Tinggi log tetap 400px; sisa tinggi window dipakai grid. Grid yang grows ke
bawah, bukan window.

## Angka tetap

Semua nilai ini di-pin di Rust, bukan di CSS.

| Konstanta | Nilai | File | Alasan |
|---|---|---|---|
| `MAX_PER_ROW` | 5 | `ui/grid.rs` | batas kartu per baris; kurang bila window sempit |
| `MAX_WIDTH` | 960 | `ui/grid.rs` | lebar maks area kartu (`Clamp`); lebih dari ini kartu tetap di tengah |
| `COLUMN_SPACING` | 20 | `ui/grid.rs` | jarak kolom |
| `ROW_SPACING` | 28 | `ui/grid.rs` | jarak baris |
| `SIDE_MARGIN` | 24 | `ui/grid.rs` | jarak kartu ke tepi window |
| `CARD_WIDTH` | 160 | `ui/card.rs` | kunci lebar kolom |
| `CARD_WIDE` | 320 | `ui/grid.rs` | kartu placeholder |
| `LOG_HEIGHT` | 400 | `ui/log.rs` | tinggi tetap panel: header + ±20 baris monospace, separuh window default |
| `LOGO_BOX` | 100 | `ui/logo.rs` | kotak logo |
| `LOGO_SIZE` | 72 | `ui/logo.rs` | panjang sisi artwork |

## Kenapa lebar kartu di Rust

Semua kolom harus sama lebar supaya `FlowBox` wrap di batas yang bersih pada
ukuran window berapa pun (5 kolom penuh = `5 × 160 + 4 × 20 = 880px`). Kalau lebar
kartu diatur dari CSS atau dari teks nama, kolom bisa tidak sejajar.

Dua pengaman lain:

- Nama dibatasi `max_width_chars(12)` + ellipsize end. Nama shell yang panjang
  tidak akan menambah lebar kolom.
- Scroller memakai policy `Automatic` di dua arah. Grid wrap dulu, jadi scroll
  samping baru muncul bila window lebih sempit dari satu kartu; konten tidak
  pernah terpotong.

Belum diverifikasi ulang setelah pindah ke `FlowBox`; cek manual di window sempit,
lebar, dan 12+ kartu.

## Logo

- SVG dimuat langsung sebagai `gdk::Texture` dari `gio::File`. GTK 4.22 bisa
  merender SVG, jadi tidak ada langkah konversi dan tidak ada PNG fallback.
- Nama file diambil dari `Shell::logo`, dideklarasikan di file shell itu
  sendiri.
- `pixel_size(72)` **dan** `size_request(72, 72)` dua-duanya dipasang.
  `pixel_size` hanya menetapkan batas bawah; tanpa `size_request`, image bisa
  dialokasikan sebesar ukuran natural SVG (mis. 256×256) dan meluber.
  Jangan ganti ke `Picture`: ukuran naturalnya mengikuti SVG (sampai ~500px),
  dan `FlowBox` homogen memakai ukuran natural terbesar, sehingga grid jatuh
  ke satu kolom.
- Kalau SVG gagal dimuat, jatuh ke huruf awal (`N`, `D`, `C`, `A`).

## Kartu shell

Isi: kotak logo 100×100, nama, status.

Status memakai **kata kerja "switch"**, bukan "install" — karena aksi utamanya
memindah shell yang sedang berjalan, bukan memasang paket:

| Keadaan | Label | Warna |
|---|---|---|
| Proses hidup | `running` | **hijau** |
| Terpasang, tidak jalan | `switch` | netral |
| Belum terpasang | `not installed` | diredupkan |

- Shell yang belum terpasang dapat class CSS `unavailable`, dan sub-labelnya
  memakai kata `not installed` — bukan `switch`, karena memang tidak ada yang
  bisa dipindah dan memasang paket berada di luar lingkup picker.
- Shell yang sedang hidup dapat class CSS `running` pada tombol **dan** pada
  sub-label, sehingga teksnya ikut hijau.
- Card milik compositor yang tidak sedang jalan **tetap bisa diklik**, bukan
  non-interaktif. Mengkliknya membuka dialog penolakan, karena card yang dibuat
  mati hanya terlihat rusak.
- Single-selection: memilih satu kartu mematikan yang lain.
- Sorotan hanya menempel kalau hasilnya `Confirm switch`. Kalau ditolak atau
  user menekan Cancel, sorotan dilepas, supaya tidak terlihat seperti ada
  switch yang tertunda.
- Card yang gagal dapat class CSS `failed` sampai operasi berikutnya.

## Dialog safety

`AdwAlertDialog`. Kelima jalur klik, ditentukan oleh fungsi murni
`ui::dialog::decide()` yang sudah ada testnya sendiri:

| Keadaan | Judul | Body | Baris penekanan |
|---|---|---|---|
| Compositor berbeda | `Blocked` | `sway is not the running compositor.` | **hyprland is running.** Switch image in "RakuOS system management". |
| Compositor tidak terdeteksi | `Blocked` | alasannya | Nothing was changed. |
| Shell sudah hidup | `Already running` | `Noctalia is already running.` | Nothing to switch to. |
| Shell belum terpasang | `Not installed` | `Caelestia Shell is not installed.` | Packages come from the **RakuOS** image. |
| Shell terpasang dan idle | `Switch to X?` | `Noctalia is running now.` | Configuration is backed up and never deleted. |

- **Pendek, bukan paragraf.** Awalnya tiap popup dua paragraf; yang penting
  sudah ada di banner kuning di atas dan di log, jadi mengulanginya di sini
  cuma menambah teks kedaluwarsa. Yang tampil satu kalimat + satu baris
  penekanan.
- `AdwAlertDialog` menampilkan body sebagai teks polos, jadi `<b>` di body akan
  tampil sebagai karakter biasa. Karena itu baris yang ditebalkan ditaruh di
  `extra` sebagai `gtk::Label` dengan `use_markup(true)`.
- Nama produk selalu ditulis dalam tanda kutip: **"RakuOS system management"**,
  supaya jelas itu nama fitur, bukan sekadar kata di dalam kalimat.
- Response ditambahkan **sebelum** `present()`. Versi lama menampilkan dialog
  lebih dulu, sehingga tombolnya belum ada saat dialog muncul.
- Dialog penolakan hanya punya `Close`; tidak ada yes/no karena memang tidak
  ada yang bisa disetujui.
- Dialog switch punya `Cancel` sebagai default response, jadi Enter melakukan
  hal yang aman.
- Hanya `Confirm switch` + `Switch` yang boleh lanjut ke backend. Empat jalur
  lain berhenti di sini, dan itu diuji langsung
  (`every_refusal_is_distinct_from_a_confirmation`).
- Eksekusi switch adalah urusan `switch::everything()` (yang memanggil
  `apply::apply()`), bukan dialog. Dialog hanya gerbang konfirmasi.

## Halaman compositor yang diblokir

Compositor yang bukan yang sedang berjalan tetap menampilkan daftar shell-nya
penuh. Menutupinya tidak membantu: yang perlu dilihat user justru apa yang akan
mereka dapat kalau nanti memang berganti image, dan bagaimana bedanya dengan
compositor sekarang.

- **Banner kuning di paling atas**, sebelum grid, dengan alasan penolakannya.
  Banner ini bukan pesan error, melainkan pemberitahuan lebih awal.
- **Kartu tetap tampil dan tetap bisa diklik.** Mengklik salah satu memunculkan
  dialog penolakan, karena itu tujuan akhirnya. Kartu yang dibuat mati hanya
  terlihat rusak.
- Banner juga muncul di halaman yang benar-benar kosong, di atas placeholder.

Dua compositor terdaftar, jadi tiap compositor punya tab sendiri dan tabel ini
membandingkan isinya.

| Shell | hyprland | niri | catatan |
|---|---|---|---|
| Noctalia | ya | ya | di base image Hyprland; dipasang di Niri |
| DankMaterialShell (DMS) | ya | ya | diinstal dari repository (multi-stage) |
| Caelestia Shell | ya | — | diinstal dari repository (quickshell + paket) |
| Ambxst | ya | ya | diinstal dari repository (quickshell + paket + companion) |
| end4-pC | ya | — | diinstal dari repository (quickshell + paket); skel dots `ii` |
| iNiR | — | ya | diinstal dari repository (quickshell + paket); skel `defaults/niri` upstream |

Ambxst ada di repository `rakuos`, jadi ia punya paket, install plan, uninstall
plan, dan `skel/` sendiri (lokasi config-nya `.config/ambxst`). Karena punya
install plan, dialog tidak lagi menolaknya lewat aturan `Unavailable`; ia
diinstal seperti DMS dan Caelestia, dan backup tetap diambil sebelum switch
seperti biasa. Data yang Ambxst dan companion-nya simpan di luar `config_dir`
(`.config/axctl` untuk catatan brightness, `.local/share/ambxst` untuk chat dan
clipboard, `.local/share/ambxst-notes` untuk catatan) ikut tersimpan lewat
`companion_paths`.

iNiR sama: paket `inir-shell` dari repository, dengan skel Niri milik upstream
(`~/.config/niri`), config di `~/.config/illogical-impulse` (nama yang dipakai
runtime QML-nya sendiri) dan state di `~/.local/state/inir`, plus data
companion di `~/.config/inir` (widget custom) dan
`~/.local/state/quickshell/{user,states.json}` (todo, catatan, chat, palet yang
diregenerasi). Beberapa kunci config-nya ikut di-seed: tema ikon `Colloid-Dark`
dan tiga keluarga font (`Inter`, `Inter`, `Maple Mono NF`). Sebabnya sama untuk
keduanya — default upstream (`WhiteSur-dark`, `Roboto Flex`,
`JetBrainsMono Nerd Font`) tidak dipaketkan repo mana pun di sini, dan nama yang
dipegang iNiR itulah yang ditulisnya ke gsettings serta ke settings GTK/Qt
setiap app, termasuk di sesi shell berikutnya; ikon tanpa tema itu jatuh ke
kotak kosong, font hanya jatuh ke keluarga lain. Dua keluarga lain yang
dokternya hitung penting — `Gabarito` (default `titleFont` upstream) dan
`Oxanium` — tidak ikut di-seed karena keduanya kini dipaketkan sendiri
(`gabarito-fonts` dan `oxanium-fonts`, dibangun di repo COPR pemelihara dari
file OFL google/fonts yang sama dengan fallback dokternya) dan masuk install
plan. Skel-nya juga yang membangun
venv Python iNiR (`~/.config/niri/scripts/inir-venv-setup.sh`, satu `spawn-at-startup`, idempoten, log di
`~/.local/state/quickshell/venv-setup.log`) — warna dari wallpaper, thumbnail,
dan fitur AI bergantung padanya, sementara switch sendiri hanya memasang paket
dan menjalankan shell. Yang membedakannya dari shell lain: unit systemd-nya
(`inir.service`) dikirim upstream tanpa bagian `[Install]`, jadi `systemctl
--user enable` menolaknya dan switch menulis sendiri symlink
`niri.service.wants/inir.service` — lihat `Port::systemd_user_unit_wanted_by`.

Aturan yang lebih umum: **skel boleh absen hanya di tempat shell tidak bisa
diinstal** — diuji
`a_skel_is_missing_only_where_the_shell_cannot_be_installed`. Niri karena itu
mendaftarkan shell yang skel-nya sudah ada (DMS, Noctalia, Ambxst, dan iNiR);
Caelestia belum punya port Niri, jadi tidak didaftarkan.

## Panel log

- **Tersembunyi saat idle.** Tidak pernah menampilkan versi aplikasi atau
  daftar shell; itu isi log aplikasi, bukan panel UI.
- Panel dibungkus `gtk::Revealer` dengan `reveal_child = false` sejak awal.
- Hanya tiga pemanggil yang boleh membukanya: `log::step()`, `log::failed()`,
  `log::succeeded()`. Semua penolakan guard dan "no-op" masuk ke log aplikasi
  lewat `crate::logging`, jadi panel tidak pernah menyala karena user sekadar
  menjelajah.
- Sukses → langsung tersembunyi lagi.
- Gagal → **tetap tampil sampai ditutup** user lewat tombol `✕` di header,
  sehingga alasan gagal tidak hilang sendiri.
- Tinggi tetap 400px, melekat di bawah.
- Header berisi judul `Progress` dan tombol tutup.
- `gtk::ScrolledWindow` dengan policy horizontal dan vertikal `Automatic` —
  baris panjang (daftar paket) digulir menyamping, bukan dipotong.
- Font monospace, read-only, tanpa cursor.
- Ditulis dengan `buffer.insert` di `end_iter`, satu baris per panggilan, lalu
  `scroll_to_iter` ke ujung buffer supaya langkah terbaru selalu terlihat
  tanpa digulir manual.
- Panel dikosongkan saat operasi baru dimulai (`Selection::clear_log`), jadi
  sisa baris operasi sebelumnya tidak terbaca sebagai langkah yang sudah
  berjalan.
- `log::panel()` mengembalikan **`Revealer`**. Versi lama mengembalikan
  `ScrolledWindow` di dalam frame sehingga styling `.log-frame` tidak pernah
  terpakai, dan versi sebelum itu selalu terlihat karena tidak ada cara
  menyembunyikannya.

## Palet

Nilai di bawah diambil dari `src/style.css`, bukan dari wireframe lama. Palet
sesungguhnya ungu gelap, bukan biru.

| Token | Nilai |
|---|---|
| Background | `#14131a` |
| Tab aktif | `#4a4458` |
| Border kartu | `alpha(white, 0.16)` |
| Border kartu terpilih | `#89b4fa` |
| Teks nama | `#cfc9d6` |
| Teks redup / `section-title` | `#8b8494` |
| Nonaktif | `#6b6577` |
| Sukses / `running` | `#a6e3a1` |
| Galat / `failed` | `#f38ba8` |
| Peringatan / guard | `#f9e2af` |

Semua warna lewat class CSS di `src/style.css`.

## Catatan

- Gambar wallpaper blur yang ada di wireframe asli **tidak dipakai**; window
  sekarang warna solid. Kalau wallpaper ditambahkan nanti, tinggi log tetap
  100px dan grid tetap boleh scroll.
- Placeholder `Logo DE` di wireframe lama sudah diganti logo shell sungguhan.
- Wireframe lama menyebut 8 kartu; yang benar 4 kartu per compositor, dan
  jumlah itu boleh bertambah tanpa mengubah layout.
