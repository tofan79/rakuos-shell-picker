use super::*;

fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn wayland() -> HashMap<String, String> {
    env(&[
        ("WAYLAND_DISPLAY", "wayland-1"),
        ("XDG_RUNTIME_DIR", "/run/user/1000"),
    ])
}

#[test]
fn empty_environment_detects_nothing() {
    let d = detect_with(&env(&[]));
    assert!(!d.wayland);
    assert_eq!(d.current, None);
    assert!(d.refusal().is_some());
}

#[test]
fn wayland_alone_is_not_enough_to_name_a_compositor() {
    // Session exists, but no compositor marker. We must refuse rather than
    // fall back to the first tab in the registry.
    let d = detect_with(&wayland());
    assert!(d.wayland);
    assert_eq!(d.current, None);
    assert!(d.refusal().unwrap().contains("refusing to guess"));
}

#[test]
fn xdg_current_desktop_is_not_a_detection_signal() {
    // It is user settable, so it must not be enough on its own.
    let mut e = wayland();
    e.insert("XDG_CURRENT_DESKTOP".into(), "niri".into());
    let d = detect_with(&e);
    assert_eq!(d.current, None);
}

#[test]
fn marker_without_wayland_session_is_rejected() {
    // Guards against a stale marker from an earlier session.
    let mut e = env(&[("HYPRLAND_INSTANCE_SIGNATURE", "abc123")]);
    let d = detect_with(&e);
    assert!(!d.wayland);
    assert_eq!(d.current, None);

    e.insert("WAYLAND_DISPLAY".into(), "wayland-1".into());
    let d = detect_with(&e);
    assert!(!d.wayland, "XDG_RUNTIME_DIR still missing");
    assert_eq!(d.current, None);
}

#[test]
fn every_compositor_is_reported_even_when_not_running() {
    let d = detect_with(&env(&[]));
    let ids: Vec<_> = d.statuses.iter().map(|s| s.id).collect();
    assert_eq!(ids, ["hyprland", "niri"]);
    assert!(d.statuses.iter().all(|s| !s.detected));
}

#[test]
fn niri_needs_its_socket_to_actually_exist() {
    let mut e = wayland();
    // Unexpanded template: this is what a shell that copied the var blindly
    // would see. Must not count.
    e.insert("NIRI_SOCKET".into(), "%NIRI_SOCKET%".into());
    assert_eq!(detect_with(&e).current, None);

    // Points somewhere that does not exist.
    e.insert(
        "NIRI_SOCKET".into(),
        "/run/user/1000/niri.wayland-1.abc".into(),
    );
    assert_eq!(detect_with(&e).current, None);
}

#[test]
fn a_real_niri_socket_names_niri() {
    // The positive half of the test above: a value that names a real unix socket
    // is the one thing that makes Niri the running compositor. A bound listener
    // is used rather than a copied file because the check is specifically
    // `is_socket`, and only a real one passes it.
    use std::os::unix::net::UnixListener;

    let socket = std::env::temp_dir().join(format!("rsp-niri-{}.sock", std::process::id()));
    let _ = std::fs::remove_file(&socket);
    let _listener = match UnixListener::bind(&socket) {
        Ok(listener) => listener,
        // Some sandboxes prohibit creating sockets even in their temp dir.
        // The negative-path detection tests still run there; exercise the
        // positive path on systems where a test socket is permitted.
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => return,
        Err(err) => panic!("bind a unix socket: {err}"),
    };

    let mut e = wayland();
    e.insert("NIRI_SOCKET".into(), socket.to_string_lossy().into_owned());
    assert_eq!(detect_with(&e).current, Some("niri"));

    let _ = std::fs::remove_file(&socket);
}

#[test]
fn empty_marker_value_counts_as_absent() {
    let mut e = wayland();
    e.insert("HYPRLAND_INSTANCE_SIGNATURE".into(), "   ".into());
    assert_eq!(detect_with(&e).current, None);
}

#[test]
fn runtime_dir_must_be_absolute() {
    let mut e = wayland();
    e.insert("XDG_RUNTIME_DIR".into(), "relative/path".into());
    assert!(!detect_with(&e).wayland);
}
