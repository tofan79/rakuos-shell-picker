# Alur Switch: Noctalia ⇄ DMS

Langkah demi langkah apa yang benar-benar terjadi pada mesin ketika berpindah
dari satu shell ke shell lain. Urutannya diambil dari
`switch::everything` → `apply::apply_with`, bukan dari urutan di UI — jadi yang
tercantum di sini adalah yang dijalankan, bukan yang dijanjikan jendelanya.

Dua arah dibahas terpisah karena tidak sama:

- **Noctalia → DMS** perlu *menginstal* sesuatu (DMS tidak ada di base image).
- **DMS → Noctalia** perlu *menghapus* sesuatu (DMS ada di repo, Noctalia tidak
  boleh disentuh).

Satu aturan menyatukan keduanya:

> **Yang ada datanya dibackup, yang tidak ada datanya dibersihkan.**

## Urutan umum

Setiap switch, di kedua arah, berjalan dalam tujuh langkah. Kode yang
mengerjakan setiap langkah disebutkan supaya bisa diperiksa langsung.

| # | Langkah | Kode | Gagal di sini |
|---|---|---|---|
| 0 | **Tolak** — shell tujuan belum terpasang dan tidak bisa dipasang, atau tidak ada yang bisa diswitch | `switch::prepare` step 1 | Berhenti. Tidak ada yang disentuh. |
| 1 | **Preflight** — skel shell tujuan harus punya konfigurasi kompositor | `switch::prepare` step 2 | Berhenti. Belum ada backup. |
| 2 | **Backup** | `switch::prepare` step 3 | Berhenti. Nol perubahan. |
| 3 | **Install** (kalau shell tujuan punya rencana instalasi) | `switch::install` | Backup tetap utuh, tidak ada yang di-apply. |
| 4 | **Apply**: plan → stage → stop lama → **backup di-ulang** → swap → start baru → verify | `apply::apply_with` | Rollback penuh (lihat bagian [Rollback](#rollback)). |
| 5 | **Clean** shell yang ditinggalkan → **reload** konfigurasi kompositor → **tunggu 5 detik** → **uninstall** paketnya | `apply::clean`, `Compositor::reload`, `switch::uninstall` | Tidak fatal — shell baru sudah hidup. |
| 6 | **Logout** | `apply::logout` | Switch tetap berhasil; diminta logout manual. |

Kenapa instalasi ada **setelah** backup dan sebelum apply: backup bersifat
read-only, jadi dikerjakan dulu sementara semuanya masih bisa menolak. Dan
tidak ada satu byte pun yang ditulis ke konfigurasi sebelum paketnya ada di
mesin — kalau `pkexec rum install` ditolak, tidak ada yang perlu dibatalkan.

---

# Arah 1: Noctalia → DMS

## 1. Backup

**Tujuan:** `~/.local/share/backups/rakuos-shell-picker/<nama>/`

Nama direktori dibentuk dari `<yyyymmdd>-<compositor>-<shell yang ditinggalkan>`,
memakai tanggal **UTC** dan tanpa jam:

```
20261007-hyprland-noctalia
```

Konsekuensinya: satu hari hanya menulis satu nama per shell yang ditinggalkan.
Switch kedua kalinya hari itu dari Noctalia akan **mengganti** direktori yang
sama, bukan menumpuk satu direktori per klik.

Isinya dua hal: `manifest.toml` (daftar lengkap, ukuran, dan path yang absen)
dan `home/` yang merupakan cermin dari `$HOME`.

Yang disimpan, lengkap dengan alasannya:

| # | Path | Kenapa |
|---|---|---|
| 1 | `.config/gtk-3.0/` | `SHARED_PATHS` — direktori sesi, bukan milik satu shell |
| 2 | `.config/gtk-4.0/` | sama |
| 3 | `.config/qt5ct/` | sama |
| 4 | `.config/qt6ct/` | sama |
| 5 | `.config/hypr/` | Konfigurasi kompositor. **Selalu** disimpan di setiap switch, karena `plan` mengambilnya dari skel shell tujuan — artinya file di sini diganti mentah-mentah dan salinan ini adalah satu-satunya cara untuk pernah kembali |
| 6 | `.config/noctalia/` | `config_dir` Noctalia |
| 7 | `.local/state/noctalia/` | `state_dir` — 389 dari 445 file yang di-backup ada di sini. Sebagian besar setup Noctalia yang sebenarnya ada di sini, bukan di `.config` |
| 8 | `.local/share/applications/dev.noctalia.Noctalia.desktop` | `backup_extra_paths` — launcher yang akan ditimpa switch |

Tabel di atas adalah separuh **shell yang ditinggalkan**. Swap juga menimpa
config dan state shell **tujuan**, jadi `backup_sources_for` menambahkan jalur
milik DMS yang sudah ada di mesin — `.config/dms`,
`.local/state/dms` dan companion-nya (`.config/dankcal`, `.config/dgop`,
`.local/share/dankcal`, `.local/state/dankcal`, serta setelan layanan
pencarian `.config/danksearch` — `.config/dsearch` sejak 2.0, dan dua ejaan
pra-2.0 `.config/DankMaterialShell` serta `.local/state/DankMaterialShell`
yang 2.0 simpan sebagai link ke direktori aslinya). Di mesin yang
belum pernah menjalankan DMS semuanya dicatat `absent`; di mesin yang pernah,
config lama itulah yang bikin tidak ada `.pre-*` tertinggal: dulu ia hanya
disimpan sebagai aside yang tidak direproduksi backup mana pun, sekarang ia
disalin ke backup dan asidenya dibuang seperti yang lain. Jalur yang sudah
tercakup daftar keluar (mis. launcher dan `SHARED_PATHS`) tidak didaftarkan dua
kali.

Path yang tidak ada di mesin **tetap ikut dalam daftar** dan dicatat sebagai
`absent`. Itu yang membuat restore bisa membedakan "memang tidak pernah ada"
dari "hilang".

Pembacaan ulang manifest terjadi **sebelum** shell lama dihentikan: kalau backup
hilang di antara saat ditulis dan saat diandalkan, switch berhenti dan tidak ada
yang diubah.

## 2. Install

DMS punya `install_plan`, dua tahap. Tiap tahap adalah satu transaksi
`pkexec rum install -y --refresh` dengan prompt polkitnya sendiri:

| Tahap | Paket |
|---|---|
| 1 | `quickshell-git` |
| 2 | `dms` `dms-cli` `dgop` `danksearch` `dankcalendar-git` `matugen` `khal` |

Urutannya aturan, bukan kenyamanan. Runtime (`quickshell-git`) jalan dulu dan
sendirian karena kecil, karena paling mungkin gagal untuk alasannya sendiri, dan
karena kalau gagal di situ **belum ada apa-apa dari DMS di mesin** — tidak ada
yang perlu dibersihkan. Satu transaksi gabungan akan menginstal semua atau
menggugurkan semua, dan itu menyeret shell-nya turun bersama runtime yang
gagal.

Paket yang sudah terpasang di lewati (`stage N: already installed, skipping`).
Kalau prompt polkit dibatalkan, switch gagal dengan pesan *"the permission
prompt for stage N was dismissed. Your backup is still in place."*

Hanya Noctalia **tidak punya** `install_plan` (ia ada di base image), sedangkan
DMS, Caelestia dan Ambxst punya. Pada arah yang tidak menyentuh apa pun (menuju
Noctalia) artinya tidak ada satu pun paket yang diinstal; menuju DMS, Caelestia
atau Ambxst, stage-stage di ataslah yang jalan — runtime `quickshell-git` di
stage pertama, lalu paket shell dan temannya di stage kedua.

## 3. Apply dan Stop

### 3a. Plan — menentukan asal tiap path

`apply::plan` memutuskan dari mana setiap file diambil, dan aturannya tidak
seragam:

- **`~/.config/hypr` SELALU dari skel shell tujuan**, tidak pernah dari backup.
  Kalau direktori ini diambil dari backup, switch akan mengembalikan
  konfigurasi shell lama dan dengan demikian membatalkan dirinya sendiri.
- `config_dir`, `state_dir`, `companion_paths`, `skel_extra_paths` dan
  `backup_extra_paths`: **backup yang namanya milik shell tujuan** (`…-<comp>-<id>
  shell tujuan>`), fallback ke skel. Inilah yang membuat pindah ke DMS lalu
  kembali ke Noctalia mengembalikan susunan asli, bukan ke default.

Yang kedua itu aturan inti dan mudah salah paham, jadi alasannya ditulis di
sini. Backup **terbaru secara keseluruhan bukan jawabannya**: direktori
`20261008-hyprland-dms` dibuat ketika DMS ditinggalkan, jadi isinya adalah
versi era DMS dari setiap file — `gtk.css` dengan
`@import url("dank-colors.css")` yang tidak lagi ditulis siapa pun,
`qt6ct.conf` menunjuk `matugen.conf`, `settings.ini` dengan tema DMS, dan
launcher Noctalia bersimbol `NoDisplay=true`. Memasangnya kembali pada saat
kembali ke Noctalia menyembunyikan Noctalia dari launcher dan membuat GTK
mengimpor stylesheet yang tidak ada. Sedangkan `20261008-hyprland-noctalia`
persis memegang era Noctalia dari file-file yang sama.

Jadi pencariannya difilter dulu ke direktori yang namanya diakhiri
`-<compositor>-<shell tujuan>`; di antara direktori itu, yang terbaru menang.
Kalau tidak ada satupun — shell ini belum pernah ada di mesin ini — jatuh ke
skel, yang berarti shell baru jalan dengan default-nya, bukan meminjam file dari
era shell lain. Backup yang dibuat saat tidak ada shell jalan bernama
`…-<comp>-none` dan ikut terlewat dengan sendirinya.

### 3b. Stage — salin semuanya dulu

Semua path di atas disalin ke direktori staging di
`~/.local/share/rakuos-shell-picker/staging-<target>-<stamp>` **sebelum** satu
pun file live disentuh. Baru setelah itu manifest backup dicek ulang masih ada.

### 3c. Catat pemilik bus notifikasi

`org.freedesktop.Notifications` dibaca **sebelum** shell lama dihentikan. Setelah
shell keluar, nama itu jadi tidak punya pemilik dan tidak bisa dibandingkan
dengan apa pun. Hasilnya dipakai di `verify` nanti.

### 3d. Stop shell lama

Cara berhenti mengikuti cara mulai, dari `stop_plan()` — ada dua cabang dan
pilihan cabangnya bukan perkiraan:

- Kalau `mechanism.uses_systemd_unit()` → `systemctl --user stop <unit>.service`.
  Menyignalkan prosesnya saja tidak cukup: unit-nya sendiri tetap hidup dan
  berada di luar jangkauan program ini, jadi yang melapor "sudah berhenti" dan
  yang diyakini sesi akan jadi dua pertanyaan berbeda. `systemctl stop`
  sinkron dan menunggu cgroup-nya kosong, lalu pid dicek ulang — unit bilang
  berhenti dan prosesnya benar-benar hilang adalah klaim yang sama, dan hanya
  satu yang diamati.
- Kalau tidak → sinyal ke pid yang `comm`-nya persis sama dengan `binary`.
  **SIGTERM dulu**, karena shell yang keluar bersih masih sempat menulis
  statenya, lalu diulang sampai `STOP_GRACE` (**10 detik**) habis, baru
  **SIGKILL**. Kill adalah cadangan, bukan rencana.

> `stop` tidak berhenti di `systemctl stop`. Unit shell-nya di-`disable` lalu
> di-`mask`, dan unit yang dideklarasikan `autostart_systemd_user_units` — untuk
> DMS, `dsearch` dari paket `danksearch` — di-`stop`, `disable` dan `mask` juga.
> Dua sumber enablement ditutup oleh ini: `rakuos-niri` menyalakan DMS dan
> `dsearch` lewat `systemctl --global enable` (`dms.service`
> `WantedBy=graphical-session.target`, `dsearch.service` `WantedBy=default.target`),
> sedangkan di Hyprland justru `enable` milik switch inilah satu-satunya yang
> akan menghidupkan keduanya di login. Tanpa mask, unitnya **kembali sendiri**
> di login berikutnya, di samping shell yang baru dipilih. Perlakuannya sama di
> kedua compositor — unitnya memang sama — dan `start` membalikkannya
> (`unmask` + `enable` + `start`), jadi tidak ada yang searah.

Pada arah ini outgoing shell-nya Noctalia, yang `mechanism: Autostart` dan
`systemd_user_unit: None` → **cabang sinyal**: SIGTERM ke pid `noctalia`.
Noctalia juga tidak punya companion, jadi tidak ada unit lain yang ikut.

> Companion dihentikan **setelah** shell-nya, bukan sebelum. Companion memang
> ada untuk shell, jadi ia sudah tidak berguna begitu shell pergi, dan
> menghentikannya lebih dulu akan terlihat merusak widget yang digambar shell
> yang masih hidup. Companion yang tidak mau berhenti dilaporkan tetapi **tidak
> mematalkan** switch — shell-nya sudah turun, dan switch yang gagal karena
> helper nakal akan menyisakan sesi tanpa shell sama sekali.

Kegagalan stop shell-nya sendiri di sini membatalkan switch **sebelum** ada satu
pun file yang diganti.

**Backup diambil ulang setelah stop** (`apply::refresh_backup`), ke direktori
yang sama. Alasannya ada di urutannya: backup ditulis **sebelum install**,
karena itulah yang membuat install bisa di-rollback — tapi artinya shell yang
ditinggalkan masih hidup selama install berjalan, dan bisa menulis statedirnya
sendiri di antara keduanya. Di mesin tempat ini pertama ditemukan, Noctalia
menulis dua entri clipboard dan riwayat notifikasi di jendela itu, lalu `clean`
menemukan backup yang tidak lagi menggambarkan file di disk — dan dengan benar
**menyimpan** file itu, karena menghapus di atas backup yang basi adalah menebak.
Hasilnya: switch selesai dengan menyisakan satu direktori yang katanya sudah
dihapus.

Stop menutup jendela itu: tidak ada yang bisa menulis state setelah prosesnya
keluar, jadi salinan kedua berisi **apa pun yang terakhir ditulisnya**, dan syarat
byte-for-byte di langkah 4 lolos karena alasan yang benar. Gagal mengambil ulang
**tidak membatalkan** — backup lama tetap ada dan tetap terverifikasi, dan
`clean` menyimpan setiap path yang berubah, persis perilaku sebelum baris ini
ada.

### 3e. Swap — pindahkan dari staging

Setiap path dipindahkan dari staging ke tempatnya. Yang ada di jalur **tidak
dihapus** melainkan di-rename menjadi:

```
.<nama-file>.pre-<target>-<stamp>
```

di direktori yang sama — misalnya `~/.config/.hypr.pre-dms-20261007`. Ini
satu-satunya hal yang membuat rollback mungkin setelah file live mulai diganti.

Kalau langkah ini gagal di tengah, asides dibalikkan, shell lama di-resume, dan
switch dilaporkan gagal.

Staging dihapus begitu swap selesai.

### 3f. Start shell baru

Urutannya companion dulu, baru shell:

1. `systemctl --user start dcal.service`
2. `systemctl --user start dms.service`

Companion duluan karena unit shell-nya sendiri **tidak** bergantung padanya:
systemd akan dengan senang hati menjalankan DMS tanpa `dcal`, dan hasilnya shell
yang tampak jalan tapi widget kalendernya salah — jenis kerusakan yang tidak
seorang pun sadari sampai nanti.

Untuk shell yang memakai unit systemd, `start` lebih dulu membalikkan apa yang
`stop` lakukan. Unit yang dideklarasikan `autostart_systemd_user_units` naik
dulu — untuk DMS, `dsearch`, dan kegagalannya membatalkan switch seperti
companion — lalu unit shell-nya sendiri, masing-masing lewat
`systemctl --user unmask`, `enable`, baru `start`. Itu yang membuat kembali ke
DMS persis kebalikan dari meninggalkannya, di kedua compositor.

### 3g. Verify

Shell tujuan harus benar-benar hidup, bukan sekadar sudah diluncurkan. Tiga
pemeriksaan, berurutan, dengan `START_GRACE` **90 detik**:

1. **Unit aktif** — hanya untuk shell yang memakai systemd. Bertanya
   `systemctl --user is-active` lebih kuat daripada mencari proses: unit bisa
   aktif sementara shell di dalamnya belum berhasil menggambar, dan proses bisa
   hidup sementara unit yang seharusnya mengawasinya tidak.
2. **Prosesnya muncul** — `pids_of(<binary>)` tidak kosong.
3. **Bus notifikasi jatuh ke shell baru** — dan inilah yang benar-benar
   mengatakan switch berhasil. Proses hidup dan unit aktif sama-sama kompatibel
   dengan shell lama yang masih memegang bus; itu kegagalan yang kalau tidak
   ditangkap baru terasa sebagai desktop tanpa notifikasi setelah logout.

Pemeriksaan ketiga bersifat spesial: shell yang di-spawn langsung butuh waktu
sebentar untuk mengklaim bus, jadi *"tidak ada yang punya"* dianggap sementara
selama masih di dalam grace period — bukan kegagalan seketika.

Gagal di sini → shell baru dihentikan dulu (supaya rollback tidak menghasilkan
dua shell setengah jalan yang berebut bus), asides dibalikkan, shell lama
di-resume.

Berhasil → asides dibuang, karena sekarang tidak ada lagi yang bisa rollback ke
salinan tersebut. Pembuangan tetap diperiksa dulu: asides hanya dihapus kalau
**backup mana pun** mereproduksinya byte-for-byte. Bukan "backup terbaru" —
mana pun, karena pertanyaannya satu-satunya apakah byte-nya aman di suatu
tempat, dan menanyakan direktori terbaru justru menyisakan `.pre-*` yang sudah
terverifikasi di disk (direktori terbaru milik shell yang baru masuk, sedangkan
asidenya dari shell sebelumnya).

## 4. Clean

Baru di sini, setelah shell baru terbukti hidup dan tidak ada jalan balik lagi
selain backup. Yang dibersihkan adalah **milik Noctalia**:

| Path | Jenis | Syarat penghapusan |
|---|---|---|
| `.config/noctalia/` | `config_dir` | backup switch ini memegangnya byte-for-byte |
| `.local/state/noctalia/` | `state_dir` | sama |
| `.config/gtk-3.0/noctalia.css` | `generated_paths` | sama |
| `.config/gtk-4.0/noctalia.css` | `generated_paths` | sama |
| `.config/qt5ct/colors/noctalia.conf` | `generated_paths` | sama |
| `.config/qt6ct/colors/noctalia.conf` | `generated_paths` | sama |
| `.cache/noctalia/` | `discard_paths` | **tanpa syarat** |

Dua kategori yang berbeda sengaja:

- **`config_dir`, `state_dir`, `generated_paths`** hanya dihapus kalau backup
  yang diambil untuk switch ini sendiri mereproduksinya persis. Cek ini perlu,
  karena shell-nya masih hidup saat backup diambil dan baru dihentikan sesaat
  kemudian — ia bisa saja sempat menulis sesuatu di antaranya. Tidak cocok →
  file **dipertahankan** dan dilaporkan, bukan dihapus.
- **`discard_paths`** dihapus atas dasar deklarasi saja. Isinya memang tidak ada
  data: cache dan berkas render yang tidak pernah diedit siapa pun. Tidak ada
  yang perlu dibackup dan tidak ada yang perlu dicek.

Guard tetap berlaku untuk keduanya: sebuah path **tidak disentuh** kalau shell
tujuan menyatakan path yang sama miliknya (`config_dir`, `state_dir`,
`skel_extra_paths`, atau `generated_paths`). Itu yang mencegah clean mengambil
kembali berkas yang barusan dipasang untuk shell yang sedang menyala.

## 5. Uninstall

**Tidak ada.** Noctalia punya `uninstall_plan: None` karena ia bagian dari base
image — `rum remove` pasti menolak, dan rencana yang selalu gagal lebih buruk
daripada tidak ada rencana.

Konfigurasi dan statenya tetap dibersihkan (langkah 4) karena itu file user,
bukan milik image.

## 6. Logout

Tiga jalur, dicoba berurutan (`apply::logout`):

1. `compositor.exit()` — meminta kompositor keluar lewat socket-nya sendiri
   (`hyprctl dispatch exit` di Hyprland), tanpa pertanyaan scope sama sekali.
2. `uwsm stop` — dipakai kalau UWSM yang memiliki sesi; berhenti sebagai unit,
   jadi tidak terkena aturan scope di bawah.
3. `loginctl terminate-session $XDG_SESSION_ID` — tepat ketika proses ini memang
   bagian dari sesi (mis. dijalankan dari terminal di dalamnya).

App yang dijalankan sebagai systemd user service hidup di
`user@UID.service/app.slice`, bukan di scope sesi grafis, sehingga logind
menolak `terminate-session` sebagai permintaan lintas-sesi — itulah sebabnya
jalur kompositor dan UWSM dicoba lebih dulu. Kalau ketiganya gagal, switch tetap
berhasil dan user diminta logout manual.

Logout adalah bagian dari pekerjaan, bukan saran: sesi yang masih berjalan tetap
memakai konfigurasi Hyprland lama, dan shell barunya baru benar-benar hidup di
sesi berikutnya.

---

# Arah 2: DMS → Noctalia

Urutan langkahnya identik. Yang berbeda adalah isi setiap langkah.

## 1. Backup

**Tujuan:** `~/.local/share/backups/rakuos-shell-picker/20261007-hyprland-dms/`

| # | Path | Kenapa |
|---|---|---|
| 1–4 | `.config/gtk-3.0/`, `.config/gtk-4.0/`, `.config/qt5ct/`, `.config/qt6ct/` | `SHARED_PATHS` |
| 5 | `.config/hypr/` | konfigurasi kompositor, selalu disimpan |
| 6 | `.config/dms/` | `config_dir` DMS |
| 7 | `.local/state/dms/` | `state_dir` DMS (`session.json` sudah ada di skel, tetapi file user di sini tetap disimpan) |
| 8 | `.local/share/applications/dev.noctalia.Noctalia.desktop` | `backup_extra_paths` |

Direktori `.cache/` **tidak** ikut backup. Tidak ada yang perlu disimpan di
dalamnya (lihat [Ke mana cache pergi](#ke-manakah-cache-pergi)).

## 2. Install

**Tidak ada.** Noctalia tanpa `install_plan` — log akan menulis *"Noctalia has
no install plan, so no packages were touched"*. Tidak ada prompt polkit, tidak
ada yang menunggu.

## 3. Apply dan Stop

### Stop

Cabang **unit**, karena DMS `mechanism: SystemdUserUnit`:
`systemctl --user stop dms.service`. `KillMode=control-group` berarti seluruh
proses dalam unit ikut mati — dan itu penting di sini, karena DMS menjalankan
`qs -p /run/user/1000/danklinux-shell/...` yang `comm`-nya `qs`. Pencocokan
nama biasa tidak akan pernah menemukannya; cgroup-nya yang menemukan.

Bersamaan itu companion **`dcal.service` ikut dihentikan**. `dankcalendar-git`
akan dihapus pada langkah 5, jadi membiarkannya hidup berarti meninggalkan unit
yang menunjuk ke binary yang sudah tidak ada.

Sama seperti arah 1, backup **diambil ulang** setelah kedua unit berhenti —
lihat [3d](#3d-stop-shell-lama) untuk alasannya.

### Start

Noctalia `mechanism: Autostart` dan tanpa unit systemd, jadi cabangnya yang
kedua: picker **men-spawn `noctalia` langsung** sebagai child-nya sendiri
(`start_argv: ["noctalia"]`, tanpa `--daemon` — Hyprland tidak menunggu perintah
startup selesai). Companion tidak ada yang bisa dimulai.

Yang menjalankan Noctalia pada sesi berikutnya adalah `config/autostart.lua` di
konfigurasi Hyprland yang barusan dipasang; spawn ini yang memastikan shell-nya
langsung ada di sesi sekarang.

### Verify

Pemeriksaan pertama (unit aktif) **dilewati** — Noctalia tidak memakai unit.
Sisa urutannya sama: tunggu pid `noctalia` muncul selama `START_GRACE` 90 detik,
lalu tunggu bus notifikasi jatuh kepadanya.

Kegagalan yang ditangkap di sini adalah `dms.service` yang masih memegang bus
karena stop-nya diam-diam tidak sempurna. Karena itu verify gagal → rollback
penuh, bukan lanjut dengan dua shell yang berebut satu bus.

## 4. Clean

Yang dibersihkan adalah **milik DMS**:

| Path | Jenis | Syarat penghapusan |
|---|---|---|
| `.config/dms/` | `config_dir` | backup byte-for-byte |
| `.local/state/dms/` | `state_dir` | sama |
| `.config/gtk-3.0/dank-colors.css` | `generated_paths` | sama |
| `.config/gtk-4.0/dank-colors.css` | `generated_paths` | sama |
| `.config/qt5ct/colors/matugen.conf` | `generated_paths` | sama |
| `.config/qt6ct/colors/matugen.conf` | `generated_paths` | sama |
| `.cache/dms/` | `discard_paths` | **tanpa syarat** |
| `.cache/DankMaterialShell/` | `discard_paths` | **tanpa syarat** (ejaan pra-2.0, symlink) |
| `.cache/dankcal/` | `discard_paths` | **tanpa syarat** |
| `.cache/dank-calendar/` | `discard_paths` | **tanpa syarat** |
| `.cache/danksearch/` | `discard_paths` | **tanpa syarat** (ejaan pra-2.0) |
| `.cache/dsearch/` | `discard_paths` | **tanpa syarat** (index `dsearch` 2.0) |
| `.cache/quickshell/` | `discard_paths` | **tanpa syarat** |
| `.config/ghostty/themes/dankcolors` | `discard_paths` | **tanpa syarat** |
| `.config/zed/themes/dank-zed-theme.json` | `discard_paths` | **tanpa syarat** |
| `.local/share/color-schemes/DankMatugen.colors` | `discard_paths` | **tanpa syarat** |
| `.local/share/color-schemes/DankMatugenDark.colors` | `discard_paths` | **tanpa syarat** |
| `.local/share/color-schemes/DankMatugenLight.colors` | `discard_paths` | **tanpa syarat** |

Symlink yang targetnya sudah hilang tetap terhitung sebagai entri: `.cache/dms/`
dibuang tepat di atas `.cache/DankMaterialShell/`, jadi saat giliran link itu tiba
yang ditunjuknya sudah tidak ada. Pemeriksaannya lewat `present()` di `apply.rs` —
`Path::exists()` mengikuti link dan akan membaca "tidak ada apa-apa" untuk link
yang menggantung, sehingga entri itu diam-diam tertinggal di home.

Yang **sengaja tidak** dibersihkan:

- **Direktori induk** `.config/ghostty/`, `.config/zed/`,
  `.local/share/color-schemes/` — direktori itu milik programnya masing-masing.
  Yang diambil hanya file yang DMS yang menulisnya.

### Ke manakah cache pergi?

Dua kategori cache dianggap berbeda:

- **`.cache/` DMS, dcal, search, dan Quickshell** dibuang. Isinya ulangan yang
  bisa diregenerasi (`matugen.lock`, `dms-colors.json`, shader cache milik paket
  yang barusan dihapus, index pencarian, kompilasi QML beberapa ratus byte) —
  tidak ada yang pernah
  diedit orang, dan shell berikutnya membangun ulang apa pun yang ia butuhkan
  pada start pertama.
- **`.cache/` yang isinya data** tidak pernah ada di daftar `discard_paths`
  sama sekali — jalur seperti itu termasuk backup, dan penghapusannya menunggu
  syarat byte-for-byte.

## 5. Uninstall

Dua langkah pendahulu, keduanya sebelum satu kata pun `rum` dijalankan:

1. **`hyprctl reload`.** Kompositor masih membaca konfigurasi yang baru saja
   diganti, dan tidak ada yang me-reload-nya — sesi baru baru terjadi di
   langkah 6, setelah tahap privileged ini. Me-reload dulu membuat aturan shell
   baru yang ikonik di `.config/hypr` jadi yang berlaku selama sisa switch.
   Kegagalannya dilaporkan, tidak membatalkan apa pun.
2. **Jeda 5 detik (`AGENT_GRACE`).** Shell yang menjawab polkit sekarang adalah
   shell **baru**, dan ia mendaftarkan auth agent-nya **setelah** hidup: DMS
   menyalakan `PolkitService`, Noctalia memakai `polkit_agent = true`. Kalau
   `pkexec` dijalankan satu detik setelah start, tidak ada yang menjawab,
   polkit jatuh ke prompt teks milik pkexec, dan prompt itu butuh
   *controlling terminal* yang tidak dimiliki sebuah window — hasilnya
   `Error creating textual authentication agent` lalu `stage 1 was not
   authorized`. (Terjadi di mesin nyata: shell lama sudah di-stop, shell baru
   baru 1 detik hidup, prompt tidak pernah muncul.)

Baru sesudah itu:

Satu tahap, satu transaksi: `pkexec rum remove -y` atas

```
dms  dms-cli  dgop  danksearch  dankcalendar-git  matugen  khal  quickshell-git
```

Lima hal yang membuat daftar ini berbentuk begitu:

- **`quickshell-git` ikut, dengan guard.** DMS yang menginstalnya, dan setelah
  `dms` + `dankcalendar-git` berada di transaksi yang sama tidak ada lagi yang
  memintanya — itu verified langsung: `rum remove --dry-run` atas tujuh paket
  pertama membuang 7 dan menyisakan `quickshell-git` sebagai yatim 6,3 MiB yang
  hanya `rum autoremove` akan temukan. Tapi runtime itu dipakai shell lain, dan
  sekarang penjagaannya ada di tempatnya: `switch::uninstall` **men-drop** setiap
  paket yang ada di `runtime_packages` milik **shell yang sedang dinyalakan**,
  sebelum package manager melihatnya.

  | Target | `quickshell-git` |
  |---|---|
  | Noctalia | **dihapus** — biner asli, tidak membutuhkannya |
  | DMS | **dipertahankan** — DMS berjalan di atasnya, jadi mencabutnya = shell baru tanpa runtime |
  | Caelestia | **dipertahankan** — Caelestia berjalan di atasnya, jadi mencabutnya = shell baru tanpa runtime |

  Aturan yang sama berlaku pada arah sebaliknya, dan itu yang membuat Caelestia
  → DMS maupun Caelestia → Noctalia sama bersihnya dengan DMS → keduanya:
  plan pencopotan Caelestia berisi `caelestia-shell` **dan** `quickshell-git`
  (apa yang diinstalnya pula), lalu guard di atas men-drop runtime hanya kalau
  shell tujuan memang memakainya — tujuan DMS mempertahankan, tujuan Noctalia
  membuang. Tidak ada yang ditinggal sebagai yatim untuk `rum autoremove`.

- **`matugen` ikut dihapus, tetapi** `rum remove` menolak paket yang masih
  disyaratkan paket lain — solver adalah garis pertahanan kedua, bukan pertama.
- **Hanya dijalankan setelah shell baru terbukti hidup**, karena itu tahap
  terakhir: backup sudah ada dan jalur pulang sudah aman sebelum sesuatu
  dicopot.
- **Kegagalan tidak fatal.** Switch sudah sampai tahap ini berarti berhasil;
  paket yang tidak mau pergi membuat mesin lebih kotor dari yang diminta, bukan
  rusak.
- **Noctalia tidak pernah masuk sini** — `uninstall_plan`-nya `None`.

Kalau seluruh paket yang tersisa sudah tidak ada, tahapnya dilewati tanpa
menyentuh package manager. Kalau **semua** paketnya kebetulan ada di
`runtime_packages` target, hasilnya `Removed::Kept` — dilaporkan sebagai "tidak
ada yang dihapus", bukan "sudah tidak ada", supaya tidak ada yang mencari paket
yang memang sengaja ditinggalkan.

## 6. Logout

Sama persis dengan arah pertama: tiga jalur berurutan (`compositor.exit()` →
`uwsm stop` → `loginctl terminate-session`), dan kalau semuanya gagal switch
tetap dilaporkan sukses dengan catatan bahwa logout manual diperlukan.

---

## Rollback

Berlaku di **kedua arah**. Seberapa banyak yang harus dibatalkan tergantung
gagalnya di mana:

| Gagal di | Yang sudah terjadi | Yang dilakukan |
|---|---|---|
| `plan` | belum ada yang disentuh | berhenti |
| `stage` | hanya file salinan di staging | staging dihapus, berhenti |
| backup hilang | belum ada yang disentuh | berhenti |
| `stop` | shell lama sudah berhenti, file belum diganti | staging dihapus, berhenti |
| `swap` | shell lama berhenti, sebagian file sudah diganti | **rollback 2 langkah** |
| `start` / `verify` | shell baru sudah hidup, file sudah diganti | **rollback 3 langkah** |

Rollback penuh, dalam urutannya:

1. **Shell baru dihentikan dulu.** Ia mungkin sudah memegang bus notifikasi, dan
   menjalankan shell lama kembali berdampingan dengannya berarti salah satu dari
   keduanya gagal memilikinya — rollback yang menghasilkan dua shell setengah
   jalan, bukan satu shell utuh seperti sebelumnya.
2. **Asides dibalikkan** ke tempatnya.
3. **Shell lama di-resume.**

Laporan `rolled_back: true` hanya keluar kalau **ketiganya** berhasil.
`rolled_back: false` berarti ada yang tersisa dalam keadaan setengah, dan itu
dilaporkan apa adanya, bukan ditutupi.

Keadaan yang **tidak** bisa di-rollback: instalasi paket (langkah 2) dan
pembersihan (langkah 5). Keduanya sengaja diletakkan di posisi terakhir untuk
itu — pembersihan baru jalan setelah tidak ada jalan balik selain backup, dan
instalasi baru jalan setelah backup terbukti ada.

## Ringkasan perbedaan

| | Noctalia → DMS | DMS → Noctalia |
|---|---|---|
| Backup | `…-hyprland-noctalia` | `…-hyprland-dms` |
| Install | 2 tahap, 8 paket, prompt polkit per tahap | tidak ada |
| Stop shell lama | SIGTERM → KILL, pid `noctalia` | `systemctl stop dms.service` + `dcal.service` |
| Start shell baru | `dcal.service`, lalu `dms.service` | spawn `noctalia` langsung, tanpa companion |
| Verify | unit aktif + pid + bus | pid + bus (tanpa unit) |
| Clean (berdata) | 2 path | 2 path |
| Clean (generated) | 4 path tema | 4 path tema |
| Clean (tanpa data) | 1 cache | 4 cache + 5 file render |
| Uninstall | tidak ada (base image) | 1 tahap, 8 paket (quickshell dijaga `runtime_packages`) |
| Logout | sama | sama |

## Lihat juga

- [SAFETY.md](SAFETY.md) — aturan yang mengikat soal backup dan rollback
- [ARCHITECTURE.md](ARCHITECTURE.md) — struktur modul dan field registri
- [FEATURES.md](FEATURES.md) — status tiap fitur
