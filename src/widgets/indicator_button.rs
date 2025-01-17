use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::ObjectExt};

mod imp {

    use super::*;
    use std::cell::Cell;

    #[derive(Default, Debug, glib::Properties)]
    #[properties(wrapper_type = super::IndicatorButton)]
    pub struct IndicatorButton {
        #[property(get, set = Self::set_show_indicator)]
        show_indicator: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IndicatorButton {
        const NAME: &'static str = "IndicatorButton";
        type ParentType = gtk::Button;
        type Type = super::IndicatorButton;
    }

    #[glib::derived_properties]
    impl ObjectImpl for IndicatorButton {
        fn constructed(&self) {
            self.parent_constructed();
            //TODO: use multiple widget with CSS to get nice change animation?
            //https://gitlab.gnome.org/GNOME/libadwaita/-/blob/main/src/adw-indicator-bin.c#L139
            adw::StyleManager::default().connect_accent_color_rgba_notify(glib::clone!(
                #[weak(rename_to = widget)]
                self,
                move |_| {
                    // update widget when accent color changes
                    widget.obj().queue_draw();
                }
            ));
        }
    }

    impl WidgetImpl for IndicatorButton {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.parent_snapshot(snapshot);
            if !self.obj().show_indicator() {
                return;
            }

            let color = adw::StyleManager::default().accent_color_rgba();
            let rect = gtk::graphene::Rect::new(14.0, 15.0, 8.0, 8.0);
            snapshot.push_rounded_clip(&gtk::gsk::RoundedRect::from_rect(rect, 100.0));
            snapshot.append_color(&color, &rect);
            snapshot.pop();
        }
    }

    impl BoxImpl for IndicatorButton {}
    impl ButtonImpl for IndicatorButton {}
    impl IndicatorButton {
        /// Sets the visibility of the indicator.
        pub(super) fn set_show_indicator(&self, show_indicator: bool) {
            self.show_indicator.set(show_indicator);
            self.obj().queue_draw();
        }
    }
}

glib::wrapper! {
    pub struct IndicatorButton(ObjectSubclass<imp::IndicatorButton>)
    @extends gtk::Box, gtk::Widget, gtk::Button,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable, gtk::Actionable;
}
