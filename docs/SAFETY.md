# Keamanan Konfigurasi User

Bagian ini mengikat. Menyentuh kode backend berarti menerima semua aturan di
sini. Kalau suatu fitur tidak bisa dibuat dengan aturan ini, fiturnya yang
dipertahankan, bukan aturannya yang dilonggarkan.

## Yang tidak boleh pernah terjadi

  1. **Batasi perubahan hanya pada konfigurasi yang berkaitan dengan shell.** Picker hanya boleh mengelola file/folder yang jelas berkaitan dengan shell target; aplikasi umum tidak boleh disentuh. Semua perubahan wajib dibackup, tanpa `rm`/`mv` tanpa backup, dan tidak boleh truncate.
2. **Tidak pernah me-restart atau mematikan compositor.** Compositor yang sedang
   menjalankan sesi user tetap jalan selama picker hidup. Kalau picker mati,
   compositor harus tetap hidup dan shell harus tetap bisa jalan ulang.
3. **Tidak menyentuh compositor lain.** Konfigurasi sebuah compositor tidak boleh
   dibaca untuk ditulis oleh switch yang berjalan di compositor lain.

   Aturan 3 sekarang tidak bisa dicapai dari aplikasi yang sedang berjalan:
   hanya Hyprland yang terdaftar, jadi tidak ada tab milik compositor lain untuk
   diklik. Aturannya tetap ada karena compositor berikutnya akan membutuhkannya,
   dan tetap diuji lewat `compositor::fake` yang hanya ada di build test.
4. **Tidak mengarang lokasi konfigurasi.** Kalau `config_dir` untuk sebuah shell
   belum diketahui, backend itu belum boleh jalan — bukan menebak
   `~/.config/<nama>`.
5. **Tidak menjalankan `kill -9` sebagai cara normal.** Shell harus diberi
   kesempatan berhenti rapi; `kill -9` hanya boleh jadi jalan keluar terakhir
   setelah timeout.
6. **Tidak pernah menyalakan shell milik compositor lain.** Lihat
   [Deteksi dan Guard](#deteksi-dan-guard) di bawah.
7. **Tidak pernah menebak compositor yang sedang jalan.** Kalau tidak
   terdeteksi, tidak ada switch sama sekali.

## Deteksi dan Guard

Compositor tidak berganti lewat picker. Mengganti compositor berarti mengganti
image, dan itu urusan **"RakuOS system management"**, bukan aplikasi ini.

Jadi kalau ada tab milik compositor lain, semua aksi di tab itu harus
**ditolak**, bukan dijalankan lalu gagal setengah jalan.

### Guard tidak punya arah

Penolakan ini **simetris**, dan itu bukan hal yang bisa dibuktikan dengan satu
compositor. Sekarang Niri sudah terdaftar, jadi kalau user berada di Niri, tab
Hyprland yang ditolak, dengan alasannya terbalik: "hyprland is not the running
compositor — niri is". Tidak ada kode yang menyebut Hyprland sebagai pihak yang
berwenang.

Aturannya satu perbandingan: `detection.current != compositor.id()`. Karena itu
menambah compositor ketiga tidak perlu menyentuh logika guard sama sekali.

Kedua arah diuji terhadap compositor yang tidak terdaftar (`compositor::fake`),
supaya guard tetap diuji sebagai perbandingan murni, bukan lewat daftar yang
kebetulan terisi dua.

App juga membuka tab compositor yang sedang jalan, bukan tab pertama. Kalau tidak,
user compositor lain akan dibuka ke halaman Hyprland yang ditolak, dengan banner
kuning dan tanpa petunjuk bahwa tab sebelahnya yang bisa dipakai.

Kedua arah ini diuji (`the_guard_works_in_both_directions`,
`the_running_compositor_is_the_only_one_that_offers_a_switch`,
`the_app_opens_on_the_running_compositors_tab`), jadi kalau suatu saat guard
diubah jadi satu arah, testnya gagal.

### Cara menentukan compositor yang jalan

Envvar khusus compositor, yang paling bisa dipercaya:

| Compositor | Penanda | Andal |
|---|---|---|
| Hyprland | `HYPRLAND_INSTANCE_SIGNATURE` terisi | ya |
| Niri | `NIRI_SOCKET` ada dan socket-nya bisa dibuka | ya |
| Sway | `SWAYSOCK` | ya |

Yang terdaftar sekarang Hyprland dan Niri, jadi `detect_with` hanya akan pernah
menyebut salah satu dari keduanya atau tidak sama sekali. Sesi lain (misalnya
Sway) tidak terdeteksi, dan seluruh klik ditolak dengan `RefuseUndetected`,
bukan menebak compositor mana yang sedang berjalan.

Ditambah pemeriksaan dasar yang wajib lolos dulu: `WAYLAND_DISPLAY` dan
`XDG_RUNTIME_DIR` terisi.

`XDG_CURRENT_DESKTOP` **tidak boleh dipakai** untuk guard — nilainya bisa
dikarang user, sehingga tidak layak jadi dasar keputusan keselamatan.

Kalau penanda khusus tidak ada, urutannya: cek `WAYLAND_DISPLAY` (namanya
kadang memuat compositor), lalu scan `/proc/*/exe`. Envvar tetap cara utama.

### Kalau tidak terdeteksi

Tidak terdeteksi berarti picker dijalankan di luar sesi Wayland yang bisa
diandalkan. Dalam keadaan ini: **tolak semua switch**, tampilkan peringatan,
jangan menebak. Menjalankan shell yang salah lebih buruk daripada tidak
menjalankan apa pun.

### Dua lapis pertahanan

| Lapis | Wajib | Tanggung jawab |
|---|---|---|
| UI | ya | Card untuk compositor yang tidak sedang jalan diberi class `disabled`, tidak bisa diklik, ada badge penjelas |
| Backend | **ya** | `preflight()` menolak dengan alasan yang jelas |

UI saja tidak cukup — guard bisa dilewati. Backend yang benar-benar menegakkan
aturan; UI cuma supaya user tidak dicoba-coba.

Pesan penolakan yang dipakai:

```
niri is not the running compositor.
niri is running. Switch image in "RakuOS system management".
```

Baris kedua menyebut compositor yang **sedang berjalan**. `dialog` sebelumnya
menerima `current` lalu membuangnya, jadi user diberi tahu apa yang diblokir
tanpa diberi tahu apa yang harus dipakai sebagai gantinya.

## Packages: `rpm -q` tidak bisa dipercaya

RakuOS jalan dalam mode *split*: base image yang read-only, plus overlay tempat
semua instalasi yang dibuat setelah image itu. `rum` menyimpan keduanya
terpisah, dan memegangnya tetap pisah adalah disengaja:

```
$ rum paths
mode: split (RakuOS overlay)
base snapshot rpmdb: /usr/share/rpm
overlay rpmdb:       /var/lib/rakuos/rum-rpmdb
```

```
$ rum -qa | wc -l                                   # 1895
$ rpm --dbpath /usr/share/rpm -qa | wc -l          # 1895
$ rpm --dbpath /var/lib/rakuos/rum-rpmdb -qa | wc -l   # 248
```

`rpm -q` membaca base saja. Dari 248 paket overlay, 243 di antaranya tidak
ditemukan `rpm -q`. Konsekuensinya kalau backend memakainya: `rum install dms`
berhasil, lalu picker tetap menampilkan "not installed" dan preflight menolak
switch ke shell yang barusan terpasang.

Jawaban yang benar:

```
$ rum origin noctalia     ->  noctalia: Base
$ rum origin firefox     ->  firefox: Overlay
$ rum origin dms          ->  dms: NotInstalled
```

Jebakan kedua: `rum info <paket>` keluar dengan status 0 bahkan kalau paketnya
tidak ada. "Tidak ada" hanya muncul di teks output:

```
$ rum info nonexistent-package
No package named 'nonexistent-package' found.        # exit=0
```

Jadi exit code tidak boleh dipakai sebagai pemeriksaan ketersediaan. Yang dipakai
`rum origin`, atau `rum repoquery`.

### Terpasang, dan masuk overlay, itu dua hal berbeda

| Pertanyaan | Sumber jawaban |
|---|---|
| Apakah shell-nya terpasang? | `rum origin` -> Base / Overlay / NotInstalled |
| Shell mana yang tercatat di overlay? | `/var/lib/rakuos/packages.list` dan `/var/lib/rakuos/packages-rpm.list` |

Dua file terakhir itu yang dibaca tab **Overlay** di "RakuOS system management",
jadi picker membaca file yang sama supaya tidak pernah bertentangan dengan
yang ditampilkan sistem. Bedanya: `packages.list` hanya memuat paket yang
memang diminta user, sedangkan rpmdb overlay ikut memuat seluruh dependency
yang ikut terpasang.

Konsekuensi yang sudah terlihat sekarang: `Noctalia` ada di base image
Hyprland, sedangkan `dms`, `caelestia` dan `ambxst` semuanya ada di repo
`rakuos` dengan paketnya masing-masing (`dms` belum terpasang di mesin ini;
`ambxst` sedang aktif lewat paket overlay-nya). Keempatnya punya install plan.
`Noctalia` tetap punya install plan justru karena di base image Niri ia tidak
ada, jadi ia harus bisa dipasang saat picker dipakai di Niri.

Uninstall plan memakai `rum origin` untuk memilih paket mana yang boleh
dihapus: hanya paket ber-origin **Overlay** yang dikirim ke
`pkexec rum remove -y`. Paket **Base** adalah bagian image read-only — `rum
remove` menolaknya, dan memaksakannya akan merusak image — jadi paket itu selalu
dilewati dan dilaporkan sebagai "part of the base image".

## Wajib: backup sebelum switch

Urutan satu-satunya yang boleh dipakai:

```
 1. refuse    : guard compositor, lokasi config shell target diketahui, dan
                tidak ada shell asing yang sedang hidup. Shell target yang belum
                terpasang BOLEH diteruskan selama ia punya rencana install, karena
                backup selalu mendahului install
 2. preflight : environment + start mechanism siap (binary, unit systemd, atau
                file autostart benar-benar ada)
 3. BACKUP    : delapan path di bawah, ditambah jalur milik shell target yang
                sudah ada di mesin (lihat `backup_sources_for`), disalin ke
                backups/<tanggal>-<compositor>-<shell>/ lalu DIBACA ULANG untuk
                dicocokkan dengan manifest. Namanya tanggal saja, jadi satu hari
                satu direktori: switch kedua pada hari yang sama menggantikan
                yang pertama, dan hanya setelah yang baru terbukti utuh
                                            <- gagal di sini, switch dibatalkan
 4. catat state "sedang berganti"           -> supaya sesi bisa dipulihkan
  5. INSTALL   : satu privileged transaction per tahap,
                 pkexec rum install -y --refresh <paket>,
                 hanya untuk paket yang memang belum ada
                                             <- gagal di sini, switch dibatalkan
  6. terapkan skel : config shell target, file skel bersama dan ~/.config/hypr
                     diganti penuh; yang diambil dari backup milik shell target
                     kalau ada, kalau tidak dari skel — kecuali ~/.config/hypr
                     yang SELALU dari skel; file yang diganti disimpan dulu
                     sebagai .pre-*, dibuang lagi setelah langkah 9 berhasil
  7. stop      : shell lama diberi SIGTERM -> ditunggu -> batas waktu -> kill -9
                 hanya sebagai upaya terakhir
  7b. backup-ulang: backup diambil sekali lagi ke direktori yang sama, karena
                 shell lama sudah berhenti dan tidak bisa lagi menulis apa yang
                 langkah 3 simpan saat ia masih hidup; salinan keduanya berisi
                 apa yang terakhir ia tulis, sehingga langkah 12 menemukan
                 backup yang benar-benar menggambarkan file di disk
                                             <- gagal di sini TIDAK membatalkan
  8. start     : shell baru, sesuai StartMechanism-nya
  9. verify    : shell baru benar-benar hidup dalam batas waktu
 10. state     : tulis active/previous
 11. ROLLBACK  : kalau langkah 8/9 gagal, start lagi shell lama dan kembalikan
                 config dari backup
 12. bersih    : .pre-* yang direproduksi backup mana pun dibuang; lalu config,
                 state, data companion dan tema hasil render shell keluar
                 dihapus — hanya yang backup switch ini gambarkan byte untuk
                 byte, sisanya disimpan dan alasannya dilaporkan
                 <- gagal di sini TIDAK membatalkan; switch sudah sukses
 13. hapus paket: pkexec rum remove -y, hanya paket milik shell keluar yang
                  ber-origin Overlay (Base dilewati), hanya kalau shell itu
                  punya uninstall_plan
                  <- gagal di sini juga tidak membatalkan
```

**Langkah 12 dan 13 tidak pernah menggagalkan switch, dan itu disengaja.**
Keduanya berjalan setelah verify, artinya tidak ada jalur rollback yang tersisa
— membatalkan berarti membongkar konfigurasi yang sudah terbukti jalan. Yang
bisa terjadi hanya mesin lebih kotor dari yang diminta, dan itu dilaporkan
begitu adanya. Pilihan berlawanan juga salah: menaruh bersih atau uninstall
sebelum verify akan membuat kegagalan verify merusak shell yang belum sempat
diganti.

**Tidak ada langkah "cek dependency" terpisah, dan itu disengaja.** `rum install`
memang sudah menyelesaikan transaksi itu sendiri: paket yang sudah ada dilewati,
yang kurang ditarik, dan Provide yang salah tidak akan terpilih. Pemeriksaan
terpisah hanya mengulang pekerjaan itu, menambah detik ke setiap klik, dan
membaca metadata yang bisa sudah basi saat install berjalan. Yang unsafe bukan
"solvernya memilih" — unsafe kalau kita **tidak menentukan** pilihannya. Maka
nama paket ditulis langsung di command line, dan itu dibaca pada saat yang
benar:

| paket | ditulis karena |
| --- | --- |
| `quickshell-git` | `dms` meminta `(quickshell or quickshell-git)`; repo juga punya `noctalia-qs` yang menyediakan kapabilitas sama. Itu build Noctalia lama, hanya dibutuhkan paket `noctalia-legacy` yang sudah mati. |
| `dms-cli` | hard requirement yang versinya dikunci `= 1.6.2-1.fc44` |
| `dankcalendar-git` | **tidak menjadi dependency RPM DMS sama sekali**, tapi `quickshell/Modules/DankDash` memakainya. Tanpa itu widget kalender rusak, dan tidak ada metadata yang akan memberi tahu. |

Install dipecah jadi dua tahap, `quickshell-git` lebih dulu sendirian. Kalau
tahap itu gagal, tahap kedua tidak dijalankan, jadi tidak ada satu pun paket DMS
di mesin. Menggabungkan keduanya berarti kegagalan runtime ikut menjatuhkan
paket shell tanpa keuntungan apa pun.

**Install memaksa refresh metadata (`--refresh`), dan itu juga disengaja.**
Cache repo punya umur (`metadata_expire`), dan umur itu hanya menjawab "seberapa
tua salinan ini" — bukan "apakah repo berubah". Salinan kemarin masih di dalam
jendela itu sementara sudah tidak lagi menyebut paket yang ada di server, dan
gejalanya tidak kelihatan seperti masalah metadata: transaksinya resolve dengan
mulus, lalu unduhannya tidak menemukan file apa pun (`No more mirrors to try`).
Itu bukan skenario khayalan — persis itu yang terjadi saat menguji switch, cache
6 Okt masih menyebut `quickshell-git 0.3.2^866` sedangkan server sudah punya
`0.3.2^867`, dan tahap 1 gagal padahal repo-nya sehat. Refresh menambah beberapa
detik; resolve terhadap metadata yang muda tapi salah menggagalkan seluruh tahap,
dan backup sudah terlanjur diambil.

Refresh-nya ditaruh di `args` rencana install, bukan sebagai perintah terpisah
yang dijalankan lebih dulu. Alasannya praktis sekaligus jujur: `rum install`
butuh hak akses root, jadi yang dibacanya cache di `/var/cache/rum` — cache yang
sama sekali tidak bisa ditulis oleh user. Menjalankan `rum make-cache` sebagai
user keluar sukses tanpa mengubah apa pun yang nantinya dibaca install. Satu
flag di dalam transaksi yang sama berarti tidak ada prompt polkit kedua, dan
baris yang tertulis di log memang baris yang dieksekusi.


Backup selalu **lebih dulu**, sebelum satu pun langkah yang menulis. Backup
hanya membaca, jadi tidak ada ruginya kalau ternyata langkah berikutnya batal.
Install adalah langkah pertama yang menyentuh sistem, jadi ia diletakkan
setelah semua langkah yang masih boleh menolak.

### Yang di-backup

Delapan path, relatif terhadap `$HOME`:

```
.config/gtk-3.0            bersama     milik sesi, bukan milik shell
.config/gtk-4.0            bersama
.config/hypr               compositor  diganti penuh dari skel shell target
.config/qt5ct              bersama
.config/qt6ct              bersama
.config/noctalia           shell       keluar
.local/state/noctalia      shell       keluar
.local/share/applications/ shell       launcher Noctalia, ditimpa skel shell
  dev.noctalia.Noctalia.desktop        yang menggantikan (dms)
```

`~/.local/state/noctalia` ikut karena sebagian besar setup Noctalia ada di
sana: 389 file, bukan config ringan. Path yang tidak ada dicatat sebagai
`absent`, bukan dianggap gagal, supaya saat restore "tidak pernah ada" bisa
dibedakan dari "hilang".

Yang bertanda `shell` di kolom ketiga — `config_dir` dan `state_dir` — persis
daftar yang dibersihkan setelah switch (langkah 12). Satu deklarasi, tiga
kegunaan: di-backup, dihapus saat keluar, dibaca balik saat kembali. Untuk DMS
keduanya `.config/dms` dan `.local/state/dms`,
sehingga jumlahnya tetap delapan.

Satu daftar keempat ikut dibersihkan, `generated_paths`: file yang shell itu
tulis **sendiri** ke direktori bersama, bukan oleh switch — Noctalia
merender `gtk-3.0/noctalia.css`, `gtk-4.0/noctalia.css`,
`qt5ct/colors/noctalia.conf` dan `qt6ct/colors/noctalia.conf` setiap kali
temanya berubah. Keempatnya tidak muncul sebagai entri di tabel karena
direktorinya sudah ada di kolom `bersama`, tapi tetap harus dihapus: tidak ada
shell berikutnya yang menimpanya, jadi pindah dari Noctalia akan meninggalkan
empat file tema yang tidak akan pernah dipakai lagi. Syaratnya sama persis
dengan langkah 12 — hanya yang backup switch ini reproduksi byte untuk byte —
dan keempatnya bisa memenuhinya justru karena direktorinya di-backup.

Satu daftar kelima ikut dibersihkan, `discard_paths`, dan justru karena itu
tidak memakai syarat langkah 12: cache dan file render yang **tidak pernah
diedit siapa pun**. Tidak ada yang perlu disimpan untuk sebuah berkas yang bisa
diregenerasi shell pembuatnya, jadi `clean` menghapusnya atas dasar deklarasi
saja, dan backup tidak membawanya — satu switch tidak seharusnya memindahkan
beberapa megabyte ulangan. Ambxst mendeklarasikan dua: `~/.cache/ambxst` dan
`~/.cache/quickshell`, cache QML runtime Quickshell yang dipakai bersama DMS dan
tidak dibutuhkan shell berikutnya. Syaratnya bukan "dipercaya", melainkan
"mustahil berisi data": `every_discard_path_is_never_preserved` menggagalkan
suite kalau ada irisan dengan jalur mana pun yang `backup_sources` jelajahi,
termasuk kasus di mana sebuah `discard_paths` jadi induk dari direktori yang
di-backup — itu akan menghapus tanpa syarat semua isinya, termasuk salinan yang
baru saja diambil. Guard pemilik tetap berlaku: path yang dimiliki shell tujuan
tidak disentuh.

Launcher tidak dibersihkan: kedua shell memasangnya dari skel masing-masing,
jadi shell berikutnya sudah menimpanya sendiri.

Daftar keenam ikut dibersihkan **dan** ikut di-backup, `companion_paths`: data
milik program pendamping shell, dan data yang shell itu simpan sendiri di luar
`config_dir`. DMS berjalan bersama `dankcalendar-git`, yang punya
`~/.local/share/dankcal` (basis data kalendernya dan keyring) dan
`~/.local/state/dankcal`, dan `dgop`, yang punya `~/.config/dgop`. DMS 2.0 juga
menyimpan ulang ejaan pra-2.0-nya sebagai link ke direktori asli
(`.config/DankMaterialShell` dan `.local/state/DankMaterialShell`), dan keduanya
ikut didaftarkan di sini: di mesin yang migrasinya belum jalan dua jalur itu
adalah direktori aslinya, yang memegang satu-satunya salinan setelan, sedangkan di
mesin yang sudah migrasi keduanya cuma link yang tanpa pendaftaran ini akan
ditinggalkan menggantung setiap kali DMS ditinggalkan. Ambxst
menyimpan riwayat chat dan clipboard yang di-pin di `~/.local/share/ambxst`,
catatannya di `~/.local/share/ambxst-notes`, dan `axctl` menyimpan catatan
brightness-nya di `~/.config/axctl`. Tidak ada yang menginstal data ini: program
menulisnya sendiri pada pertama kali jalan, jadi backup-lah yang mengembalikannya
— kecuali config compositor yang Ambxst regenerasi di `.local/share/ambxst`, yang
ikut dised skel supaya login pertama sudah bisa memuatnya. Tanpa daftar ini,
meninggalkan shell berarti membawa kalender, chat atau catatan pergi bersama
paketnya tanpa satu salinan pun tertinggal.

Perlakuannya sama dengan `generated_paths`, bukan dengan `discard_paths`:
di-backup bersama shell itu, dihapus setelah switch hanya kalau backup switch
ini mereproduksinya byte untuk byte, dan dibaca balik oleh `plan` saat shell itu
pulang. Kalender bukan sesuatu yang bisa dibangun start berikutnya, jadi jalan
pintas "tidak ada data di sini" yang `discard_paths` izinkan tidak boleh
dipakai — dan memang dicegah: `every_companion_path_is_backed_up` menggagalkan
suite kalau ada jalur di daftar ini yang tidak dijelajahi `backup_sources`.
Guard pemilik tetap berlaku, termasuk untuk dua shell yang mendeklarasikan
companion path yang sama.

Dengan begitu daftar DMS untuk switch keluar berjumlah empat belas (delapan di
tabel plus enam companion); Noctalia dan Caelestia tanpa companion tetap
delapan, sedangkan Ambxst menambah tiga companion path (`.config/axctl`,
`.local/share/ambxst`, `.local/share/ambxst-notes`) jadi sebelas.

Yang membuat catatan chat dan kalender itu bisa dibuang dengan tenang adalah
`backup_sources_for`, bukan `backup_sources`. Yang kedua hanya menyimpan separuh
shell yang **ditinggalkan**; swap menimpa config dan state shell **tujuan**
juga — shell yang pernah ada di mesin ini punya config yang akan digantikan
skel. Tanpa jalur target di backup, config lama itu hanya bertahan sebagai
`.pre-*` yang tidak direproduksi backup mana pun, dan langkah 12 menyimpannya
selamanya. Maka backup sebuah switch berisi jalur shell keluar **dan** jalur
shell masuk: `config_dir`, `state_dir`, `skel_extra_paths`, `backup_extra_paths`,
`generated_paths` dan `companion_paths` milik target ditambahkan bila belum
tercakup daftar keluar. Salinannya tercatat atas nama shell keluar, jadi ia
menjaga byte-nya, bukan menghidupkan kembali config lama target lewat `plan` —
yang kembali saat berpindah balik adalah era yang ditinggalkan shell itu.

### Apa yang boleh ditimpa, dan apa yang tidak

- **Config milik shell itu sendiri boleh diganti penuh** setelah backup
  terverifikasi. `~/.config/<shell>` dan `~/.local/state/<shell>` milik shell
  tersebut, jadi menggantinya bukan merusak milik user.
- **`~/.config/hypr` juga diganti penuh, dari skel shell target.** Skel tiap
  shell membawa konfigurasi Hyprland miliknya sendiri yang lengkap — termasuk
  `config/autostart.lua` yang menjalankan shell itu. Tanpa penggantian penuh,
  config lama tetap akan menjalankan shell lama, logout membawa user kembali ke
  tempat semula, dan pesan suksesnya bohong. Konsekuensinya: backup
  `~/.config/hypr` bukan pelengkap. Setelah switch, salinan di backup adalah
  satu-satunya tempat config Hyprland milik user masih ada.
- **gtk/qt ikut diganti** karena menentukan tampilan seluruh program, bukan
  cuma shell. Efeknya lebih luas daripada shell, dan justru karena itu semua
  ikut di-backup dan tetap dibiarkan ada file aslinya untuk restore manual.

Aturan penting dari urutan itu:

- **Langkah 3 boleh menggagalkan switch.** Lebih baik batal daripada merusak.
- **Kalau backup gagal, switch tidak jalan.** Ini bukan peringatan, ini syarat.
- **Backup yang tidak bisa dibaca ulang dianggap tidak ada.** Switch tidak
  pernah lanjut dengan anggapan ada jalan pulang.
- **Backup tidak pernah dihapus, tapi juga tidak pernah menumpuk.** Namanya
  tanggal, jadi satu hari menghasilkan satu direktori — bukan satu per klik.
  Switch kedua pada hari yang sama menggantikan yang pertama, dan penggantian
  hanya boleh terjadi setelah salinan yang baru selesai dibaca ulang cocok
  dengan manifestnya. Yang lama dipindah ke nama tersembunyi dulu dan dihapus
  hanya setelah yang baru berada di tempatnya; kalau penggantian gagal di
  tengah, yang lama dikembalikan. Tidak pernah ada saat tidak ada backup.
  Menghapus backup tetap hanya kalau user sendiri yang memintanya.

## Boleh dan tidak boleh dilakukan picker

Batasnya bukan "tidak boleh menyentuh apa pun", tapi "boleh hanya setelah
syaratnya terpenuhi". Yang tidak punya syarat ada di tabel kedua.

| Boleh | Syaratnya |
|---|---|
| Memasang paket yang belum ada, lewat `pkexec` | Bukan `rum` yang dipanggil langsung. `pkexec` yang minta otorisasi ke polkit agent sistem, jadi password tidak pernah masuk ke proses picker. Kegagalan instalasi membatalkan switch, bukan melanjutkan setengah jalan. |
| Mengganti config milik shell itu sendiri | Hanya setelah backup tersimpan dan bisa dibaca ulang. Ganti, bukan menimpa buta: config lama ada di backup dan dikembalikan lewat copy-paste manual. |
| Mengganti `~/.config/hypr` | Selalu, tapi seluruh direktori sekaligus dari skel shell target, dan hanya setelah backup `~/.config/hypr` terbukti utuh. Tidak pernah menulis sebagian: config yang setengah berisi membuat Hyprland gagal start. |
| Mematikan autostart shell sebelumnya | Supaya tidak ada dua shell hidup. Langkah ini masih bisa di-rollback; file dan paket shell lama baru dibuang setelah verify, jauh di belakangnya. |
| Menghapus config dan state shell yang ditinggalkan | Setelah langkah 9 (verify) berhasil, **dan** hanya kalau backup untuk switch ini mereproduksi path itu byte untuk byte. Backup yang tidak cocok lagi berarti shell sempat menulis sesuatu di antaranya — file disimpan, alasannya dilaporkan. Tidak pernah di depan backup. |
| Menghapus paket shell yang ditinggalkan | Lewat `pkexec rum remove -y`, hanya paket milik shell itu sendiri, dan hanya setelah shell baru terbukti hidup. Kegagalan tidak membatalkan switch yang sudah sukses: yang terjadi mesin lebih kotor dari yang diminta, bukan sesi yang rusak. |

| Tidak | Alasannya |
|---|---|
| Menulis sebagian dari `~/.config/hypr` | Filenya saling merujuk. Menyisipkan satu file ke direktori milik shell lain meninggalkan config yang setengah milik dua shell, dan gagalnya baru terasa saat Hyprland sudah start. Diganti penuh atau tidak disentuh. |
| `rm -rf` config tanpa backup yang terbukti tersimpan | Backup yang gagal membatalkan switch. Tidak ada kondisi di mana config hilang dan switch tetap jalan. Ini berlaku sama untuk langkah bersih: ukurannya bukan "backup ada", tapi "backup ini menggambarkan file ini". |
| Menghapus runtime bersama **yang masih dipakai shell tujuan** | `quickshell-git` dipakai shell lain juga — DMS, Caelestia dan Ambxst sama-sama berjalan di atasnya (masing-masing punya paket sendiri, tetapi runtime-nya tidak mereka bawa), dan `matugen` dipakai bersama DMS dan Ambxst — jadi mencabutnya berarti shell baru tanpa runtime dan tanpa cara mengembalikannya. `switch::uninstall` men-drop setiap paket yang ada di `runtime_packages` milik shell tujuan sebelum package manager melihatnya. Membalik arahnya: DMS → Noctalia **menghapus** quickshell, karena Noctalia biner asli dan tidak membutuhkannya. |

### Membersihkan shell yang ditinggalkan

Tiga hal, semuanya setelah langkah 9 berhasil, dan semuanya dilaporkan bukan
digagalkan — switch yang sudah sampai di sini sudah sukses, jadi berantakan
bukan alasan untuk membongkarnya kembali:

1. **`.pre-*`** yang tadi disingkirkan, dibuang (`discard_asides`). Hanya yang
   reproduksinya cocok dengan **backup mana pun**; yang tidak, disimpan. Bukan
   "backup terbaru": pertanyaannya satu-satunya apakah byte-nya masih aman di
   suatu tempat, dan menuntut direktori terbaru justru menyisakan salinan yang
   sudah terverifikasi sebagai `.pre-*` yang tidak akan pernah dibaca lagi —
   direktori terbaru saat itu milik shell yang baru masuk, asidenya dari shell
   sebelumnya.
2. **config dan state shell keluar** (`config_dir` + `state_dir`) dihapus.
   Syaratnya ketat: backup untuk switch ini harus berisi path itu dengan jumlah
   file dan byte yang sama. Tidak cocok, tidak dihapus. Karena dua field itu
   juga daftar backup dan daftar pemulihan, tidak ada daftar ketiga yang bisa
   berbeda — menambah `state_dir` otomatis menambah backup, bersih, dan
   pemulihan sekaligus.
3. **paket shell keluar**, kalau shell itu punya `uninstall_plan`, lewat
   `pkexec rum remove -y`.

Satu syarat berlaku untuk ketiganya, terutama untuk poin 2: **tidak ada
penghapusan yang boleh menyentuh backup.** `clean` menolak path yang berada di
dalam `~/.local/share/backups/` maupun path yang mengandungnya — `.local/share`
yang salah deklarasi akan membawa seluruh recovery copy pergi bersama
config-nya, dan pemeriksaan jumlah byte tidak menolaknya karena backup memang
berisi dirinya sendiri. Ada dua penjaga untuk ini: `clean` menolak
beroperasi pada path seperti itu (`refuses ...; it reaches the backups`), dan
test `no_declared_path_can_reach_the_backups` menggagalkan suite begitu ada
deklarasi yang berada terlalu tinggi di pohon direktori.

`state_dir` ikut dipulihkan dari backup oleh `plan` persis seperti
`config_dir`. Tanpa itu, Noctalia akan kehilangan 389 dari 445 file miliknya
begitu langkah 2 dijalankan: tersimpan di backup, tidak pernah dibaca balik,
dan selama ini selamat hanya karena tidak ada yang menghapusnya.

Untuk Noctalia `uninstall_plan`-nya `None`. Dia ada di base image yang
read-only, `rum remove` akan menolaknya, dan rencana yang selalu gagal lebih
buruk daripada tidak ada. Config dan statenya tetap dibersihkan — itu file
user, bukan file image.

Switch-back tetap mungkin setelah bersih: `install_plan` memasang paketnya
kembali, dan `plan` membaca config serta state dari backup yang sama.

## Skel: apa yang diganti penuh

Skel tiap shell membawa konfigurasi lengkap, bukan potongan. Semuanya dipasang
dengan cara yang sama — direktori lama disimpan dulu sebagai `.pre-*`, yang baru
dimasukkan, dan `.pre-*` dibuang lagi setelah shell baru terbukti hidup:

| Dipasang penuh | Isinya |
|---|---|
| `~/.config/<shell>/` | config milik shell itu sendiri |
| `~/.config/hypr/` | konfigurasi Hyprland milik shell target, termasuk `config/autostart.lua` yang menjalankannya |
| `~/.config/gtk-3.0`, `gtk-4.0`, `qt5ct`, `qt6ct` | tampilan, dipilih shell lewat konfigurasinya |

`~/.config/hypr/` bukan pengecualian lagi. Dulu picker menulis satu file sendiri
ke sana dan membiarkan Lua milik user; sekarang direktorinya diganti utuh,
karena `config/autostart.lua` di dalamnya adalah yang benar-benar menjalankan
shell. Config lama akan terus menjalankan shell lama, dan kalau itu dibiarkan,
logout setelah "sukses" akan membawa user persis kembali ke tempat semula.

Yang membuat penggantian itu aman bukan harapan bahwa user tidak punya isi di
sana, melainkan urutan di depannya: `~/.config/hypr` ikut di-backup pada langkah
3, dibaca ulang, dan baru diganti pada langkah 6. Setelah switch, salinan di
backup adalah satu-satunya tempat config Hyprland milik user masih ada — jadi
backup itu bukan pelengkap, dan menghapusnya berarti menghapus config sendiri.

`uwsm` tidak berperan dalam **menjalankan shell**, dan memang tidak pernah.
Rantai start-nya `start-hyprland` -> `Hyprland` -> `noctalia`: UWSM menjalankan
kompositor lalu turun, tidak ada proses uwsm yang menaungi shell. Yang
menjalankan shell adalah baris `hl.exec_cmd("noctalia")` di
`config/autostart.lua` — ada di skel, diinstal utuh bersama direktorinya. (UWSM
tetap muncul di **logout** sebagai salah satu jalur `uwsm stop`, lihat
`apply::logout`; itu soal mengakhiri sesi, bukan menjalankan shell.)
`~/.config/uwsm` lagi tidak per-desktop sama sekali —
bagian per-desktopnya ada di sistem, `/usr/share/uwsm/plugins/<desktop>.sh`,
yang tidak pernah kita sentuh; bagian user-nya satu skrip `env` global yang
dipakai bersama semua sesi. Jadi isinya akan identik untuk setiap kompositor,
yaitu bukan urusan ganti shell, cuma risiko merusak file milik desktop lain.

Bentuk direktori:

```
src/compositor/hyprland/shells/
  noctalia/
    mod.rs           -> apa yang ditulis switch untuk Noctalia di Hyprland
    skel/            -> file yang dipasang switch
  dms/
    mod.rs
    skel/
```

Skel ada di dalam folder shell-nya, bukan di folder `skel/` terpisah, supaya
satu shell di satu compositor utuh di satu tempat: `mod.rs` menyatakan ke mana
file-nya ditulis dan mana yang perlu diselamatkan, `skel/` berisi file aslinya.
Tidak perlu mencari dua tempat untuk satu shell.

Tidak semua shell punya default config untuk disalin. Noctalia tidak mengirim
satu pun, dia membuat `~/.config/noctalia/config.toml` sendiri saat pertama
jalan. Karena itu tiap shell perlu ditandai: salin dari skel, atau biarkan
shell yang membuat default-nya sendiri.

## Apa yang wajib diselamatkan, dan siapa yang mengatakannya

Switch menulis tiga kelompok path, dan ketiganya harus punya backup:

| Yang ditulis switch | Dizukan dari |
|---|---|
| config compositor | `Compositor::config_dir` |
| `config_dir` + `state_dir` shell | `Port`, otomatis |
| `skel_extra_paths` | `Port::skel_extra_paths` |
| `backup_extra_paths` | `Port::backup_extra_paths`, per shell |

Satu daftar di luar tabel: `companion_paths` (`Identity`), yang bukan ditulis
switch melainkan oleh program pendamping saat pertama kali jalan. Tetap wajib
disimpan dengan alasan yang sama — tidak ada jalan kembali setelah paketnya
ikut dihapus — dan dikunci oleh `every_companion_path_is_backed_up`.

Dua kelompok terakhir yang paling mudah terlewat. `skel_extra_paths` adalah daftar file yang
ditimpa, jadi setiap isinya wajib ada di backup. Dulu daftar backup diturunkan
dari `config_dir` dan `state_dir` saja, sehingga tiga file ditulis tapi tidak
diselamatkan: launcher `.desktop` milik Noctalia, dan direktori state DMS. Switch
bisa menghapus salinan user tanpa ada jalan kembali — persis hasil yang aturan 2
minta untuk dicegah.

Sekarang tiap shell mendeklarasikan `backup_extra_paths` sendiri, dan test
`every_installed_path_is_backed_up` mengunci hubungan itu: menambahkan path ke
`skel_extra_paths` tanpa menambahkannya ke backup akan menggagalkan test, bukan
mengirim switch yang tidak bisa dibatalkan.

Bedanya antara "dipasang" dan "perlu disimpan" sengaja tidak disamakan. DMS
memasang `dev.noctalia.Noctalia.desktop` untuk menyembunyikan launcher Noctalia,
tetapi file itu milik Noctalia — jadi yang menyimpannya adalah shell yang
memilikinya, bukan yang menutupnya.

## Me-restart compositor bukan cara ganti shell

Compositor tidak mengenal shell. `hyprctl reload` hanya membaca ulang
konfigurasi; tidak menjalankan shell apa pun. Jadi "reload lalu otomatis
berganti" bukan mekanisme yang benar — digantungkan begitu saja, tidak akan
pernah terjadi.

Shell harus dihentikan dan dijalankan ulang oleh picker sendiri, sementara
compositor dibiarkan terus berjalan. Ini aturan 2 di atas, dan tidak ada
pengecualian untuk hal ini.

## Lokasi backup

```
# $XDG_DATA_HOME/backups/rakuos-shell-picker/, yaitu ~/.local/share/... di sini
# BUKAN $XDG_STATE_HOME. State adalah tempat untuk bendahara yang tidak penting
# dan aman dihapus antar-run; backup justru bendahara yang paling perlu selamat,
# jadi diletakkan di share. Kalau tidak, satu "rm -rf ~/.local/state/*" untuk
# bersihin cache akan ikut menghapus seluruh recovery copy.
```
~/.local/share/backups/rakuos-shell-picker/
  20261006-hyprland-noctalia/
    manifest.toml          isi tiap path, jumlah file, ukuran, symlink
    home/                  salinan, mirroring path asli relatif terhadap $HOME
  20261005-hyprland-dms/
    ...
```

Namanya `YYYYMMDD-<compositor>-<shell>` — **tanggal saja, tanpa jam**. Suffix
shell di belakang itu penting dan tidak boleh dilepas: kalau namanya cuma
tanggal, backup shell kedua pada hari yang sama akan menghapus satu-satunya
salinan config shell pertama. Satu hari menghasilkan satu direktori per shell
yang ditinggalkan, jadi switch berulang pada hari yang sama menggantikan yang
sudah ada, bukan menambah — dan penggantian hanya terjadi setelah salinan yang
baru selesai dan cocok dengan manifestnya.

Backup bersifat **tambahan**, bukan pengganti: config asli tetap di tempat
asalnya. Menyalin, bukan memindahkan.

File `.pre-*` yang dibuat saat mengganti config tidak ikut backup, dan dibuang
begitu shell baru terbukti hidup. Mereka hanya ada untuk rollback yang sudah
tidak mungkin lagi terjadi pada saat itu.

## Isi `manifest.toml`

Minimum ini, supaya rollback bisa memutuskan sendiri apa yang harus dilakukan:

```toml
from_shell = "noctalia"
from_compositor = "hyprland"
to_shell = "dms"
to_compositor = "hyprland"
started_at = "2026-10-02T18:30:45Z"   # RFC 3339, UTC
outcome = "started"                    # started | failed | rolled_back
app_version = "1.0.2"
```

## Rollback

Rollback wajib kalau: proses target tidak hidup dalam batas waktu, atau target
langsung mati setelah start.

Rollback berarti: kembalikan backup ke tempat asalnya, start lagi shell lama,
tandai entri backup sebagai `rolled_back`, catat kejadian di log. Membatalkan
rollback butuh alasan yang eksplisit dari user.

## Logging saat gagal

Detail kegagalan ditulis ke file terpisah, bukan ke panel UI supaya tetap bisa
dibaca setelah app ditutup. Aturan lengkapnya ada di [LOGGING.md](LOGGING.md).

Yang dicatat: `preflight`, `backup`, `stop`, `start`, `verify`, `state`.
Yang **tidak** dicatat sebagai kegagalan: guard yang ditolak dari awal, dan
shell yang memang belum terpasang — keduanya sudah terlihat lewat badge di card.

## State file

`~/.local/state/rakuos/shell-picker/state.toml`

- Ditulis secara atomik: tulis file sementara di direktori yang sama, lalu
  `rename`. `fs::write` langsung ke `state.toml` bisa meninggalkan file
  setengah tertulis kalau supply daya hilang di tengah jalan.
- Isinya hanya id shell & compositor. **Tidak pernah** berisi isi konfigurasi.
- Rusak tidak boleh jadi kegagalan fatal: state yang tidak terbaca diperlakukan
  sebagai state kosong, dengan warning di log.

## Yang perlu ditambahkan sebelum backend hidup

- [ ] Test `detect_compositor()` untuk tiap compositor: envvar terisi, kosong,
      dan socket yang tidak bisa dibuka.
- [ ] Test penolakan guard: compositor terdeteksi A, lalu backend untuk B
      dipanggil — harus menolak, dan tidak boleh menyentuh apa pun.
- [ ] Test `manifest.toml`: isi tiap path, jumlah file, ukuran, symlink.
- [ ] Test yang membuktikan config asli masih utuh setelah backup.
- [ ] Test bahwa backup yang rusak ditolak: hapus satu file dari salinan, lalu
      `verify()` harus menolak dan menyebut file yang hilang.
- [ ] Test rollback dengan target yang dijamin gagal start.
- [ ] Deteksi "shell lain yang jalan di luar daftar kita" sebelum switch, supaya
      picker tidak diam-diam menyalakan shell kedua.
- [ ] Batas waktu untuk stop/start, supaya tidak menggantung selamanya.
- [ ] Test rotasi log gagal: lebih dari 10 file, yang tertua dipangkas, backup
      konfigurasi **tidak** ikut terpangkas.
