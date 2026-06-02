use crate::models::{FirewallConfig, FirewallRule, PortSpec, Protocol};
use gettextrs::gettext;
use gtk4::prelude::*;
use gtk4::{Entry, Label, Orientation};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct AddRuleDialog {
    window: adw::Window,
    config: Rc<RefCell<FirewallConfig>>,
    on_added: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
}

impl AddRuleDialog {
    pub fn new(config: Rc<RefCell<FirewallConfig>>) -> Rc<Self> {
        Self::build(config, None)
    }

    pub fn new_edit(config: Rc<RefCell<FirewallConfig>>, index: usize, rule: &FirewallRule) -> Rc<Self> {
        Self::build(config, Some((index, rule.clone())))
    }

    fn build(config: Rc<RefCell<FirewallConfig>>, edit: Option<(usize, FirewallRule)>) -> Rc<Self> {
        let is_edit = edit.is_some();
        let window = adw::Window::builder()
            .modal(true)
            .default_width(450)
            .default_height(400)
            .build();

        let toolbar_view = adw::ToolbarView::new();

        let header = adw::HeaderBar::new();
        let title_text = if is_edit {
            gettext("Edit firewall rule")
        } else {
            gettext("Add a firewall rule")
        };
        header.set_title_widget(Some(&Label::new(Some(&title_text))));
        toolbar_view.add_top_bar(&header);

        let content = gtk4::Box::new(Orientation::Vertical, 16);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Name field
        let name_label = Label::new(Some(&gettext("Rule name")));
        name_label.set_halign(gtk4::Align::Start);
        name_label.add_css_class("heading");
        content.append(&name_label);

        let name_entry = Entry::builder()
            .placeholder_text("SSH, HTTP, Gaming...")
            .build();
        content.append(&name_entry);

        // Protocol selection
        let proto_label = Label::new(Some(&gettext("Protocol")));
        proto_label.set_halign(gtk4::Align::Start);
        proto_label.add_css_class("heading");
        proto_label.set_margin_top(8);
        content.append(&proto_label);

        let proto_box = gtk4::Box::new(Orientation::Horizontal, 8);
        let tcp_btn = gtk4::ToggleButton::with_label("TCP");
        let udp_btn = gtk4::ToggleButton::with_label("UDP");
        let both_btn = gtk4::ToggleButton::with_label("TCP+UDP");

        tcp_btn.set_active(true);
        udp_btn.set_group(Some(&tcp_btn));
        both_btn.set_group(Some(&tcp_btn));

        tcp_btn.add_css_class("pill");
        udp_btn.add_css_class("pill");
        both_btn.add_css_class("pill");

        proto_box.append(&tcp_btn);
        proto_box.append(&udp_btn);
        proto_box.append(&both_btn);
        content.append(&proto_box);

        // Port type selection
        let type_label = Label::new(Some(&gettext("Port type")));
        type_label.set_halign(gtk4::Align::Start);
        type_label.add_css_class("heading");
        type_label.set_margin_top(8);
        content.append(&type_label);

        let type_box = gtk4::Box::new(Orientation::Horizontal, 8);
        let single_btn = gtk4::ToggleButton::with_label(&gettext("Single port"));
        let range_btn = gtk4::ToggleButton::with_label(&gettext("Port range"));
        range_btn.set_group(Some(&single_btn));
        single_btn.set_active(true);

        single_btn.add_css_class("pill");
        range_btn.add_css_class("pill");

        type_box.append(&single_btn);
        type_box.append(&range_btn);
        content.append(&type_box);

        // Port input (single)
        let port_box = gtk4::Box::new(Orientation::Horizontal, 8);
        port_box.set_margin_top(8);

        let port_entry = Entry::builder()
            .placeholder_text("22, 80, 443...")
            .hexpand(true)
            .build();
        port_box.append(&port_entry);

        // Range inputs (hidden by default)
        let range_box = gtk4::Box::new(Orientation::Horizontal, 8);
        range_box.set_margin_top(8);
        range_box.set_visible(false);

        let from_entry = Entry::builder()
            .placeholder_text(&gettext("From"))
            .hexpand(true)
            .build();
        let to_label = Label::new(Some("-"));
        let to_entry = Entry::builder()
            .placeholder_text(&gettext("To"))
            .hexpand(true)
            .build();
        range_box.append(&from_entry);
        range_box.append(&to_label);
        range_box.append(&to_entry);

        content.append(&port_box);
        content.append(&range_box);

        // Pre-fill fields if editing an existing rule
        if let Some((_, ref rule)) = edit {
            name_entry.set_text(&rule.name);
            match rule.protocol {
                Protocol::Tcp => tcp_btn.set_active(true),
                Protocol::Udp => udp_btn.set_active(true),
                Protocol::Both => both_btn.set_active(true),
            }
            match &rule.ports {
                PortSpec::Single(p) => {
                    single_btn.set_active(true);
                    port_entry.set_text(&p.to_string());
                }
                PortSpec::Multiple(ports) => {
                    single_btn.set_active(true);
                    let text: Vec<String> = ports.iter().map(|p| p.to_string()).collect();
                    port_entry.set_text(&text.join(", "));
                }
                PortSpec::Range(from, to) => {
                    range_btn.set_active(true);
                    port_box.set_visible(false);
                    range_box.set_visible(true);
                    from_entry.set_text(&from.to_string());
                    to_entry.set_text(&to.to_string());
                }
            }
        }

        // Toggle visibility based on type selection
        let port_box_clone = port_box.clone();
        let range_box_clone = range_box.clone();
        single_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                port_box_clone.set_visible(true);
                range_box_clone.set_visible(false);
            }
        });

        let port_box_clone2 = port_box.clone();
        let range_box_clone2 = range_box.clone();
        range_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                port_box_clone2.set_visible(false);
                range_box_clone2.set_visible(true);
            }
        });

        // Error label
        let error_label = Label::new(None);
        error_label.add_css_class("error");
        error_label.set_visible(false);
        content.append(&error_label);

        // Buttons
        let btn_box = gtk4::Box::new(Orientation::Horizontal, 12);
        btn_box.set_halign(gtk4::Align::End);
        btn_box.set_margin_top(16);

        let cancel_btn = gtk4::Button::with_label(&gettext("Cancel"));
        cancel_btn.add_css_class("pill");

        let add_btn = gtk4::Button::new();
        let (btn_icon, btn_label) = if is_edit {
            ("document-edit-symbolic", gettext("Save"))
        } else {
            ("list-add-symbolic", gettext("Add"))
        };
        let add_content = adw::ButtonContent::builder()
            .icon_name(btn_icon)
            .label(&btn_label)
            .build();
        add_btn.set_child(Some(&add_content));
        add_btn.add_css_class("pill");
        add_btn.add_css_class("suggested-action");

        btn_box.append(&cancel_btn);
        btn_box.append(&add_btn);
        content.append(&btn_box);

        toolbar_view.set_content(Some(&content));
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

        // Add / Save
        let config_for_add = config.clone();
        let window_for_add = window.clone();
        let on_added_for_add = on_added.clone();
        let edit_index = edit.map(|(idx, _)| idx);
        add_btn.connect_clicked(move |_| {
            let name = name_entry.text().to_string().trim().to_string();
            if name.is_empty() {
                error_label.set_text(&gettext("Please enter a rule name"));
                error_label.set_visible(true);
                return;
            }

            let protocol = if tcp_btn.is_active() {
                Protocol::Tcp
            } else if udp_btn.is_active() {
                Protocol::Udp
            } else {
                Protocol::Both
            };

            let ports = if single_btn.is_active() {
                let text = port_entry.text().to_string().trim().to_string();
                // Support comma-separated ports
                let parsed: Result<Vec<u16>, _> = text
                    .split(|c: char| c == ',' || c == ' ')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.trim().parse::<u16>())
                    .collect();

                match parsed {
                    Ok(ports) if ports.is_empty() => {
                        error_label.set_text(&gettext("Please enter at least one port number"));
                        error_label.set_visible(true);
                        return;
                    }
                    Ok(ports) if ports.len() == 1 => PortSpec::Single(ports[0]),
                    Ok(ports) => PortSpec::Multiple(ports),
                    Err(_) => {
                        error_label.set_text(&gettext("Invalid port number (1-65535)"));
                        error_label.set_visible(true);
                        return;
                    }
                }
            } else {
                let from_text = from_entry.text().to_string().trim().to_string();
                let to_text = to_entry.text().to_string().trim().to_string();

                match (from_text.parse::<u16>(), to_text.parse::<u16>()) {
                    (Ok(from), Ok(to)) if from < to => PortSpec::Range(from, to),
                    _ => {
                        error_label.set_text(&gettext(
                            "Invalid port range (from must be less than to)",
                        ));
                        error_label.set_visible(true);
                        return;
                    }
                }
            };

            let rule = FirewallRule::new(&name, protocol, ports);

            let mut cfg = config_for_add.borrow_mut();
            if let Some(idx) = edit_index {
                if idx < cfg.rules.len() {
                    cfg.rules[idx] = rule;
                }
            } else {
                cfg.rules.push(rule);
            }
            drop(cfg);

            if let Some(ref cb) = *on_added_for_add.borrow() {
                cb();
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
