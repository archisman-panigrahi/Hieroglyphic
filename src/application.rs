use tracing::{debug, info};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};

use crate::config::{self};
use crate::window::TeXMatchWindow;

mod imp {
    use super::*;
    use adw::subclass::application::AdwApplicationImpl;
    use glib::WeakRef;
    use std::cell::OnceCell;

    #[derive(Debug, Default)]
    pub struct TexApplication {
        pub window: OnceCell<WeakRef<TeXMatchWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TexApplication {
        const NAME: &'static str = "TexApplication";
        type Type = super::TexApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for TexApplication {}

    impl ApplicationImpl for TexApplication {
        fn activate(&self) {
            debug!("TeXMatch<TexApplication>::activate");
            self.parent_activate();
            let app = self.obj();

            if let Some(window) = self.window.get() {
                let window = window.upgrade().unwrap();
                window.present();
                return;
            }

            let window = TeXMatchWindow::new(&app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.main_window().present();
        }

        fn startup(&self) {
            debug!("TeXMatch<ExampleApplication>::startup");
            self.parent_startup();
            let app = self.obj();

            // Set icons for shell
            gtk::Window::set_default_icon_name(config::APP_ID);

            app.setup_css();
            app.setup_gactions();
            app.setup_accels();
        }
    }

    impl GtkApplicationImpl for TexApplication {}
    impl AdwApplicationImpl for TexApplication {}
}

glib::wrapper! {
    pub struct TexApplication(ObjectSubclass<imp::TexApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl TexApplication {
    fn main_window(&self) -> TeXMatchWindow {
        self.imp().window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        let action_quit = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| {
                // This is needed to trigger the delete event and saving the window state
                app.main_window().close();
                app.quit();
            })
            .build();

        // About
        let action_about = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| {
                app.show_about_dialog();
            })
            .build();
        self.add_action_entries([action_quit, action_about]);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("window.close", &["<Control>w"]);
    }

    fn setup_css(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/fyi/zoey/TeX-Match/style.css");
        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    fn show_about_dialog(&self) {
        let dialog = gtk::AboutDialog::builder()
            .logo_icon_name(config::APP_ID)
            .license_type(gtk::License::Gpl30)
            .version(config::VERSION)
            .transient_for(&self.main_window())
            .translator_credits(gettext("translator-credits"))
            .modal(true)
            .authors(vec!["Zoey Sheffield"])
            .build();

        dialog.present();
    }

    pub fn run(&self) -> glib::ExitCode {
        info!("Tex Match ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        ApplicationExtManual::run(self)
    }
}

impl Default for TexApplication {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("resource-base-path", "/fyi/zoey/TeX-Match/")
            .build()
    }
}
