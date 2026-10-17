use gtk4::prelude::*;
// libadwaita has its own prelude; AdwWindowExt / AdwDialogExt live there.
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow};

use crate::ui::Picker;

const APP_ID: &str = "org.rakuos.ShellPicker";
const DEFAULT_SIZE: i32 = 800;

pub fn run() -> gtk4::glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        // The UI is dark-only (see style.css), so lock the scheme through
        // libadwaita. This is also what keeps libadwaita from warning about a
        // `gtk-application-prefer-dark-theme` that a shell's settings.ini sets.
        libadwaita::StyleManager::default().set_color_scheme(libadwaita::ColorScheme::ForceDark);
        load_css();
        // Channel 1 opens before anything else runs, so a crash during startup
        // still leaves a log behind.
        match crate::logging::init() {
            Some(path) => crate::logging::info(&format!("log file: {}", path.display())),
            None => eprintln!("warning: no writable log location"),
        }
    });

    app.connect_activate(|app| {
        if let Some(window) = app.active_window() {
            window.present();
            return;
        }

        // The window is built first because the page needs it: dialogs are
        // parented to it, so a dialog cannot be created before the window
        // exists. The content is attached afterwards.
        let window = ApplicationWindow::builder()
            .application(app)
            .title("RakuOS Shell Picker")
            .default_width(DEFAULT_SIZE)
            .default_height(DEFAULT_SIZE)
            .build();

        // Gives `window.picker` in style.css something to match.
        window.add_css_class("picker");

        let picker = Picker::new(&window);
        window.set_content(Some(picker.widget()));
        window.present();
    });

    app.run()
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(include_str!("style.css"));
    let display = gtk4::gdk::Display::default().expect("no display");
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
