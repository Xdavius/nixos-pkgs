use crate::models::presets::get_preset_categories;
use crate::models::{FirewallConfig, FirewallRule};
use gettextrs::gettext;
use gtk4::prelude::*;
use gtk4::{Label, Orientation};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PresetsDialog {
    window: adw::Window,
    config: Rc<RefCell<FirewallConfig>>,
    on_added: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
}

impl PresetsDialog {
    pub fn new(config: Rc<RefCell<FirewallConfig>>) -> Rc<Self> {
        let window = adw::Window::builder()
            .modal(true)
            .default_width(450)
            .default_height(550)
            .build();

        let toolbar_view = adw::ToolbarView::new();

        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(&Label::new(Some(&gettext("Predefined rules")))));
        toolbar_view.add_top_bar(&header);

        let scrolled = gtk4::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();

        let content = gtk4::Box::new(Orientation::Vertical, 16);
        content.set_margin_top(16);
        content.set_margin_bottom(16);
        content.set_margin_start(16);
        content.set_margin_end(16);

        let categories = get_preset_categories();
        let check_buttons: Rc<RefCell<Vec<(gtk4::CheckButton, usize, usize)>>> =
            Rc::new(RefCell::new(Vec::new()));

        for (cat_idx, category) in categories.iter().enumerate() {
            // Category header
            let cat_label = Label::new(Some(&category.name));
            cat_label.add_css_class("heading");
            cat_label.set_halign(gtk4::Align::Start);
            if cat_idx > 0 {
                cat_label.set_margin_top(8);
            }
            content.append(&cat_label);

            let cat_box = gtk4::Box::new(Orientation::Vertical, 0);
            cat_box.add_css_class("card");

            for (rule_idx, rule) in category.rules.iter().enumerate() {
                if rule_idx > 0 {
                    let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
                    cat_box.append(&sep);
                }

                let row = gtk4::Box::new(Orientation::Horizontal, 12);
                row.set_margin_top(8);
                row.set_margin_bottom(8);
                row.set_margin_start(12);
                row.set_margin_end(12);

                let check = gtk4::CheckButton::new();
                row.append(&check);

                let info_box = gtk4::Box::new(Orientation::Vertical, 2);
                info_box.set_hexpand(true);

                let name_label = Label::new(Some(&rule.name));
                name_label.set_halign(gtk4::Align::Start);
                info_box.append(&name_label);

                let detail = Label::new(Some(&format!("{} {}", rule.protocol, rule.ports)));
                detail.add_css_class("caption");
                detail.add_css_class("dim-label");
                detail.set_halign(gtk4::Align::Start);
                info_box.append(&detail);

                row.append(&info_box);
                cat_box.append(&row);

                check_buttons
                    .borrow_mut()
                    .push((check, cat_idx, rule_idx));
            }

            content.append(&cat_box);
        }

        // Buttons
        let btn_box = gtk4::Box::new(Orientation::Horizontal, 12);
        btn_box.set_halign(gtk4::Align::End);
        btn_box.set_margin_top(16);

        let cancel_btn = gtk4::Button::with_label(&gettext("Cancel"));
        cancel_btn.add_css_class("pill");

        let add_btn = gtk4::Button::new();
        let add_content = adw::ButtonContent::builder()
            .icon_name("list-add-symbolic")
            .label(&gettext("Add selection"))
            .build();
        add_btn.set_child(Some(&add_content));
        add_btn.add_css_class("pill");
        add_btn.add_css_class("suggested-action");

        btn_box.append(&cancel_btn);
        btn_box.append(&add_btn);
        content.append(&btn_box);

        scrolled.set_child(Some(&content));
        toolbar_view.set_content(Some(&scrolled));
        window.set_content(Some(&toolbar_view));

        let on_added: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));

        let dialog = Rc::new(Self {
            window: window.clone(),
            config: config.clone(),
            on_added: on_added.clone(),
        });

        // Cancel
        let window_for_cancel = window.clone();
        cancel_btn.connect_clicked(move |_| {
            window_for_cancel.close();
        });

        // Add selected
        let config_for_add = config.clone();
        let window_for_add = window.clone();
        let on_added_for_add = on_added.clone();
        let checks = check_buttons.clone();
        add_btn.connect_clicked(move |_| {
            let categories = get_preset_categories();
            let mut added = 0;

            for (check, cat_idx, rule_idx) in checks.borrow().iter() {
                if check.is_active() {
                    if let Some(cat) = categories.get(*cat_idx) {
                        if let Some(rule) = cat.rules.get(*rule_idx) {
                            let cloned: FirewallRule = rule.clone();
                            config_for_add.borrow_mut().rules.push(cloned);
                            added += 1;
                        }
                    }
                }
            }

            if added > 0 {
                if let Some(ref cb) = *on_added_for_add.borrow() {
                    cb();
                }
            }

            window_for_add.close();
        });

        dialog
    }

    pub fn set_on_added<F: Fn() + 'static>(&self, callback: F) {
        *self.on_added.borrow_mut() = Some(Rc::new(callback));
    }

    pub fn present(&self, parent: Option<&impl IsA<gtk4::Widget>>) {
        if let Some(p) = parent {
            if let Some(window) = p.dynamic_cast_ref::<gtk4::Window>() {
                self.window.set_transient_for(Some(window));
            }
        }
        self.window.present();
    }
}
