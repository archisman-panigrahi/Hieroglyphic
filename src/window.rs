
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::application::TexApplication;
use crate::config::PROFILE;

mod imp {
    use std::cell::{OnceCell, RefCell};

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/fyi/zoey/TeX-Match/ui/window.ui")]
    pub struct TeXMatchWindow {
        #[template_child]
        pub drawing_area: TemplateChild<gtk::DrawingArea>,
        #[template_child]
        pub symbol_list: TemplateChild<gtk::ListBox>,
        pub surface: RefCell<Option<cairo::ImageSurface>>,
        pub symbols: OnceCell<gio::ListStore>,
        pub strokes: RefCell<Vec<detexify::Stroke>>,
        pub current_stroke: RefCell<detexify::Stroke>,
        pub sender: OnceCell<std::sync::mpsc::Sender<Vec<detexify::Stroke>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TeXMatchWindow {
        const NAME: &'static str = "TeXMatchWindow";
        type Type = super::TeXMatchWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TeXMatchWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel Profile
            if PROFILE == "Devel" {
                // Causes GTK_CRITICAL: investigae
                // obj.add_css_class("devel");
            }
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for TeXMatchWindow {}
    impl WindowImpl for TeXMatchWindow {}

    impl ApplicationWindowImpl for TeXMatchWindow {}
}

glib::wrapper! {
    pub struct TeXMatchWindow(ObjectSubclass<imp::TeXMatchWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root;
}

#[gtk::template_callbacks]
impl TeXMatchWindow {
    pub fn new(app: &TexApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }
}
