use std::time::Instant;

use gettextrs::gettext;
use gtk::subclass::prelude::*;
use gtk::{gdk, prelude::*};
use gtk::{gio, glib};
use itertools::Itertools;

use crate::application::HieroglyphicApplication;
use crate::config::PROFILE;
use crate::symbol_item::SymbolItem;

mod imp {
    use std::{
        cell::{OnceCell, RefCell},
        sync::mpsc::Sender,
    };

    use adw::subclass::application_window::AdwApplicationWindowImpl;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/finefindus/Hieroglyphic/ui/window.ui")]
    pub struct HieroglyphicWindow {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub drawing_area: TemplateChild<gtk::DrawingArea>,
        #[template_child]
        pub symbol_list: TemplateChild<gtk::ListBox>,
        pub toast: RefCell<Option<adw::Toast>>,
        pub surface: RefCell<Option<cairo::ImageSurface>>,
        pub symbols: OnceCell<gio::ListStore>,
        pub strokes: RefCell<Vec<detexify::Stroke>>,
        pub current_stroke: RefCell<detexify::Stroke>,
        pub sender: OnceCell<Sender<Vec<detexify::Stroke>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HieroglyphicWindow {
        const NAME: &'static str = "HieroglyphicWindow";
        type Type = super::HieroglyphicWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();

            klass.install_action("win.clear", None, move |win, _, _| {
                win.clear();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HieroglyphicWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            obj.setup_symbol_list();
            obj.setup_drawing_area();
            obj.setup_classifier();
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for HieroglyphicWindow {}
    impl WindowImpl for HieroglyphicWindow {}

    impl ApplicationWindowImpl for HieroglyphicWindow {}
    impl AdwApplicationWindowImpl for HieroglyphicWindow {}
}

glib::wrapper! {
    pub struct HieroglyphicWindow(ObjectSubclass<imp::HieroglyphicWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root;
}

#[gtk::template_callbacks]
impl HieroglyphicWindow {
    pub fn new(app: &HieroglyphicApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    /// Shows a basic toast with the given text.
    fn show_toast(&self, text: impl AsRef<str>) {
        let toast = adw::Toast::new(text.as_ref());
        toast.set_use_markup(false);
        // dismiss and replace the previous toast if it exists
        if let Some(prev_toast) = self.imp().toast.replace(Some(toast.clone())) {
            prev_toast.dismiss();
        }
        self.imp().toast_overlay.add_toast(toast.clone());
    }

    /// Returns the symbols list store object.
    fn symbols(&self) -> &gio::ListStore {
        self.imp()
            .symbols
            .get()
            .expect("`symbols` should be initialized in `setup_symbol_list`")
    }

    fn setup_symbol_list(&self) {
        let mut model = gio::ListStore::new::<gtk::StringObject>();
        model.extend(
            detexify::iter_symbols()
                .map(|sym| sym.id())
                .map(gtk::StringObject::new),
        );
        tracing::debug!("Loaded {} symbols", model.n_items());

        self.imp()
            .symbols
            .set(model.clone())
            .expect("Failed to set symbol model");

        let selection_model = gtk::NoSelection::new(Some(model));
        self.imp().symbol_list.bind_model(
            Some(&selection_model),
            glib::clone!(@weak self as window => @default-panic, move |obj| {
                let symbol_object = obj.downcast_ref::<gtk::StringObject>().expect("Object should be of type `StringObject`");
                let symbol_item = SymbolItem::new(detexify::Symbol::from_id(symbol_object.string().as_str()).expect("`symbol_object` should be a valid symbol id"));
                symbol_item.upcast()
            }),
        );
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

                // switching out all 1k symbols takes too long, so only display the first 25
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

    fn create_surface(&self, width: i32, height: i32) -> cairo::ImageSurface {
        cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)
            .expect("Failed to create surface")
    }

    /// Returns a theme-specific color for the drawing line.
    fn line_color(&self) -> gdk::RGBA {
        if adw::StyleManager::default().is_dark() {
            // #CCCCCC
            gdk::RGBA::new(0.8, 0.8, 0.8, 1.0)
        } else {
            // adw @dark_2 color
            gdk::RGBA::new(0.37, 0.36, 0.39, 1.0)
        }
    }

    fn setup_drawing_area(&self) {
        self.imp().drawing_area.set_draw_func(
            glib::clone!(@weak self as window => move |_area: &gtk::DrawingArea, ctx: &cairo::Context, width, height| {
                let mut surface = window.imp().surface.borrow_mut();
                let surface = surface.get_or_insert_with(|| window.create_surface(width, height));

                ctx.set_source_surface(surface, 0.0, 0.0).expect("Failed to set surface");
                ctx.set_source_color(&window.line_color());
                ctx.set_line_width(3.0);
                ctx.set_line_cap(cairo::LineCap::Round);

                let curr_stroke = window.imp().current_stroke.borrow().clone();
                for stroke in window.imp().strokes.borrow().iter().chain(std::iter::once(&curr_stroke)) {
                    tracing::trace!("Drawing: {:?}", stroke);
                    let mut looped = false;
                    for (p, q) in stroke.points().tuple_windows() {
                        ctx.move_to(p.x, p.y);
                        ctx.line_to(q.x, q.y);
                        looped = true;
                    }
                    ctx.stroke().expect("Failed to draw stroke");

                    if !looped && stroke.points().count() == 1 {
                        let p = stroke.points().next().unwrap();
                        ctx.arc(p.x, p.y, 1.5, 0.0, 2.0 * std::f64::consts::PI);
                        ctx.fill().expect("Failed to fill");
                    }
                }
            }
        ));
    }

    #[template_callback]
    fn clear(&self) {
        //clear previous strokes
        self.imp().strokes.borrow_mut().clear();
        self.imp().current_stroke.borrow_mut().clear();

        self.imp().drawing_area.queue_draw();
    }

    #[template_callback]
    fn on_resize(&self, width: i32, height: i32) {
        //recreate surface on size change
        self.imp()
            .surface
            .borrow_mut()
            .get_or_insert_with(|| self.create_surface(width, height));
    }

    #[template_callback]
    fn on_drag_begin(&self, x: f64, y: f64) {
        tracing::trace!("Drag started at {},{}", x, y);
        self.imp()
            .current_stroke
            .borrow_mut()
            .add_point(detexify::Point { x, y });
        self.imp().drawing_area.queue_draw();
    }

    #[template_callback]
    fn on_drag_update(&self, x: f64, y: f64) {
        tracing::trace!("Drag update at {},{}", x, y);
        let mut stroke = self.imp().current_stroke.borrow_mut();
        //x,y refers to movements relative to start coord
        let detexify::Point {
            x: prev_x,
            y: prev_y,
        } = stroke.points().next().copied().unwrap();
        stroke.add_point(detexify::Point {
            x: prev_x + x,
            y: prev_y + y,
        });
        self.imp().drawing_area.queue_draw();
    }

    #[template_callback]
    fn on_drag_end(&self, x: f64, y: f64) {
        tracing::trace!("Drag end at {},{}", x, y);
        let stroke = self.imp().current_stroke.take();
        self.imp().strokes.borrow_mut().push(stroke);
        self.imp().drawing_area.queue_draw();
        self.classify();
    }

    #[template_callback]
    fn on_item_activated(&self, row: Option<&gtk::ListBoxRow>) {
        let binding = row.and_then(|row| row.child());
        let Some(symbol) = binding.and_downcast_ref::<SymbolItem>() else {
            return;
        };
        let command = symbol.command();
        let clipboard = self.clipboard();
        clipboard.set_text(&command);
        tracing::debug!("Selected: {} ({})", &command, symbol.id());
        self.show_toast(gettext("Copied “{}”").replace("{}", &command));
    }
}
