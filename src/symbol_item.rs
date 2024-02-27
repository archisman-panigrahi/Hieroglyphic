use glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::ObjectExt};

mod imp {

    use std::cell::{Cell, RefCell};

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/fyi/zoey/TeX-Match/ui/symbol-item.ui")]
    #[properties(wrapper_type = super::SymbolItem)]
    pub struct SymbolItem {
        #[property(get, set)]
        pub(super) icon: RefCell<String>,
        #[property(get, set)]
        pub(super) package: RefCell<String>,
        #[property(get, set)]
        pub(super) font_encoding: RefCell<String>,
        #[property(get, set)]
        pub(super) command: RefCell<String>,
        #[property(get, set)]
        pub(super) mode: RefCell<String>,
        #[property(get, set)]
        pub(super) text_mode: Cell<bool>,
        #[property(get, set)]
        pub(super) math_mode: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SymbolItem {
        const NAME: &'static str = "SymbolItem";
        type ParentType = gtk::Box;
        type Type = super::SymbolItem;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SymbolItem {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for SymbolItem {}
    impl BoxImpl for SymbolItem {}
}

glib::wrapper! {
    pub struct SymbolItem(ObjectSubclass<imp::SymbolItem>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable, gtk::Actionable;
}

#[gtk::template_callbacks]
impl SymbolItem {
    pub fn new(symbol: detexify::Symbol) -> Self {
        Object::builder()
            .property("icon", &format!("{}-symbolic", symbol.id()))
            .property("command", symbol.command)
            .property("package", format!("\\usepackage{{ {} }}", symbol.package))
            .property(
                "mode",
                if symbol.math_mode {
                    "mathmode"
                } else {
                    "textmode"
                },
            )
            // .property("font_encoding", symbol.font_encoding)
            // .property("text_mode", symbol.text_mode)
            // .property("math_mode", symbol.math_mode)
            .build()
    }

    #[template_callback]
    fn on_click(&self) {
        let clipboard = self.clipboard();
        clipboard.set_text("TODO: copy item name");
        //TODO: show adw toast
    }
}
