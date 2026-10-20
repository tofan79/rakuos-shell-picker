use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

use super::compositor;
use super::shells;
use crate::compositor::Compositor;

fn assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

#[test]
fn every_compositor_id_is_unique() {
    let ids: HashSet<_> = compositor::all().iter().map(|c| c.id()).collect();
    assert_eq!(ids.len(), compositor::all().len());
}

#[test]
fn every_compositor_has_a_unique_id_and_label() {
    for a in compositor::all() {
        assert!(!a.id().is_empty());
        assert!(!a.label().is_empty());
    }
}

#[test]
fn shell_ids_are_unique_within_a_compositor() {
    // Not across compositors: the same shell appears in more than one list
    // on purpose, because which shells exist is a property of the shell plus
    // the compositor. What must never happen is the same id twice on one
    // page, which would make two cards indistinguishable.
    for compositor in compositor::all() {
        let mut seen = HashSet::new();
        for shell in compositor.shells() {
            assert!(
                seen.insert(shell.id),
                "{}: duplicate shell id {}",
                compositor.id(),
                shell.id
            );
        }
    }
}

#[test]
fn a_shell_id_always_refers_to_the_same_shell() {
    // The id is what gets written to the session file, so the same id must
    // not mean two different upstreams.
    let mut upstream: HashMap<&str, &str> = HashMap::new();
    for compositor in compositor::all() {
        for shell in compositor.shells() {
            match upstream.get(shell.id) {
                Some(previous) => assert_eq!(
                    *previous,
                    shell.upstream,
                    "{}: id {} maps to two upstreams",
                    compositor.id(),
                    shell.id
                ),
                None => {
                    upstream.insert(shell.id, shell.upstream);
                }
            }
        }
    }
}

#[test]
fn every_registered_shell_has_its_logo_on_disk() {
    for compositor in compositor::all() {
        for shell in compositor.shells() {
            let file = shell
                .logo
                .unwrap_or_else(|| panic!("{} has no logo", shell.id));
            let path = assets().join(file);
            assert!(
                path.is_file(),
                "missing logo for {}: {}",
                shell.id,
                path.display()
            );
        }
    }
}

#[test]
fn each_compositor_lists_the_shells_it_really_supports() {
    let hyprland = compositor::all()
        .into_iter()
        .find(|c| c.id() == "hyprland")
        .expect("hyprland registered");
    assert_eq!(
        hyprland.shells().iter().map(|s| s.id).collect::<Vec<_>>(),
        ["noctalia", "dms", "caelestia", "ambxst", "end4pc"]
    );

    // Niri's four: DMS from its base image, Noctalia and Ambxst from the
    // repository, and iNiR whose card exists ahead of its backend. Caelestia
    // and end4-pC are Hyprland shells upstream, so both are absent — a shell
    // missing for a compositor is exactly what the per-compositor registry
    // exists to express.
    let niri = compositor::all()
        .into_iter()
        .find(|c| c.id() == "niri")
        .expect("niri registered");
    assert_eq!(
        niri.shells().iter().map(|s| s.id).collect::<Vec<_>>(),
        ["dms", "noctalia", "ambxst", "inir"]
    );
}

#[test]
fn a_compositor_only_offers_shells_it_really_supports() {
    // The rule this states rather than exercises: a compositor's shell list is
    // reviewed against upstream when it is added, not derived here. Caelestia is
    // on Hyprland because upstream ships a Hyprland shell for it; it is not on
    // Niri because only third-party forks exist. Noctalia and DMS are on both,
    // each with its own port.
    assert_eq!(
        compositor::all().len(),
        2,
        "Hyprland and Niri are registered"
    );
}

#[test]
fn an_install_plan_names_the_same_package_as_the_shell() {
    // The availability check is done against Shell::package, and the install
    // is done with InstallPlan::package. If they drift apart the picker would
    // check one package and install another.
    for compositor in compositor::all() {
        for shell in compositor.shells() {
            if let Some(plan) = shell.install_plan {
                assert_eq!(
                    shell.package,
                    Some(plan.package),
                    "{}: install plan installs {} but the shell is checked as {:?}",
                    shell.id,
                    plan.package,
                    shell.package
                );
            }
        }
    }
}

#[test]
fn only_packaged_shells_can_be_installed_and_they_use_polkit() {
    // Every registered shell is packaged in the `rakuos` repository, including
    // Noctalia: it happens to be in the Hyprland image already, so its install
    // never runs there, but the same plan installs it on Niri. Each install goes
    // through polkit so the password never passes through this process.
    let with_plan: BTreeSet<_> = compositor::all()
        .iter()
        .flat_map(|c| c.shells())
        .filter(|s| s.install_plan.is_some())
        .map(|s| s.id)
        .collect();
    assert_eq!(
        with_plan,
        BTreeSet::from([
            "ambxst", "caelestia", "dms", "end4pc", "inir", "noctalia"
        ])
    );

    let dms = super::shell::find("dms").expect("dms registered");
    let plan = dms.install_plan.expect("dms installs from a repository");
    assert_eq!(
        plan.command_line(),
        "pkexec rum install -y --refresh quickshell-git dms dms-cli dgop \
         danksearch dankcalendar-git matugen"
    );
    // The password must go to the polkit agent, never through this process.
    assert_eq!(plan.program, "pkexec");
    // quickshell-git is named so the solver cannot hand back noctalia-qs,
    // which is the legacy Noctalia build and not a dependency of anything
    // current.
    assert!(!plan.command_line().contains("noctalia-qs"));

    let caelestia = super::shell::find("caelestia").expect("caelestia registered");
    let plan = caelestia
        .install_plan
        .expect("caelestia installs from a repository");
    // Its own package, the runtime named in a stage of its own (mirroring
    // DMS), the three fonts nothing requires at the package level — without
    // which the shell would start with blank glyphs — and the two tools
    // nothing else pulls in either: the polkit agent the config starts and
    // the annotate step behind the area picker's screenshot action.
    assert_eq!(
        plan.command_line(),
        "pkexec rum install -y --refresh quickshell-git caelestia-shell \
         google-rubik-fonts cascadia-code-nf-fonts material-symbols-fonts mate-polkit \
         swappy"
    );
    // The shell's own package must be among the packages it installs, or the
    // availability check and the install would drift apart.
    assert!(plan.packages().contains(&plan.package));
    assert_eq!(plan.program, "pkexec");

    let ambxst = super::shell::find("ambxst").expect("ambxst registered");
    let plan = ambxst
        .install_plan
        .expect("ambxst installs from a repository");
    // Its own package, the runtime in a stage of its own, and the six things
    // nothing requires: the companion daemon, the theming tool, the icon fonts
    // the shell draws with, the polkit agent its config starts, the audio
    // effects daemon and the recorder its screen-record bind runs.
    assert_eq!(
        plan.command_line(),
        "pkexec rum install -y --refresh quickshell-git ambxst axctl matugen \
         phosphor-icons-fonts mate-polkit easyeffects gpu-screen-recorder"
    );
    assert!(plan.packages().contains(&plan.package));
    assert_eq!(plan.program, "pkexec");

    let end4pc = super::shell::find("end4pc").expect("end4pc registered");
    let plan = end4pc
        .install_plan
        .expect("end4pc installs from a repository");
    // Its own package, the runtime in a stage of its own, and the base `ii` it
    // grew from — `end4-pc` requires it, so naming it changes nothing at
    // install time; it is named so the uninstall plan can take the base back
    // out when end4-pC leaves. Then everything the skel or the QML runs that
    // nothing requires: the theming tool its colour scripts drive; the idle
    // daemon and lock screen behind `loginctl lock-session`; the audio effects
    // daemon its equalizer drives; the night light its toggles spawn; the
    // recorder and annotate tool behind its capture keybinds; the fallback menu
    // its keybinds use when the shell is not around; the three fonts its
    // interface is drawn with; and `uv` with the four `-devel` packages, which
    // bootstrap the Python venv (`pycairo` and `pygobject` compile against
    // `cairo-devel` and `gobject-introspection-devel`, `pygobject`'s cairo
    // binding includes `cairo-gobject.h`, and `dbus-python` compiles against the
    // `dbus-1` headers). No `plasma-integration`: it drags in all of
    // plasma-workspace (127 packages), and `QT_QPA_PLATFORMTHEME=qt6ct` reads the
    // icon theme from qt6ct.conf just as well. No polkit agent: the fork
    // answers polkit from its own
    // `PolkitService`.
    assert_eq!(
        plan.command_line(),
        "pkexec rum install -y --refresh quickshell-git end4-pc illogical-impulse \
         matugen hypridle hyprlock google-rubik-fonts cascadia-code-nf-fonts \
         material-symbols-fonts easyeffects hyprsunset wf-recorder swappy fuzzel \
         uv cairo-devel cairo-gobject-devel gobject-introspection-devel dbus-devel"
    );
    assert!(plan.packages().contains(&plan.package));
    assert_eq!(plan.program, "pkexec");

    let noctalia = super::shell::find("noctalia").expect("noctalia registered");
    let plan = noctalia
        .install_plan
        .expect("noctalia installs from the repository");
    // One package and nothing else: Noctalia is a native binary that declares
    // what it needs, unlike the quickshell shells, so there is no runtime or font
    // to name beside it.
    assert_eq!(
        plan.command_line(),
        "pkexec rum install -y --refresh noctalia"
    );
    assert_eq!(plan.program, "pkexec");

    let inir = super::shell::find("inir").expect("inir registered");
    let plan = inir.install_plan.expect("inir installs from a repository");
    // One stage, and only what the niri image does not already bring: the
    // runtime, `qt6ct`, `wl-clipboard`, `ddcutil` and `matugen` are the image's,
    // and `swaylock` is one the image removes. What is left is the shell plus
    // every tool the skel or the QML runs that nothing requires at the package
    // level: the QML and Qt pieces its interface imports, the style its skel
    // overrides with, the platform theme beside it, the sycoca builder, the
    // clipboard store the overlay reads, the capture tools behind its screenshot
    // and record keys, the image probe it sizes a wallpaper with, OCR, the
    // panel's brightness backend, the idle daemon, the night light, the Go
    // toolchain its theme modules build with, the Python toolchain and the
    // headers its venv's C extensions are built against, the icon font its
    // interface is drawn with, and the two families its doctor counts as
    // important — `Gabarito` and `Oxanium`, which no repository RakuOS enables
    // carries and which upstream otherwise downloads per user. The families the
    // skel *seeds* are a different statement: those are the image's, so the
    // config points at them (see `inir_names_the_theme_and_fonts_this_machine_has`).
    // No polkit agent: iNiR answers polkit from its own.
    assert_eq!(
        plan.command_line(),
        "pkexec rum install -y --refresh inir-shell qt6-qt5compat \
         qt6-qtmultimedia qt6-qtimageformats darkly plasma-integration \
         kf6-kservice cliphist grim slurp wf-recorder ImageMagick tesseract \
         brightnessctl swayidle wlsunset golang uv cairo-devel \
         cairo-gobject-devel gobject-introspection-devel dbus-devel \
         material-symbols-fonts gabarito-fonts oxanium-fonts"
    );
    assert!(plan.packages().contains(&plan.package));
    assert_eq!(plan.program, "pkexec");
    // The runtime the shell runs on, but does not install: the image has it and
    // protects it, and `runtime_packages` is where the uninstall guard reads it.
    assert!(
        !plan.packages().contains(&"quickshell-git"),
        "quickshell-git is the niri image's, so the plan neither installs nor \
         removes it"
    );
    assert!(inir.runtime_packages.contains(&"quickshell-git"));
}

#[test]
fn inir_names_the_theme_and_fonts_this_machine_has() {
    // iNiR's own defaults for `appearance.iconTheme` and the three
    // `appearance.typography` families are `WhiteSur-dark`, `Roboto Flex`,
    // `Gabarito` and `JetBrainsMono Nerd Font` — the first, second and last in no
    // repository RakuOS enables, and the third only because the install plan now
    // carries `gabarito-fonts` (the doctor counts that family as important; see
    // the plan test above). The shell renders the names it holds into gsettings and
    // into every Qt/GTK app's settings file (`gtk-3.0`/`gtk-4.0/settings.ini`,
    // `kdeglobals`, `qt6ct`), so a name that is not installed is a desktop of
    // missing icons — in iNiR, and in whichever shell comes after it, because
    // the icon theme is one dconf key and no switch replaces dconf — or, for a
    // font, a session where every toolkit falls back on its own. So the Niri skel
    // seeds the icon theme `rakuos-branding` installs (the same name DMS's and
    // Caelestia's skels render) and the sans and monospace the image already sets
    // system-wide, and states the icon theme at login as well.
    //
    // The seed is pinned whole rather than key by key: naming these names is the
    // entire point, and a JSON file is the one skel file nothing else in this
    // suite would catch going bad — `every_toml_in_a_skel_parses` parses the TOML
    // and the KDL is checked by hand, but a JSON the shell cannot read is a shell
    // that comes up without its configuration. Whitespace is normalised so a
    // reformat is not a failure; a key, a value or a brace that stops parsing is.
    let skel = crate::apply::skel_for("niri", "inir");
    let seed = std::fs::read_to_string(skel.join(".config/illogical-impulse/config.json"))
        .expect("the iNiR Niri skel seeds iNiR's own config");
    let compact: String = seed.chars().filter(|c| !c.is_whitespace()).collect();
    // Whitespace is stripped on both sides, so the multi-word font names appear
    // here without their spaces.
    assert_eq!(
        compact,
        r#"{"appearance":{"iconTheme":"Colloid-Dark","typography":{"mainFont":"Inter","titleFont":"Inter","monospaceFont":"MapleMonoNF"}}}"#,
        "the seeded names have to be the ones this image installs"
    );

    let startup = std::fs::read_to_string(skel.join(".config/niri/config.d/50-startup.kdl"))
        .expect("the iNiR Niri skel starts the session");
    assert!(
        startup.contains(
            r#"spawn-at-startup "gsettings" "set" "org.gnome.desktop.interface" "icon-theme" "Colloid-Dark""#
        ),
        "a session has to say which icon theme it uses, or GTK apps open on a \
         fallback"
    );
}

#[test]
fn inir_builds_its_python_environment_at_login() {
    // The palette iNiR generates from a wallpaper, the thumbnails and the AI
    // features run on upstream's Python requirements, which the package ships as
    // data: upstream builds the venv in its own installer, and nobody runs that
    // on a machine this picker set up. So the skel has to carry the step itself —
    // a script that builds the venv with `uv`, and a startup line that runs it
    // every login, idempotent — and the plan has to install the builder and the
    // headers upstream's requirements compile against, or the build stops at the
    // first one that cannot build and the palette generation never arrives
    // either. `cairo-gobject-devel` and `dbus-devel` are the entries only a live
    // bootstrap could find: `pygobject`'s cairo binding includes
    // `cairo-gobject.h`, and `dbus-python` — which `kde-material-you-colors`
    // pulls in — builds against `dbus-1.pc`; upstream's list installs in one
    // step, so either one missing holds the wheels back.
    //
    // Where the script sits is part of the test: it has to be in the compositor's
    // tree, not under the shell's `~/.config/illogical-impulse`. `apply::plan`
    // installs the compositor's directory from the skel on every switch, while a
    // shell's config directory comes from that shell's own backup whenever there
    // is one — so a script under it would never arrive on a machine that has run
    // iNiR before, and the startup line would call a file that is not there.
    use std::os::unix::fs::PermissionsExt;

    let skel = crate::apply::skel_for("niri", "inir");
    let script = std::fs::read_to_string(skel.join(".config/niri/scripts/inir-venv-setup.sh"))
        .expect("the iNiR Niri skel bootstraps the venv its scripts run in");
    assert!(
        script.contains("/usr/share/quickshell/inir/sdata/uv/requirements.txt"),
        "the bootstrap has to install the requirements the package ships, not a \
         list of its own"
    );
    assert!(
        script.contains("command -v uv"),
        "the bootstrap has to leave quietly when there is no `uv` to build with"
    );
    assert!(
        script.contains("UV_PYTHON_DOWNLOADS=automatic"),
        "Fedora's uv refuses interpreter downloads by default, and Fedora 44 \
         ships no Python 3.12 to build upstream's requirements on"
    );
    let mode = std::fs::metadata(skel.join(".config/niri/scripts/inir-venv-setup.sh"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(
        mode & 0o777,
        0o755,
        "the startup line runs the script itself, so it has to stay executable"
    );

    let startup = std::fs::read_to_string(skel.join(".config/niri/config.d/50-startup.kdl"))
        .expect("the iNiR Niri skel starts the session");
    assert!(
        startup.contains(
            r#"spawn-at-startup "bash" "-c" "$HOME/.config/niri/scripts/inir-venv-setup.sh""#
        ),
        "nothing else runs the bootstrap, so a switch would leave the venv \
         unbuilt until someone read the module comment"
    );

    let inir = super::shell::find("inir").expect("inir registered");
    let plan = inir.install_plan.expect("inir installs from a repository");
    for package in [
        "uv",
        "cairo-devel",
        "cairo-gobject-devel",
        "gobject-introspection-devel",
        "dbus-devel",
    ] {
        assert!(
            plan.packages().contains(&package),
            "`{package}` is what the bootstrap needs to finish, so the plan has \
             to put it there"
        );
    }
    assert!(
        plan.packages().contains(&"plasma-integration"),
        "iNiR's rendered colours reach Qt applications through the KDE platform \
         theme, and iNiR's own doctor names this package when it is missing"
    );
    let discard = inir.discard_paths;
    assert!(
        discard.contains(&".local/state/quickshell/venv-setup.log")
            && discard.contains(&".local/state/quickshell/.venv"),
        "the venv and the log of the build that made it are mine to throw away, \
         and a backup should carry neither"
    );
}

#[test]
fn a_shell_without_an_install_plan_is_not_removable_either() {
    // A removal only makes sense for a package the picker could have put there.
    // A shell the picker cannot install — one that lives in a base image with no
    // repository entry — must therefore not carry a removal plan either, or the
    // dialog would offer to take off something no switch ever added. Every
    // packaged shell here carries both plans today, so the rule reads as a
    // formality; it is the shape the pair has to keep as shells are added.
    for compositor in compositor::all() {
        for shell in compositor.shells() {
            if shell.install_plan.is_none() {
                assert!(
                    shell.uninstall_plan.is_none(),
                    "{}/{} cannot be installed, so it must not be removable either",
                    compositor.id(),
                    shell.id
                );
            }
        }
    }
}

#[test]
fn every_installed_plan_can_do_something() {
    // A plan with no stages, an empty stage, or a `package` that appears in
    // none of its stages installs nothing while looking deliberate in the
    // source. `packages()` on such a plan is empty, and "is every one of
    // these packages installed?" over an empty list is answered yes, so the
    // switch would report the shell as already installed and then try to
    // start it. That is the one place where a typo becomes a lie told to the
    // user after their backup was taken.
    //
    // DMS is the only plan there is today, so this reads as a formality. It
    // is here for whoever adds the next shell and mistypes a stage.
    for compositor in compositor::all() {
        for shell in compositor.shells() {
            let Some(plan) = shell.install_plan else {
                continue;
            };
            assert!(
                plan.is_well_formed(),
                "{}/{} declares an install plan that cannot install anything: \
                 stages {:?}",
                compositor.id(),
                shell.id,
                plan.stages
            );
            assert!(
                !plan.packages().is_empty(),
                "{}/{} has a well-formed plan naming no packages",
                compositor.id(),
                shell.id
            );
        }
    }
}

#[test]
fn every_shell_states_where_its_config_lives() {
    // A shell with no config location cannot be backed up or seeded, which
    // is why preflight refuses it. Recorded here so adding such a shell is a
    // deliberate act rather than an oversight.
    for compositor in compositor::all() {
        for shell in compositor.shells() {
            if shell.config_dir.is_none() {
                assert!(
                    shell.install_plan.is_none(),
                    "{} has no config directory, so it must also be uninstallable",
                    shell.id
                );
            }
        }
    }
}

#[test]
fn unknown_id_does_not_resolve() {
    assert!(super::shell::find("does-not-exist").is_none());
}

#[test]
fn every_identity_is_offered_by_at_least_one_compositor() {
    // Only meaningful because an identity lives in its own file now. A shell
    // nothing registers is a file nobody reads: it compiles, it passes every
    // other test in this file, and it is not on any card. Listing the
    // identities here is what makes that visible.
    for identity in shells::every() {
        let offered = super::compositor::all()
            .iter()
            .any(|c| c.shells().iter().any(|s| s.id == identity.id));
        assert!(
            offered,
            "{} is declared in shells/ but no compositor offers it",
            identity.id
        );
    }
}

#[test]
fn one_identity_serves_every_compositor_offering_that_shell() {
    // The drift the split exists to prevent. `Port` has no `package`,
    // `upstream`, `install_plan` or `companion_systemd_user_units` field, so
    // these cannot disagree today; this test is what stops someone adding one
    // back to `Port` and quietly reintroducing the second copy. A mismatch here
    // would mean one card checks `rpm -q` for a package another card installs.
    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            let identity = shells::every()
                .into_iter()
                .find(|i| i.id == shell.id)
                .unwrap_or_else(|| panic!("{} has no identity file", shell.id));

            assert_eq!(shell.upstream, identity.upstream, "{} upstream", shell.id);
            assert_eq!(shell.package, identity.package, "{} package", shell.id);
            assert_eq!(shell.binary, identity.binary, "{} binary", shell.id);
            assert_eq!(
                shell.start_argv, identity.start_argv,
                "{} start argv",
                shell.id
            );
            assert_eq!(
                shell.companion_systemd_user_units, identity.companion_systemd_user_units,
                "{} companion units",
                shell.id
            );
            assert_eq!(
                shell.generated_paths, identity.generated_paths,
                "{} generated paths",
                shell.id
            );
            assert_eq!(
                shell.discard_paths, identity.discard_paths,
                "{} discard paths",
                shell.id
            );
            assert_eq!(
                shell.companion_paths, identity.companion_paths,
                "{} companion paths",
                shell.id
            );
            assert_eq!(
                shell.runtime_packages, identity.runtime_packages,
                "{} runtime packages",
                shell.id
            );
            assert_eq!(shell.logo, identity.logo, "{} logo", shell.id);
            assert_eq!(
                shell.install_plan.map(|p| p.package),
                identity.install_plan.map(|p| p.package),
                "{} install plan package",
                shell.id
            );
            assert_eq!(
                shell.install_plan.map(|p| (p.stages, p.program, p.args)),
                identity.install_plan.map(|p| (p.stages, p.program, p.args)),
                "{} install plan",
                shell.id
            );
        }
    }
}

#[test]
fn every_port_stays_inside_home() {
    // `config_dir`, `state_dir`, `skel_extra_paths` and `generated_paths` are all
    // joined to `$HOME` and then written to, so a leading slash or a `..` would
    // put a switch outside the user's home. SAFETY.md rule 1 depends on this
    // holding.
    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            for rel in [shell.config_dir, shell.state_dir]
                .into_iter()
                .flatten()
                .chain(shell.skel_extra_paths.iter().copied())
                .chain(shell.generated_paths.iter().copied())
            {
                assert!(
                    !rel.starts_with('/'),
                    "{}/{}: {rel} is absolute",
                    compositor.id(),
                    shell.id
                );
                assert!(
                    !rel.split('/').any(|part| part == ".."),
                    "{}/{}: {rel} climbs out of home",
                    compositor.id(),
                    shell.id
                );
            }
        }
    }
}

#[test]
fn every_declared_skel_extra_path_really_is_in_the_skel() {
    // `apply::plan` treats a missing extra path as a refinement it can skip, so
    // a path renamed or deleted inside a skel used to fail quietly: the switch
    // went through without it, and only the log said why.
    //
    // Scoped to shells that have a skel directory. One without is refused
    // outright by `plan`, before extras matter — see
    // `a_compositor_with_no_skel_yet_refuses_every_shell_on_it` — so there is
    // nothing to check. Every registered compositor has a skel today, so this
    // loop only fires for a shell that arrives before its skel does.
    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            let skel = crate::apply::skel_for(compositor.id(), shell.id);
            if !skel.is_dir() {
                continue;
            }
            for rel in shell.skel_extra_paths {
                assert!(
                    skel.join(rel).exists(),
                    "{}/{} lists {rel} but {} does not carry it",
                    compositor.id(),
                    shell.id,
                    skel.display()
                );
            }
        }
    }
}

#[test]
fn every_file_in_a_skel_reaches_the_machine() {
    // The mirror of the test above, and the one that catches a file added to a
    // skel without a declaration to go with it.
    //
    // `apply::plan` writes exactly five things: the compositor's config_dir, the
    // target's config_dir, its state_dir, `skel_extra_paths` and
    // `companion_paths`. A sixth file dropped into a skel is shipped and never
    // installed — it passes the test above precisely because nothing declares
    // it, and a switch then leaves the previous shell's copy of that path in
    // place. Two files were in exactly that position before this test existed,
    // each with a comment in the skel explaining why the switch has to replace
    // it.
    //
    // `generated_paths` counts as coverage: a skel may seed a file the shell
    // refreshes itself, which is what Noctalia's rendered themes are.
    // `companion_paths` counts for the same reason Ambxst's generated compositor
    // configs are seeded: the switch installs them from the skel when the shell
    // has never run here, and reads them back from the backup when it has.
    fn collect(dir: &std::path::Path, base: &std::path::Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            // A symlink is one file to a switch: `copy` in `apply` puts the
            // link itself in place and never walks through it. The walk has to
            // stop at links for the same reason — end4-pC's config dir is one,
            // pointing into the installed package, and following it would read
            // the package's tree instead of the skel's own files.
            let is_link = std::fs::symlink_metadata(&path)
                .map(|meta| meta.file_type().is_symlink())
                .unwrap_or(false);
            if path.is_dir() && !is_link {
                collect(&path, base, out);
            } else {
                let rel = path
                    .strip_prefix(base)
                    .expect("every skel file is inside the skel")
                    .to_string_lossy()
                    .into_owned();
                out.push(rel);
            }
        }
    }

    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            let skel = crate::apply::skel_for(compositor.id(), shell.id);
            if !skel.is_dir() {
                continue;
            }
            let mut files = Vec::new();
            collect(&skel, &skel, &mut files);
            files.sort();
            for rel in files {
                let owned = [
                    Some(compositor.config_dir()),
                    shell.config_dir,
                    shell.state_dir,
                ]
                .into_iter()
                .flatten()
                .chain(shell.skel_extra_paths.iter().copied())
                .chain(shell.generated_paths.iter().copied())
                .chain(shell.companion_paths.iter().copied())
                .any(|prefix| rel == prefix || rel.starts_with(&format!("{prefix}/")));
                assert!(
                    owned,
                    "{}/{} ships {rel} but nothing makes a switch write it: plan only \
                     writes config_dir, state_dir, skel_extra_paths and companion_paths, \
                     and this file is under none of them",
                    compositor.id(),
                    shell.id
                );
            }
        }
    }
}

#[test]
fn every_toml_in_a_skel_parses() {
    // A skel is copied to the machine verbatim, so a file that does not parse is
    // shipped broken: the shell reads it, rejects it, and quietly drops whatever
    // it was meant to configure. Noctalia's `config.toml` carried a duplicate
    // `[shell]` table in the Niri skel for exactly this reason — the file reaches
    // the user either way, and nothing above looked inside it. Parsing every
    // `.toml` a skel ships makes a redefinition that would cost a setting fail
    // the suite instead.
    fn collect(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "toml") {
                out.push(path);
            }
        }
    }

    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            let skel = crate::apply::skel_for(compositor.id(), shell.id);
            if !skel.is_dir() {
                continue;
            }
            let mut files = Vec::new();
            collect(&skel, &mut files);
            for path in files {
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
                let rel = path.strip_prefix(&skel).unwrap_or(&path);
                toml::from_str::<toml::Value>(&text).unwrap_or_else(|e| {
                    panic!(
                        "{}/{} ships {} which is not valid TOML: {e}",
                        compositor.id(),
                        shell.id,
                        rel.display()
                    )
                });
            }
        }
    }
}

#[test]
fn a_mechanism_agrees_with_its_unit_name() {
    // `apply::start` and `apply::verify` branch on the mechanism and then read
    // the unit name, so a shell could claim one and not supply the other. Both
    // directions are checked because `start` only consults the unit *after*
    // asking the mechanism, which is exactly why the two can drift apart.
    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            assert_eq!(
                shell.mechanism.uses_systemd_unit(),
                shell.systemd_user_unit.is_some(),
                "{}/{}: mechanism {:?} and unit {:?} have to agree",
                compositor.id(),
                shell.id,
                shell.mechanism,
                shell.systemd_user_unit
            );
        }
    }
}

#[test]
fn a_unit_that_autostarts_with_a_shell_belongs_to_one() {
    // The enable/disable verbs are only reachable through a systemd unit, so a
    // shell with no unit that declared one would carry a fact nothing reads.
    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            if !shell.autostart_systemd_user_units.is_empty() {
                assert!(
                    shell.mechanism.uses_systemd_unit(),
                    "{}/{} declares units that autostart with it but starts \
                     through no systemd unit",
                    compositor.id(),
                    shell.id
                );
            }
        }
    }

    // The one list: DMS declares `dsearch`, the unit `danksearch` ships. DMS
    // searches through it, and leaving DMS has to stop, disable and mask it on
    // every compositor — on Niri because the image enables it globally, on
    // Hyprland because the picker's own enable put it there. Pinned both ways
    // so a shell quietly gaining or losing the daemon is a test failure rather
    // than a search service left running beside a shell it does not belong to.
    assert_eq!(
        super::shells::dms::IDENTITY.autostart_systemd_user_units,
        &["dsearch"]
    );
    for identity in super::shells::every() {
        if identity.id == "dms" {
            continue;
        }
        assert!(
            identity.autostart_systemd_user_units.is_empty(),
            "{} declares a unit that autostarts with it; only DMS owns one today",
            identity.id
        );
    }
}

#[test]
fn a_unit_with_no_install_section_is_enabled_through_a_wants_link() {
    // Almost every unit here carries its own `[Install]`, so `systemctl --user
    // enable` writes the link and the picker never has to know how it got there.
    // iNiR's `inir.service` is the one that does not: upstream enables it by
    // hand, with a symlink into the running compositor's own `.wants/`
    // directory, and `enable` refuses a unit with nothing to install from. That
    // is what `Port::systemd_user_unit_wanted_by` states, and the two
    // declarations have to agree or the switch takes the wrong branch: a target
    // named for a shell that starts some other way sends `apply` writing a link
    // nothing reads, and a shell whose unit has no `[Install]` but no target
    // either is left enabled by a verb that cannot have worked.
    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            if shell.systemd_user_unit_wanted_by.is_none() {
                continue;
            }
            assert!(
                shell.systemd_user_unit.is_some() && shell.mechanism.uses_systemd_unit(),
                "{}/{} enables a unit through a wants link, so it has to start \
                 through a unit of its own",
                compositor.id(),
                shell.id
            );
        }
    }

    // Pinned both ways: iNiR on Niri is the only shell whose unit ships no
    // `[Install]`, and the target it names is the compositor's own service.
    let inir = super::compositor::NIRI
        .shells()
        .iter()
        .find(|s| s.id == "inir")
        .expect("inir registered for niri");
    assert_eq!(inir.systemd_user_unit_wanted_by, Some("niri.service"));
    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            if shell.id == "inir" {
                continue;
            }
            assert_eq!(
                shell.systemd_user_unit_wanted_by,
                None,
                "{}/{} enables itself through a wants link; only iNiR's unit is \
                 written without an `[Install]` section today",
                compositor.id(),
                shell.id
            );
        }
    }
}

#[test]
fn only_a_detached_start_detaches() {
    // The flag belongs to the shells whose launcher waits on the command
    // returning. Every other mechanism either has a unit that never reaches
    // the spawn branch at all, or a compositor start hook that does not wait,
    // so carrying the flag would cost the process accounting `stop` relies on
    // and buy nothing.
    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            if shell.mechanism.uses_systemd_unit() {
                assert!(
                    !shell.mechanism.needs_detached_start(),
                    "{}/{} goes through systemd, so a flag would never be seen",
                    compositor.id(),
                    shell.id
                );
            }
        }
    }

    // `SpawnAtStartup` is the one mechanism that has to detach, because Niri
    // waits for a startup command to return before continuing. Asserted on the
    // mechanism itself as well, so a variant that stopped detaching is caught
    // even if no port used it.
    assert!(super::shell::StartMechanism::SpawnAtStartup.needs_detached_start());
    for mechanism in [
        super::shell::StartMechanism::Autostart,
        super::shell::StartMechanism::StartEvent,
        super::shell::StartMechanism::SystemdUserUnit,
        super::shell::StartMechanism::DirectProcess,
    ] {
        assert!(
            !mechanism.needs_detached_start(),
            "{mechanism:?} does not wait for the command, so detaching buys nothing"
        );
    }

    // Niri is where the flag is used: its Noctalia port starts from
    // `spawn-at-startup` and so must detach, while DMS there goes through the
    // same systemd unit as everywhere else.
    let niri = &super::compositor::NIRI;
    let noctalia = niri
        .shells()
        .iter()
        .find(|s| s.id == "noctalia")
        .expect("noctalia registered for niri");
    assert!(
        noctalia.mechanism.needs_detached_start(),
        "niri waits for spawn-at-startup to return, so noctalia has to detach"
    );
    let dms = niri
        .shells()
        .iter()
        .find(|s| s.id == "dms")
        .expect("dms registered for niri");
    assert!(
        !dms.mechanism.needs_detached_start(),
        "niri/dms starts through systemd and does not detach"
    );

    // Nothing on hyprland detaches, whatever its mechanism: hyprland.lua's
    // exec_on does not wait for the command, so the flag buys nothing and
    // costs the process accounting `stop` relies on.
    for shell in super::compositor::HYPRLAND.shells() {
        assert!(
            !shell.mechanism.needs_detached_start(),
            "hyprland/{} should not detach",
            shell.id
        );
    }
}

#[test]
fn every_installed_path_is_backed_up() {
    // The one that matters most, and the reason the backup set is declared per
    // shell instead of being derived.
    //
    // A switch writes every `skel_extra_paths` entry, replacing whatever is
    // there. When the list was derived from `config_dir` and `state_dir` alone,
    // three such writes were not covered by any backup: Noctalia's launcher
    // entry, and DMS's state directory. So a switch could destroy the user's
    // copy of a file with no way back — the one outcome rule 2 of SAFETY.md
    // exists to prevent, and it was shipping.
    //
    // Coverage is checked against the real backup list rather than against the
    // fields directly, because that list is what the switch actually uses. A
    // path counts as covered if some entry in the list is it, contains it, or
    // is contained by it — the last case because backing up a parent directory
    // covers a file inside it.
    let home = std::path::PathBuf::from("/home/tester");
    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            let saved: Vec<String> = crate::switch::backup_sources(&home, compositor, Some(shell))
                .iter()
                .map(|p| {
                    p.strip_prefix(&home)
                        .expect("every backup path is inside home")
                        .to_string_lossy()
                        .into_owned()
                })
                .collect();
            let covers = |rel: &str| {
                saved.iter().any(|s| {
                    rel == s
                        || rel.starts_with(&format!("{s}/"))
                        || s.starts_with(&format!("{rel}/"))
                })
            };

            for rel in shell.skel_extra_paths {
                assert!(
                    covers(rel),
                    "{}/{} writes {rel} on every switch but never backs it up: \
                     add it to backup_extra_paths, or to SHARED_PATHS if it \
                     belongs to the session rather than to this shell",
                    compositor.id(),
                    shell.id
                );
            }
            for rel in shell.backup_extra_paths {
                assert!(
                    covers(rel),
                    "{}/{} backs up {rel}, which its backup list already \
                     covers another way — drop the duplicate",
                    compositor.id(),
                    shell.id
                );
            }
        }
    }
}

#[test]
fn every_generated_path_is_backed_up() {
    // The mirror of `every_installed_path_is_backed_up`, for the paths a shell
    // writes for itself rather than the ones a switch writes for it.
    //
    // `clean` will not delete a generated path unless this switch's backup
    // reproduces it byte for byte, so a path outside every directory
    // `backup_sources` walks would be kept forever and reported as such. That is
    // merely untidy. The reason it is still a test is the guard itself: the day
    // somebody loosens the byte-for-byte check, an uncovered entry becomes a
    // delete with no copy anywhere — which is exactly what the rule-2 test above
    // exists to prevent, reached from the other direction.
    let home = std::path::PathBuf::from("/home/tester");
    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            let saved: Vec<String> = crate::switch::backup_sources(&home, compositor, Some(shell))
                .iter()
                .map(|p| {
                    p.strip_prefix(&home)
                        .expect("every backup path is inside home")
                        .to_string_lossy()
                        .into_owned()
                })
                .collect();
            for rel in shell.generated_paths {
                let covered = saved.iter().any(|s| {
                    rel == s
                        || rel.starts_with(&format!("{s}/"))
                        || s.starts_with(&format!("{rel}/"))
                });
                assert!(
                    covered,
                    "{}/{} removes {rel} after a switch but never backs it up: \
                     it has to sit inside a path backup_sources already walks \
                     (SHARED_PATHS, config_dir or state_dir)",
                    compositor.id(),
                    shell.id
                );
            }
        }
    }
}

#[test]
fn every_companion_path_is_backed_up() {
    // The third leg of the same rule. `clean` will not delete a companion's
    // data unless this switch's backup reproduces it byte for byte, so a path
    // outside everything `backup_sources` walks would be kept forever and
    // reported as such — the data would survive, but as one more piece of the
    // shell that never goes away. The deeper reason for the test is the check
    // itself: the day somebody loosens it, an uncovered entry becomes a delete
    // with no copy anywhere, and a calendar database is not a thing that
    // regenerates.
    let home = std::path::PathBuf::from("/home/tester");
    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            let saved: Vec<String> = crate::switch::backup_sources(&home, compositor, Some(shell))
                .iter()
                .map(|p| {
                    p.strip_prefix(&home)
                        .expect("every backup path is inside home")
                        .to_string_lossy()
                        .into_owned()
                })
                .collect();
            for rel in shell.companion_paths {
                let covered = saved.iter().any(|s| {
                    rel == s
                        || rel.starts_with(&format!("{s}/"))
                        || s.starts_with(&format!("{rel}/"))
                });
                assert!(
                    covered,
                    "{}/{} keeps {rel} for its companions but never backs it up: \
                     it has to sit inside a path backup_sources already walks",
                    compositor.id(),
                    shell.id
                );
            }
        }
    }
}

#[test]
fn no_declared_path_can_reach_the_backups() {
    // The backups are what every removal in `clean` is measured against, so they
    // are the one thing on the other side of it that must never be reached. Two
    // ways to reach them, both fatal: a path sitting *inside* them, treated as a
    // shell's own, and a path *containing* them — `.local/share` would take the
    // whole tree. The byte-for-byte check does not save the second case either,
    // because the backup does contain itself.
    //
    // Nothing declared today is anywhere near `~/.local/share/backups`. This is
    // the half of the guarantee that fails in the suite rather than in a switch:
    // the registry cannot grow a path one directory too high without saying so,
    // and `clean` refuses on its own side whatever still gets past here.
    let home = PathBuf::from(std::env::var_os("HOME").expect("HOME is set"));
    let backups = crate::session::State::backups_dir();

    let mut declared: BTreeSet<&str> = BTreeSet::new();
    declared.extend(crate::switch::SHARED_PATHS.iter().copied());
    for compositor in compositor::all() {
        declared.insert(compositor.config_dir());
        for shell in compositor.shells() {
            declared.extend([shell.config_dir, shell.state_dir].into_iter().flatten());
            declared.extend(shell.skel_extra_paths.iter().copied());
            declared.extend(shell.backup_extra_paths.iter().copied());
            declared.extend(shell.generated_paths.iter().copied());
            declared.extend(shell.companion_paths.iter().copied());
            declared.extend(shell.discard_paths.iter().copied());
        }
    }

    for rel in declared {
        let live = home.join(rel);
        assert!(
            !live.starts_with(&backups),
            "{rel} sits inside the backups at {}",
            backups.display()
        );
        assert!(
            !backups.starts_with(&live),
            "{rel} contains the backups at {}; removing it would take them too",
            backups.display()
        );
    }
}

#[test]
fn every_discard_path_is_never_preserved() {
    // The inverse of the two tests above, and the reason they can stay strict.
    //
    // `generated_paths` may only live where a backup already walks, because
    // `clean` refuses to delete it otherwise. `discard_paths` may not live there
    // at all: it is the list of things with no data in them, `clean` deletes it
    // without asking the backup, and a path the backup reproduces is by
    // definition a path somebody could have written to. Letting the two overlap
    // would turn "there is nothing here to lose" from a property into a hope.
    //
    // `covers` is the same three-way containment the tests above use, run the
    // other way round: it fails if a discard path is a backed-up path, sits
    // inside one, or contains one. The last case matters most — a discard path
    // that were a parent of a backed-up directory would delete on trust
    // everything under it, including the copy the switch just took.
    let home = std::path::PathBuf::from("/home/tester");
    for compositor in super::compositor::all() {
        for shell in compositor.shells() {
            let saved: Vec<String> = crate::switch::backup_sources(&home, compositor, Some(shell))
                .iter()
                .map(|p| {
                    p.strip_prefix(&home)
                        .expect("every backup path is inside home")
                        .to_string_lossy()
                        .into_owned()
                })
                .collect();
            for rel in shell.discard_paths {
                assert!(
                    !rel.starts_with('/') && !rel.contains(".."),
                    "{}/{} discards {rel}, which is not a path inside home",
                    compositor.id(),
                    shell.id
                );
                let covered = saved.iter().any(|s| {
                    rel == s
                        || rel.starts_with(&format!("{s}/"))
                        || s.starts_with(&format!("{rel}/"))
                });
                assert!(
                    !covered,
                    "{}/{} discards {rel}, which a switch backs up: it belongs in \
                     generated_paths, where the backup has to reproduce it byte \
                     for byte before it is removed",
                    compositor.id(),
                    shell.id
                );
            }
            for (i, rel) in shell.discard_paths.iter().enumerate() {
                assert!(
                    !shell.discard_paths[..i].contains(rel),
                    "{}/{} discards {rel} twice",
                    compositor.id(),
                    shell.id
                );
            }
        }
    }
}

#[test]
fn a_compositor_with_no_skel_yet_refuses_every_shell_on_it() {
    // A compositor with no skel has nothing to install, so a switch under it
    // cannot complete — and has to say so rather than leave the old
    // configuration in place and call it a switch.
    //
    // Niri once had three ports and an empty skel, which first exposed this.
    // The rule is the general shape of that hole — any compositor whose skel
    // has not been written yet hits exactly this — so it is tested through a
    // compositor with no skel directory at all, independent of which real
    // compositors are registered.
    let shell = super::compositor::HYPRLAND
        .shells()
        .iter()
        .find(|s| s.id == "noctalia")
        .expect("noctalia registered for hyprland");

    let home = std::env::temp_dir()
        .join("rakuos-shell-picker-tests")
        .join("no-skel");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let detail = crate::apply::dry_run(
        shell,
        &super::compositor::fake::FAKE,
        &crate::apply::Context {
            home: home.clone(),
            backups_root: home.join("backups"),
            // Nothing has ever written a skel for this id.
            skel: crate::apply::skel_for("fake", "noctalia"),
        },
        &mut |_| {},
    )
    .expect_err("a switch must refuse while there is no skel to install");

    assert!(
        detail.contains("nothing to switch to"),
        "the refusal has to explain itself: {detail}"
    );
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn a_skel_is_missing_only_where_the_shell_cannot_be_installed() {
    // The card-only port used to be the case this ran on for real — iNiR
    // arrived without a skel, and now ships upstream's `defaults/niri`. Every
    // port here has one today, so the loop below only fires for a shell that
    // arrives before its skel does.
    //
    // Niri had three ports and no skel at all, and that was not deliberate.
    // Two of its shells were installable, so the dialog offered them, and the
    // switch then refused with a message about an empty skel — which reads as
    // a bug in the picker rather than a decision about Niri.
    //
    // So the rule is not that every shell has a skel. It is that a missing
    // skel is only allowed where a switch cannot reach a refusal *after* the
    // backup. Two ways to get there: a shell nobody can put on the machine
    // (no package, no plan), or a card-only port whose `config_dir` is still
    // undeclared — `switch::prepare` refuses that one in `NoConfig` as its
    // first check, before `Backup::create` runs. Declaring `config_dir`
    // without writing the skel would move the refusal behind the backup,
    // which is the pair this rejects.
    for compositor in compositor::all() {
        for shell in compositor.shells() {
            if crate::apply::skel_for(compositor.id(), shell.id).is_dir() {
                continue;
            }
            let refuses_before_a_backup = shell.config_dir.is_none();
            let cannot_be_installed =
                shell.package.is_none() && shell.install_plan.is_none();
            assert!(
                refuses_before_a_backup || cannot_be_installed,
                "{}/{} has no skel, so a switch to it cannot complete — and \
                 nothing stops one from reaching the backup first, so the user \
                 would be refused only after it was taken. Write the skel, or \
                 take the shell out of the registry until there is one.",
                compositor.id(),
                shell.id
            );
        }
    }
}

#[test]
fn ambxst_is_a_first_class_shell() {
    // Ambxst is packaged in the same overlay as DMS, so it is installable,
    // removable, and has a skel and a config location like every other shell.
    // This was not always the case: it used to be declared unpackaged, which is
    // why it had neither an install plan nor a skel. Pinning the whole set here
    // keeps a later change from quietly taking one of them away.
    let ambxst = super::shell::find("ambxst").expect("ambxst registered");
    assert_eq!(ambxst.package, Some("ambxst"));
    assert!(
        ambxst.install_plan.is_some(),
        "ambxst installs from the rakuos repository"
    );
    assert!(ambxst.uninstall_plan.is_some(), "and can be removed again");
    assert!(
        crate::apply::skel_for("hyprland", "ambxst").is_dir(),
        "ambxst ships a skel, or a switch to it could not complete; removing it \
         would leave an installable card with nothing to install"
    );
    assert!(
        crate::apply::skel_for("niri", "ambxst").is_dir(),
        "ambxst ships a niri skel, or a switch to it could not complete"
    );
    assert!(
        ambxst.config_dir.is_some(),
        "the skel only means something because the config location is known; \
         `switch::prepare` refuses a shell whose config_dir is None"
    );
    assert!(
        ambxst.runtime_packages.contains(&"quickshell-git"),
        "ambxst runs on quickshell, so it has to say so"
    );
}

#[test]
fn every_uninstall_plan_can_do_something() {
    // The mirror of `every_installed_plan_can_do_something`, and it matters for
    // a reason that is easy to miss: an uninstall stage with nothing in it runs
    // successfully and removes nothing, so the switch would announce the shell
    // was gone while its packages were still on the machine. Half-removed and
    // reported as clean is worse than not having offered it.
    for compositor in compositor::all() {
        for shell in compositor.shells() {
            let Some(plan) = shell.uninstall_plan else {
                continue;
            };
            assert!(
                plan.is_well_formed(),
                "{}/{} declares an uninstall plan that cannot remove anything: \
                 stages {:?}",
                compositor.id(),
                shell.id,
                plan.stages
            );
            assert!(!plan.packages().is_empty());
        }
    }
}

#[test]
fn a_shell_that_can_be_installed_can_also_be_removed() {
    // Otherwise the picker would put a shell on the machine that only the
    // package manager could ever take off again, which is exactly the
    // accumulation this feature exists to prevent. The rule runs one way here;
    // the other direction — a shell with no plan at all — is checked by
    // `a_shell_without_an_install_plan_is_not_removable_either`.
    for compositor in compositor::all() {
        for shell in compositor.shells() {
            if shell.install_plan.is_none() {
                continue;
            }
            assert!(
                shell.uninstall_plan.is_some(),
                "{}/{} can be installed but never removed, so switching away \
                 from it would leave it behind",
                compositor.id(),
                shell.id
            );
        }
    }
}

#[test]
fn an_uninstall_plan_only_removes_packages_the_shell_was_installed_with() {
    // Removing something the install plan never put in is how one shell takes a
    // package away from another. Both plans are declared in the same file, so
    // the pair can be checked against each other directly.
    for compositor in compositor::all() {
        for shell in compositor.shells() {
            let (Some(install), Some(uninstall)) = (shell.install_plan, shell.uninstall_plan)
            else {
                continue;
            };
            let installed = install.packages();
            for package in uninstall.packages() {
                assert!(
                    installed.contains(&package),
                    "{}/{} removes {package}, which its install plan never installs",
                    compositor.id(),
                    shell.id
                );
            }
        }
    }
}

#[test]
fn a_base_image_shell_can_carry_a_removal_plan() {
    // Noctalia ships in the Hyprland image, and `rum remove` refuses a base
    // package — but that is a runtime property, not a reason to leave the plan
    // off. `switch::uninstall` drops any package whose origin is `Base` before the
    // package manager sees it, so the plan is safe where the shell is in the image
    // and useful where a switch installed it (Niri). The base-image safety itself
    // is tested where it runs, in `src/tests/switch_uninstall.rs`.
    let noctalia = super::shell::find("noctalia").expect("registered");
    assert!(noctalia.install_plan.is_some());
    assert!(noctalia.uninstall_plan.is_some());
    // Its files are the user's, so cleaning them when switching away is `clean`'s
    // job independent of either plan.
    assert!(noctalia.config_dir.is_some() || noctalia.state_dir.is_some());
}

#[test]
fn a_shell_that_runs_on_a_shared_runtime_says_so_and_the_others_do_not() {
    // The declaration `switch::uninstall` filters against, pinned to what the
    // shells actually are. Getting it backwards in either direction costs
    // something real: a shell that runs on quickshell and does not declare it
    // loses its runtime the moment DMS leaves, and Noctalia declaring it would
    // mean switching to Noctalia never cleaned anything of DMS's up.
    for id in ["dms", "caelestia", "ambxst", "end4pc", "inir"] {
        let shell = super::shell::find(id).expect("registered");
        assert!(
            shell.runtime_packages.contains(&"quickshell-git"),
            "{id} runs on quickshell, so it has to say so or DMS will take it away"
        );
        assert!(
            shell.runtime_packages.contains(&"quickshell"),
            "{id} accepts either spelling of the runtime"
        );
    }
    let noctalia = super::shell::find("noctalia").expect("registered");
    assert!(
        noctalia.runtime_packages.is_empty(),
        "Noctalia is a native binary and needs no runtime, which is what makes \
         switching to it the direction that cleans quickshell up"
    );
}
