# Arsitektur

## Peta modul

```
src/main.rs                 hanya memanggil app::run()
src/app.rs                  aplikasi GTK/libadwaita, window, CSS
src/lib.rs                  daftar modul publik
src/tests/                  semua test, dipisah dari file produksi
                            (lihat "Di mana testnya" di docs/DEVELOPMENT.md)

src/ui/                     widget — tidak tahu shell, tidak tahu compositor
  mod.rs                    menyusun halaman, memegang Selection
  tabbar.rs                 satu tab per compositor
  grid.rs                   grid kartu untuk satu compositor + placeholder
  card.rs                   satu kartu shell
  logo.rs                   pemuatan SVG + kotak logo
  log.rs                    panel log

src/shells/                 satu file per shell — apa shell itu, di compositor mana pun
  mod.rs                    daftar semua identity (untuk test)
  noctalia.rs               IDENTITY: id, nama, upstream, paket, install plan,
                            uninstall plan, logo
  dms.rs
  caelestia.rs
  ambxst.rs
  end4pc.rs                 IDENTITY: paket, plan, probe; skel di port Hyprland
  inir.rs

src/compositor/             satu folder per compositor
  mod.rs                    trait Compositor + registry tab
  hyprland/
    mod.rs                  alat khusus Hyprland (hyprctl, window rule)
    shells/                 satu folder per shell — shell itu DI HYPRLAND
      mod.rs                urutan kartu
      noctalia/
        mod.rs              PORT: ke mana ditulis, apa yang diselamatkan,
                            apa yang dibersihkan saat shell keluar
        skel/               file yang dipasang switch (.config/hypr, gtk, qt6ct,
                            .config/noctalia, launcher)
      dms/
        mod.rs
        skel/
      caelestia/mod.rs + skel    skel/.config/hypr milik upstream Caelestia
      ambxst/
        mod.rs              PORT: config `.config/ambxst`, state `.local/state/ambxst`
        skel/               config Hyprland, qt6ct, data .local/share/ambxst,
                            launcher (Noctalia, NoDisplay)
      end4pc/mod.rs + skel    skel: dots ii (`qsConfig=end4-pC`); config: symlink
                            `~/.config/quickshell/end4-pC` ke payload RPM;
                            `custom/`: bootstrap venv + default tampilan
  niri/
    mod.rs                  alat khusus Niri (niri msg action load-config-file/quit -s)
    shells/                 satu folder per shell — shell itu DI NIRI
      mod.rs                urutan kartu
      dms/mod.rs + skel/
      noctalia/mod.rs + skel/
      ambxst/mod.rs + skel/
      inir/mod.rs + skel/    PORT: unit systemd `inir` di-aktifkan lewat symlink
                            `niri.service.wants/` (unit-nya tanpa `[Install]`);
                            skel: `defaults/niri` upstream + launcher Noctalia
                            (NoDisplay) + seed config (tema ikon Colloid-Dark);
                            config `.config/illogical-impulse` +
                            state `.local/state/inir`
  fake.rs                   compositor kedua untuk test, hanya di build test

src/shell.rs                tipe Identity + Port + Shell + InstallPlan +
                            UninstallPlan, dan cara memeriksa satu shell
src/session.rs              state active/previous di XDG
```

## Kenapa shell dipecah dua

Dulu setiap `(compositor, shell)` punya satu `Shell` penuh, jadi data yang
benar-benar sama ditulis dua kali: `upstream`, `package`, `install_plan`, dan
`companion_systemd_user_units` ulang untuk satu shell di dua compositor.
Tidak ada yang menolak compile kalau salah satu tidak sengaja diubah — dan
akibatnya bukan error kompilasi, melainkan satu tab memeriksa paket yang tab
lain memasang.

Sekarang `Identity` memegang yang benar-benar milik shell, dan `Port` hanya
yang berubah menurut compositor:

| Field | `Identity` | `Port` |
|---|---|
| `id`, `display_name`, `upstream` | `Identity` | — |
| `package`, `start_argv`, `logo` | `Identity` | — |
| `install_plan`, `uninstall_plan`, `companion_systemd_user_units`, `companion_paths`, `generated_paths`, `discard_paths`, `runtime_packages` | `Identity` | — |
| `config_dir`, `state_dir` | — | `Port` |
| `systemd_user_unit`, `mechanism` | — | `Port` |
| `systemd_user_unit_wanted_by` | — | `Port` |
| `skel_extra_paths`, `backup_extra_paths` | — | `Port` |

`Port` sengaja tidak punya field yang bisa berduplikasi, jadi menyalinnya ke
compositor lain tidak bisa gagal diam-diam. `Shell::join` adalah `const fn`,
jadi `Shell` yang dirakit tetap bisa dipakai di `static` tanpa biaya runtime.

Yang tetap di `switch.rs`, bukan diulang per shell: config bersama yang selalu
disimpan backup-nya, dan urutan langkah switch.

## Arah dependensi

```
app ──▶ ui ──▶ compositor ──▶ shell ◀── shells
                     │
                     └───▶ shell (untuk registry)
session ──▶ shell
```

- `ui/` tidak pernah mengimpor `compositor::hyprland::shells::*` — itu hanya
  boleh muncul di `compositor/mod.rs`.
- `shell.rs` **tidak** tahu shell apa yang ada. Daftar shell milik tiap
  compositor. `shell.rs` hanya berisi tipe dan cara menginspeksinya.
- `compositor/niri/` tidak boleh mengimpor dari `compositor/hyprland/`.
- `compositor/<a>/` tidak boleh mengimpor dari `compositor/<b>/` untuk `a != b`.
- `compositor/*/shells/*/mod.rs` boleh mengimpor `crate::shells::*`, dan tidak
  boleh mengimpor milik compositor lain.

## Test yang menjaga pemisahan ini

Di `src/tests/lib_registry.rs` (dideklarasikan dari `src/lib.rs` sebagai
`mod registry_tests`):

- `one_identity_serves_every_compositor_offering_that_shell` — membandingkan
  setiap field yang dimiliki `Identity` dengan yang muncul di tiap compositor.
  Karena `Port` tidak punya field itu, test ini hanya bisa gagal kalau ada yang
  menambahkannya kembali ke `Port`.
- `every_identity_is_offered_by_at_least_one_compositor` — menangkap file
  identity yang tidak terdaftar di compositor mana pun: ia compile, ia lolos
  semua test lain, dan ia tidak ada di kartu mana pun.
- `every_port_stays_inside_home` — semua path relatif, karena semuanya
  digabung ke `$HOME` lalu ditulis.
- `every_declared_skel_extra_path_really_is_in_the_skel` — `apply::plan`
  memperlakukan path yang hilang sebagai sekadar melewati, jadi path yang sudah
  di-rename dulunya hilang diam-diam.
- `every_installed_path_is_backed_up` — setiap file yang ditulis switch dari skel
  wajib punya backup. Ini yang menahan penambahan `skel_extra_paths` tanpa
  Declaration backup yang cocok.
- `every_generated_path_is_backed_up` — kebalikannya: setiap path yang `clean`
  hanya berani hapus kalau backup mereproduksinya byte-for-byte wajib berada
  di dalam direktori yang `backup_sources` jelajahi, atau tidak akan pernah
  dihapus sama sekali.
- `every_discard_path_is_never_preserved` — dan kebalikannya lagi: path yang
  dihapus **tanpa** syarat backup tidak boleh beririsan dengan jalur mana pun
  yang di-backup. Yang ada datanya dibackup, yang tidak ada datanya dibersihkan;
  test ini yang memastikan keduanya tidak pernah bertukar tempat.
- `every_file_in_a_skel_reaches_the_machine` — setiap file di folder skel
  ternyata punya jalur yang benar-benar ditulis `plan`, bukan cuma tercopy ke
  staging lalu dilupakan.

## Menambah shell

Dua folder, satu di global dan satu per compositor. Yang pertama menjawab
"shell ini dipasang dan dibersihkan bagaimana", yang kedua menjawab "di
compositor ini ditulis dan diselamatkan di mana".

```rust
// src/shells/nama_baru.rs — yang benar-benar milik shell, di mana pun
use crate::shell::{Identity, InstallPlan, UninstallPlan};

pub const IDENTITY: Identity = Identity {
    id: "nama_baru",
    display_name: "Nama Baru",
    upstream: "org/nama-baru",
    package: None,
    binary: "nama-baru",
    start_argv: &["nama-baru"],
    companion_systemd_user_units: &[],
    logo: Some("nama-baru.svg"),
    // Paketnya ada di repo? Isi. Kalau tidak, `None`.
    install_plan: None,
    // Paketnya boleh dilepas lagi? Isi. Base image selalu `None`.
    uninstall_plan: None,
};
```

`a_shell_that_can_be_installed_can_also_be_removed` menagih pasangan itu:
mengisi `install_plan` tanpa `uninstall_plan` membuat shell yang bisa dipasang
tapi tidak pernah bisa dibuang, dan switch berikutnya akan meninggalkannya.

Lalu satu folder per compositor yang menawarkannya, isinya lengkap dari install
sampai bersih:

```
src/compositor/hyprland/shells/nama_baru/
  mod.rs      PORT: ke mana ditulis, apa yang diselamatkan, apa yang dibersihkan
  skel/       file yang dipasang switch
```

```rust
// src/compositor/hyprland/shells/nama_baru/mod.rs
use crate::shell::{Port, Shell, StartMechanism};
use crate::shells::nama_baru::IDENTITY;

const PORT: Port = Port {
    config_dir: Some(".config/nama-baru"),
    state_dir: None,
    systemd_user_unit: None,
    // Hanya kalau unit-nya tidak punya `[Install]`: nama unit yang `.wants/`-nya
    // memuat symlink enable-nya (`"niri.service"`).
    systemd_user_unit_wanted_by: None,
    mechanism: StartMechanism::DirectProcess,
    skel_extra_paths: &[],
    backup_extra_paths: &[],
};

pub static SHELL: Shell = Shell::join(IDENTITY, PORT);
```

Daftarkan di `compositor/hyprland/shells/mod.rs`:

```rust
pub mod nama_baru;
// ...
pub static ALL: &[Shell] = &[..., nama_baru::SHELL];
```

Ulangi folder itu untuk setiap compositor lain yang menawarkannya.
`<compositor>/shells/nama_baru/mod.rs` akan berbeda di `mechanism` atau
`systemd_user_unit` kalau memang begitu, dan identik kalau memang sama.

Sembilan field yang sering tertukar:

- `skel_extra_paths` — file yang switch **timpa**. Tambah di sini berarti
  menimpa file milik user, jadi harus ada di `backup_extra_paths` atau
  `SHARED_PATHS`. `every_installed_path_is_backed_up` akan menggagalkan kalau
  tidak.
- `backup_extra_paths` — file milik shell ini yang switch hancurkan tanpa
  tercatat di `config_dir`/`state_dir`, dan yang tidak punya tempat lain untuk
  dibaca balik. Launcher Noctalia ada di sini karena DMS menimpanya.
- `companion_paths` — data yang disimpan di luar `config_dir`, baik oleh program
  pendamping shell (DMS: `.local/share/dankcal`, `.config/dgop`) maupun oleh
  shell itu sendiri (Ambxst: `.local/share/ambxst` untuk chat dan clipboard,
  `.local/share/ambxst-notes`, plus `.config/axctl` milik `axctl`). Datanya
  hanya ada di backup, tetapi skel boleh ikut menyeed config yang programnya
  regenerasi: Ambxst membawa `.local/share/ambxst/hyprland.lua` dan kawan-kawan
  supaya login pertama sudah punya file yang dimuat config compositor. `plan`
  membaca balik dari backup dengan skel sebagai fallback;
  `every_companion_path_is_backed_up` menahan penambahan jalur di luar apa yang
  `backup_sources` jelajahi, dan `every_file_in_a_skel_reaches_the_machine`
  menahan file skel yang tidak tercakup.
- `generated_paths` — file yang **shell sendiri** tulis ke direktori bersama
  (tema yang dirender ke `gtk-3.0`, `gtk-4.0`, `qt5ct`, `qt6ct`). Bukan
  masukan switch, jadi tidak boleh ada di `skel_extra_paths`, tapi tetap ikut
  dibersihkan setelah switch supaya tidak meninggalkan tema yang tak terpakai
  di mesin. Ada di `Identity`, bukan `Port`, karena sifatnya milik shell.
  `every_generated_path_is_backed_up` menggagalkan kalau jalurnya berada di
  luar semua direktori yang `backup_sources` jelajahi — tanpa itu `clean`
  tidak akan pernah berani menghapusnya.
- `config_dir` dan `state_dir` — bukan sekadar lokasi. Bersama
  `backup_extra_paths`, keduanya adalah daftar backup; bersama
  `generated_paths`, keduanya adalah daftar yang **dibersihkan** setelah
  switch. `plan` juga membacanya kembali saat shell itu dipilih lagi. Mengisi
  `state_dir` otomatis menambah semuanya; membuat daftar bersih terpisah hanya
  menambah tempat yang bisa berbeda.

Lalu yang memisahkan kedua daftar bersih itu — aturan satu kalimat: **yang ada
datanya dibackup, yang tidak ada datanya dibersihkan.**

- `discard_paths` — cache dan file render yang **tidak pernah diedit siapa
  pun** (`.cache/dms`, `.cache/dsearch`, `themes/dankcolors`, skema warna
  `DankMatugen*`). `clean` menghapusnya tanpa memeriksa backup, karena memang
  tidak ada yang perlu diselamatkan; backup pun tidak membawanya, supaya satu
  switch tidak memindahkan beberapa megabyte ulangan yang bisa diregenerasi.
  `every_discard_path_is_never_preserved` menggagalkan kalau ada irisan dengan
  jalur mana pun yang `backup_sources` jelajahi — kalau irisan itu ada, "tidak
  ada yang bisa hilang" berubah dari sifat jadi harapan.

Benturan dua aturan ini di dua arah yang berbeda adalah disengaja:
`generated_paths` harus berada **di dalam** jalur backup supaya boleh dihapus,
`discard_paths` harus berada **di luar** supaya boleh dihapus tanpa syarat.

- `runtime_packages` — paket yang shell ini **dijalankan di atas**, bukan paket
  yang dirinya sendiri. Ini penjaga yang membuat `uninstall_plan` boleh berisi
  lebih dari paket milik shell yang pergi: `switch::uninstall` men-drop setiap
  paket yang ada di daftar ini milik **shell yang sedang dinyalakan**, sebelum
  package manager melihatnya. Caelestia, Ambxst dan DMS semuanya berjalan di
  atas `quickshell` — masing-masing punya paketnya sendiri, tetapi runtime itu
  tidak mereka bawa — jadi tanpa guard ini arah DMS → Caelestia/Ambxst akan
  mencabut runtime shell baru. DMS dan Ambxst juga berbagi `matugen`, jadi
  berpindah antara keduanya tidak mencabutnya. Sebaliknya, Noctalia kosong di
  daftar ini —
  itulah yang membuat arah DMS → Noctalia justru menghapus `quickshell-git`.
  Salah isi hanya bisa berakibat paket **ditahan**, bukan shell rusak, jadi
  arah yang aman untuk salah adalah mendeklarasikannya.
- `autostart_systemd_user_units` dan `polkit_agent` — dua deklarasi yang membuat
  shell dibawa masuk dan keluar dengan utuh. Yang pertama: unit systemd yang
  **ikut hidup/mati bersama shell**, dengan transisi yang sama seperti unit
  shell-nya sendiri — keluar: `stop` + `disable` + `mask`; masuk: `unmask` +
  `enable` + `start`. DMS mendeklarasikan `dsearch`, unit dari paket
  `danksearch`; jangan tertukar dengan `companion_systemd_user_units`, yang
  hanya di-stop/start dan tidak pernah di-disable/mask karena tidak ada yang
  meng-enable-nya. Yang kedua: argv agen polkit yang harus dijalankan switch
  **sebelum** tahap `pkexec rum remove` — Caelestia dan Ambxst menjawab polkit
  lewat `/usr/libexec/polkit-mate-authentication-agent-1`; DMS, Noctalia,
  end4-pC dan iNiR menjawab polkit sendiri dan kosong di sini.
- `systemd_user_unit_wanted_by` — nama unit yang `.wants/`-nya memuat symlink
  enable, untuk unit yang **tidak punya `[Install]`**. `systemctl --user enable`
  menolak unit seperti itu, jadi `apply` menulis sendiri link yang seharusnya
  ditulis `enable` (`start`: link + `daemon-reload`; `stop_unit`: link itu
  dihapus, tanpa `disable` — tidak ada `[Install]` untuk dibatalkan — dan tanpa
  `mask`, karena upstream punya perintah `inir service enable` sendiri yang akan
  menulis link yang sama). iNiR satu-satunya yang memakainya hari ini, dengan
  `niri.service`: unit `inir.service` memang dikirim apa adanya dari upstream,
  yang meng-enable-nya dengan symlink seperti itu. Target-nya di `Port` karena
  jawabannya unit milik compositor.

Lihat [SWITCH-FLOW.md](SWITCH-FLOW.md) untuk urutan lengkap kedua arah beserta
daftar file yang masing-masing langkah sentuh.

Test `every_registered_shell_has_its_logo_on_disk` gagal kalau SVG-nya lupa
ditaruh di `assets/`.

## Menambah compositor

```rust
// src/compositor/sway/mod.rs
use crate::compositor::Compositor;
use crate::shell::Shell;

pub struct Sway;
impl Compositor for Sway {
    fn id(&self) -> &'static str { "sway" }
    fn label(&self) -> &'static str { "sway" }
    fn shells(&self) -> &'static [Shell] { &[] }  // placeholder dulu
}
```

Lalu daftarkan di `compositor/mod.rs`:

```rust
pub mod sway;
pub static SWAY: sway::Sway = sway::Sway;

pub fn all() -> Vec<&'static dyn Compositor> {
    vec![&HYPRLAND, &SWAY]
}
```

Compositor dengan daftar kosong otomatis dapat halaman "Coming soon". Setelah
daftarnya tidak kosong, tambahkan satu folder per shell di `sway/shells/<shell>/`
— `mod.rs` berisi `Port`-nya sendiri, `skel/` berisi file yangpasang —
dengan `Identity` yang sudah ada di `src/shells/` tidak disalin.

**Skel harus ada sebelum mendaftarkan compositornya.** Niri pernah terdaftar
dengan tiga port dan tanpa satu pun `skel/`: tabnya menampilkan tiga kartu dan
tidak satu pun bisa diselesaikan. Switch ditolak `apply::plan` dengan pesan
bahwa config compositor-nya belum ada — jujur soal penyebab, tapi membuat picker
tampak rusak. Sekarang `a_skel_is_missing_only_where_the_shell_cannot_be_installed`
menangkapnya: mendaftarkan compositor yang punya skel kosong adalah test yang
gagal, bukan tab berisi penolakan. Niri terdaftar kembali setelah skel-nya ada —
`shells/dms/`, `shells/noctalia/` dan `shells/ambxst/` masing-masing membawa
`skel/`.

## Konvensi nama

| Hal | Nilai |
|---|---|
| Id compositor | huruf kecil, nama compositor asli (`hyprland`, `sway`) |
| Id shell | huruf kecil, nama upstream (`noctalia`, `dms`) |
| Konstanta layout | di file `ui/` yang memakainya, bukan di root |
| File statis | `UPPER_SNAKE` untuk `Shell`, `IDENTITY` untuk `Identity`, `UPPER_SNAKE` untuk `Port` |
| Letak skel | `compositor/<c>/shells/<shell>/skel/`, di dalam folder shell |

## Test

Ada unit test tanpa display server:

```bash
cargo test
```

Yang dijaga: id compositor unik, id shell unik lintas compositor, setiap shell
punya logo yang benar-benar ada di disk, urutan kartu tiap compositor, setiap
identity dipakai oleh minimal satu compositor, field `Identity` tidak pernah
berbeda antar compositor, semua path tetap di dalam `$HOME`, setiap
`skel_extra_paths` benar-benar ada di skel compositor itu, roundtrip state
session, dan collision nama backup.

Dua test membaca `$HOME` asli dan ditandai `#[ignore]`, supaya `cargo test`
tidak pernah menyalin konfigurasi desktop tanpa sengaja. Semuanya hijau,
termasuk keduanya:

```bash
cargo test -- --include-ignored
```

Jangan comorbidkan yang ini. Salah satunya yang menemukan bug file tunggal: unit
test lain semua memakai direktori sebagai sumber, sementara launcher Noctalia
adalah sebuah file, jadi tidak ada yang menutupinya sampai test ini dijalankan
di desktop sungguhan.

Sebelum commit, `cargo fmt --check`, `cargo clippy --all-targets`, dan
`cargo test` harus bersih.

Suite ini pernah punya test yang gagal 13 dari 15 kali karena nama directory
backup berbasis milidetik. Jadi kalau sebuah test gagal tanpa sebab yang jelas,
periksa dulu memang tidak _flaky_: jalankan beberapa kali. Kalau konsisten gagal,
coba cari test yang tidak pernah jalan dalam keadaan nyata. Dua bug terbaru di
sini ketahuan begitu, dan keduanya ada di jalur yang paling jarang dilewati
orang.
