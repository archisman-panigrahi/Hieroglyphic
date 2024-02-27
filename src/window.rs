use std::time::Instant;

use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::{prelude::*, StringObject};
use itertools::Itertools;

use crate::application::TexApplication;
use crate::config::PROFILE;
use crate::symbol_item::SymbolItem;

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

            obj.setup_symbol_list();
            obj.setup_classifier();
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

    /// Returns the symbols list store object.
    fn symbols(&self) -> &gio::ListStore {
        self.imp().symbols.get().expect("Failed to get symbols")
    }

    fn setup_symbol_list(&self) {
        let mut model = gio::ListStore::new::<gtk::StringObject>();
        model.extend(
            detexify::iter_symbols()
                .map(|sym| sym.id())
                .map(gtk::StringObject::new),
        );
        // let model: gtk::StringList = detexify::iter_symbols().map(|symbol| symbol.id()).collect();
        tracing::debug!("Loaded {} symbols", model.n_items());

        self.imp()
            .symbols
            .set(model.clone())
            .expect("Failed to set symbol model");

        let selection_model = gtk::NoSelection::new(Some(model));
        self.imp().symbol_list.bind_model(
            Some(&selection_model),
            glib::clone!(@weak self as window => @default-panic, move |obj| {
                let symbol_object = obj.downcast_ref::<StringObject>().expect("The object is not of type `StringObject`.");
                let symbol_item = SymbolItem::new(detexify::Symbol::from_id(symbol_object.string().as_str()).expect("Failed to get symbol"));
                symbol_item.upcast()
            }),
        );

        self.imp().symbol_list.set_visible(true);
    }

    fn setup_classifier(&self) {
        let (req_tx, req_rx) = std::sync::mpsc::channel();
        let (res_tx, res_rx) = async_channel::bounded(1);
        self.imp().sender.set(req_tx).expect("Failed to set tx");
        gio::spawn_blocking(move || {
            tracing::info!("Classifier thread started");
            let classifier = detexify::Classifier::default();

            loop {
                let Some(strokes) = req_rx.iter().next() else {
                    //channel has hung up, cleanly exit
                    tracing::info!("Exiting classifier thread");
                    return;
                };

                let classifications: Option<Vec<detexify::Score>> = 'classify: {
                    let Some(sample) = detexify::StrokeSample::new(strokes) else {
                        tracing::warn!("Skipping classification on empty strokes");
                        break 'classify None;
                    };

                    let start = Instant::now();
                    let Some(results) = classifier.classify(sample) else {
                        tracing::warn!("Classifier returned None");
                        break 'classify None;
                    };
                    tracing::info!(
                        "Classification complete in {}ms",
                        start.elapsed().as_millis()
                    );
                    Some(results)
                };

                res_tx
                    .send_blocking(classifications.unwrap_or_default())
                    .expect("Failed to send classifications");
            }
        });

        glib::spawn_future_local(glib::clone!(@weak self as window => async move {
            tracing::debug!("Listening for classifications");
            while let Ok(classifications) = res_rx.recv().await {

                let symbols = window.symbols();
                symbols.remove_all();

                // let objs = classifications.iter().map(|score|gtk::StringObject::new(&score.id)).collect_vec();
                // symbols.extend_from_slice(&objs);

                // swicthing out all 1k symbols takes too long, so only display the first 25
                // TODO: find faster ways and display all
                for symbol in classifications.iter().take(25) {
                    symbols.append(&gtk::StringObject::new(&symbol.id))
                }
            }
        }));
    }

    fn classify(&self) {
        let imp = self.imp();
        let strokes = imp.strokes.borrow().clone();
        imp.sender
            .get()
            .unwrap()
            .send(strokes)
            .expect("Failed to send strokes");
    }


    #[template_callback]
    fn clear(&self, _button: &gtk::Button) {
        // recreate drawing area
        let width = self.imp().drawing_area.content_width();
        let height = self.imp().drawing_area.content_height();
        self.create_surface(width, height);

        //clear previous strokes
        self.imp().strokes.borrow_mut().clear();
        self.imp().current_stroke.borrow_mut().clear();

        self.imp().drawing_area.queue_draw();
    }
}
