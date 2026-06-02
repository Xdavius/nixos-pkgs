use crate::models::FirewallConfig;
use gettextrs::gettext;
use gtk4::prelude::*;
use gtk4::{Button, Label, Orientation};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Widget displaying the list of active firewall rules inside a card
pub struct RulesListWidget {
    container: gtk4::Box,
    config: Rc<RefCell<FirewallConfig>>,
    on_edit: Rc<RefCell<Option<Rc<dyn Fn(usize)>>>>,
}

impl Clone for RulesListWidget {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
            config: self.config.clone(),
            on_edit: self.on_edit.clone(),
        }
    }
}

impl RulesListWidget {
    pub fn new(config: Rc<RefCell<FirewallConfig>>) -> Self {
        let container = gtk4::Box::new(Orientation::Vertical, 0);

        let widget = Self {
            container: container.clone(),
            config,
            on_edit: Rc::new(RefCell::new(None)),
        };

        widget.populate();
        widget
    }

    pub fn set_on_edit<F: Fn(usize) + 'static>(&self, callback: F) {
        *self.on_edit.borrow_mut() = Some(Rc::new(callback));
    }

    pub fn widget(&self) -> gtk4::Box {
        self.container.clone()
    }

    pub fn refresh(&self) {
        self.populate();
        self.container.queue_resize();
        self.container.queue_draw();
    }

    fn populate(&self) {
        // Clear
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        let card = gtk4::Box::new(Orientation::Vertical, 0);
        card.add_css_class("card");

        // Header
        let header = gtk4::Box::new(Orientation::Horizontal, 8);
        header.set_margin_top(12);
        header.set_margin_start(12);
        header.set_margin_end(12);
        header.set_margin_bottom(8);

        let title = Label::new(Some(&gettext("Active rules")));
        title.add_css_class("heading");
        title.set_halign(gtk4::Align::Start);
        title.set_hexpand(true);
        header.append(&title);

        let count_label = {
            let cfg = self.config.borrow();
            let count = cfg.rules.len();
            Label::new(Some(&format!("{}", count)))
        };
        count_label.add_css_class("dim-label");
        header.append(&count_label);

        card.append(&header);

        // Rules
        let cfg = self.config.borrow();

        if cfg.rules.is_empty() {
            let empty_box = gtk4::Box::new(Orientation::Vertical, 8);
            empty_box.set_margin_top(16);
            empty_box.set_margin_bottom(24);

            let empty_icon = gtk4::Image::from_icon_name("security-medium-symbolic");
            empty_icon.set_pixel_size(48);
            empty_icon.add_css_class("dim-label");
            empty_box.append(&empty_icon);

            let empty_label = Label::new(Some(&gettext("No firewall rule configured")));
            empty_label.add_css_class("dim-label");
            empty_label.set_halign(gtk4::Align::Center);
            empty_box.append(&empty_label);

            let hint_label = Label::new(Some(&gettext(
                "Add rules with the button below or use presets",
            )));
            hint_label.add_css_class("dim-label");
            hint_label.add_css_class("caption");
            hint_label.set_halign(gtk4::Align::Center);
            empty_box.append(&hint_label);

            card.append(&empty_box);
        } else {
            let rules_box = gtk4::Box::new(Orientation::Vertical, 0);

            for (idx, rule) in cfg.rules.iter().enumerate() {
                if idx > 0 {
                    let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
                    rules_box.append(&sep);
                }

                let row = gtk4::Box::new(Orientation::Horizontal, 12);
                row.set_margin_top(8);
                row.set_margin_bottom(8);
                row.set_margin_start(12);
                row.set_margin_end(12);

                // Protocol badge
                let proto_label = Label::new(Some(&rule.protocol.to_string()));
                proto_label.add_css_class("caption");
                proto_label.add_css_class("accent");
                proto_label.set_width_request(60);
                row.append(&proto_label);

                // Rule info
                let info_box = gtk4::Box::new(Orientation::Vertical, 2);
                info_box.set_hexpand(true);

                let name_label = Label::new(Some(&rule.name));
                name_label.set_halign(gtk4::Align::Start);
                name_label.add_css_class("body");
                info_box.append(&name_label);

                let ports_label = Label::new(Some(&format!("Ports: {}", rule.ports)));
                ports_label.set_halign(gtk4::Align::Start);
                ports_label.add_css_class("caption");
                ports_label.add_css_class("dim-label");
                info_box.append(&ports_label);

                row.append(&info_box);

                // Edit button
                let edit_btn = Button::from_icon_name("document-edit-symbolic");
                edit_btn.add_css_class("flat");
                edit_btn.set_tooltip_text(Some(&gettext("Edit this rule")));
                edit_btn.set_valign(gtk4::Align::Center);

                let on_edit_clone = self.on_edit.clone();
                let edit_idx = idx;
                edit_btn.connect_clicked(move |_| {
                    if let Some(ref cb) = *on_edit_clone.borrow() {
                        cb(edit_idx);
                    }
                });

                row.append(&edit_btn);

                // Delete button
                let delete_btn = Button::from_icon_name("user-trash-symbolic");
                delete_btn.add_css_class("flat");
                delete_btn.add_css_class("destructive-action");
                delete_btn.set_tooltip_text(Some(&gettext("Remove this rule")));
                delete_btn.set_valign(gtk4::Align::Center);

                let config_for_delete = self.config.clone();
                let container_for_delete = self.container.clone();
                let on_edit_for_delete = self.on_edit.clone();
                let rule_idx = idx;

                delete_btn.connect_clicked(move |btn| {
                    let dialog = adw::MessageDialog::new(
                        btn.root()
                            .and_then(|r| r.downcast::<gtk4::Window>().ok())
                            .as_ref(),
                        Some(&gettext("Remove rule")),
                        Some(&gettext(
                            "Do you really want to remove this firewall rule?",
                        )),
                    );

                    dialog.add_response("cancel", &gettext("Cancel"));
                    dialog.add_response("confirm", &gettext("Remove"));
                    dialog
                        .set_response_appearance("confirm", adw::ResponseAppearance::Destructive);
                    dialog.set_default_response(Some("confirm"));
                    dialog.set_close_response("cancel");

                    let cfg_clone = config_for_delete.clone();
                    let container_clone = container_for_delete.clone();
                    let on_edit_clone = on_edit_for_delete.clone();

                    dialog.connect_response(None, move |_, response| {
                        if response == "confirm" {
                            let mut cfg = cfg_clone.borrow_mut();
                            if rule_idx < cfg.rules.len() {
                                cfg.rules.remove(rule_idx);
                            }
                            drop(cfg);

                            // Refresh by reconstructing
                            let temp = RulesListWidget {
                                container: container_clone.clone(),
                                config: cfg_clone.clone(),
                                on_edit: on_edit_clone.clone(),
                            };
                            temp.populate();
                        }
                    });

                    dialog.present();
                });

                row.append(&delete_btn);
                rules_box.append(&row);
            }

            card.append(&rules_box);
        }

        self.container.append(&card);
    }
}
