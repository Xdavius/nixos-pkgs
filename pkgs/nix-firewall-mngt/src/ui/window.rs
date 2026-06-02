use crate::models::FirewallConfig;
use crate::ui::app::save_theme_preference;
use crate::ui::dialogs::{AddRuleDialog, PresetsDialog};
use crate::ui::widgets::RulesListWidget;
use crate::utils::{generate_firewall_nix, inject_firewall_import};
use gettextrs::gettext;
use gtk4::prelude::*;
use gtk4::{gio, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

pub struct NixFirewallWindow {
    window: adw::ApplicationWindow,
    config: Rc<RefCell<FirewallConfig>>,
    firewall_path: PathBuf,
    default_nix_path: PathBuf,
    needs_import: Rc<RefCell<bool>>,
    rebuild_banner: adw::Banner,
    rebuild_error_banner: adw::Banner,
    toast_overlay: adw::ToastOverlay,
    rules_widget: RulesListWidget,
}

impl NixFirewallWindow {
    pub fn new(
        app: &adw::Application,
        config: Rc<RefCell<FirewallConfig>>,
        firewall_path: PathBuf,
        default_nix_path: PathBuf,
        needs_import: Rc<RefCell<bool>>,
    ) -> Rc<Self> {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Nix Firewall Manager")
            .default_width(700)
            .default_height(600)
            .icon_name("nix-firewall-mngt")
            .resizable(true)
            .build();

        // Main layout
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        // Banners
        let rebuild_banner = adw::Banner::new(&gettext("Rebuilding NixOS configuration..."));
        rebuild_banner.set_revealed(false);

        let rebuild_error_banner =
            adw::Banner::new(&gettext("Failed to rebuild NixOS configuration"));
        rebuild_error_banner.set_revealed(false);
        rebuild_error_banner.add_css_class("error");

        main_box.append(&rebuild_banner);
        main_box.append(&rebuild_error_banner);

        // Header bar
        let header_bar = adw::HeaderBar::new();

        // Theme menu button (System / Light / Dark)
        let theme_menu = gio::Menu::new();
        theme_menu.append(Some(&gettext("System")), Some("app.theme-system"));
        theme_menu.append(Some(&gettext("Light")), Some("app.theme-light"));
        theme_menu.append(Some(&gettext("Dark")), Some("app.theme-dark"));

        let menu_btn = gtk4::MenuButton::new();
        menu_btn.set_icon_name("display-brightness-symbolic");
        menu_btn.set_menu_model(Some(&theme_menu));
        menu_btn.set_tooltip_text(Some(&gettext("Appearance")));
        header_bar.pack_end(&menu_btn);

        // Register theme actions
        let action_system = gio::SimpleAction::new("theme-system", None);
        action_system.connect_activate(|_, _| {
            let sm = adw::StyleManager::default();
            sm.set_color_scheme(adw::ColorScheme::Default);
            save_theme_preference(adw::ColorScheme::Default);
        });

        let action_light = gio::SimpleAction::new("theme-light", None);
        action_light.connect_activate(|_, _| {
            let sm = adw::StyleManager::default();
            sm.set_color_scheme(adw::ColorScheme::ForceLight);
            save_theme_preference(adw::ColorScheme::ForceLight);
        });

        let action_dark = gio::SimpleAction::new("theme-dark", None);
        action_dark.connect_activate(|_, _| {
            let sm = adw::StyleManager::default();
            sm.set_color_scheme(adw::ColorScheme::ForceDark);
            save_theme_preference(adw::ColorScheme::ForceDark);
        });

        app.add_action(&action_system);
        app.add_action(&action_light);
        app.add_action(&action_dark);

        main_box.append(&header_bar);

        // Toast overlay
        let toast_overlay = adw::ToastOverlay::new();

        // Scrolled content
        let scrolled = gtk4::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content_box.set_margin_top(24);
        content_box.set_margin_bottom(24);
        content_box.set_margin_start(24);
        content_box.set_margin_end(24);

        // ── Firewall enable/disable toggle ──
        let toggle_card = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        toggle_card.add_css_class("card");
        toggle_card.set_margin_bottom(8);

        let shield_icon = gtk4::Image::from_icon_name("security-high-symbolic");
        shield_icon.set_pixel_size(32);
        shield_icon.set_margin_top(16);
        shield_icon.set_margin_bottom(16);
        shield_icon.set_margin_start(16);
        toggle_card.append(&shield_icon);

        let toggle_label_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        toggle_label_box.set_valign(gtk4::Align::Center);
        toggle_label_box.set_hexpand(true);

        let toggle_title = gtk4::Label::new(Some(&gettext("Firewall")));
        toggle_title.add_css_class("title-3");
        toggle_title.set_halign(gtk4::Align::Start);
        toggle_label_box.append(&toggle_title);

        let toggle_subtitle = gtk4::Label::new(Some(&gettext("Enable NixOS firewall protection")));
        toggle_subtitle.add_css_class("dim-label");
        toggle_subtitle.add_css_class("caption");
        toggle_subtitle.set_halign(gtk4::Align::Start);
        toggle_label_box.append(&toggle_subtitle);

        toggle_card.append(&toggle_label_box);

        let firewall_switch = gtk4::Switch::new();
        firewall_switch.set_active(config.borrow().enabled);
        firewall_switch.set_valign(gtk4::Align::Center);
        firewall_switch.set_margin_end(16);

        let config_for_switch = config.clone();
        firewall_switch.connect_state_set(move |_, state| {
            config_for_switch.borrow_mut().enabled = state;
            glib::Propagation::Proceed
        });

        toggle_card.append(&firewall_switch);
        content_box.append(&toggle_card);

        // ── Rules list widget ──
        let rules_widget = RulesListWidget::new(config.clone());
        content_box.append(&rules_widget.widget());

        // ── Trusted interfaces card ──
        let ifaces_card = Self::build_interfaces_card(config.clone());
        content_box.append(&ifaces_card);

        scrolled.set_child(Some(&content_box));
        toast_overlay.set_child(Some(&scrolled));
        main_box.append(&toast_overlay);

        // ── Fixed bottom action bar (outside ScrolledWindow) ──
        let action_bar = gtk4::ActionBar::new();

        let add_rule_btn = gtk4::Button::new();
        let add_content = adw::ButtonContent::builder()
            .icon_name("list-add-symbolic")
            .label(&gettext("Add a rule"))
            .build();
        add_rule_btn.set_child(Some(&add_content));
        add_rule_btn.add_css_class("pill");

        let presets_btn = gtk4::Button::new();
        let presets_content = adw::ButtonContent::builder()
            .icon_name("view-list-symbolic")
            .label(&gettext("Presets"))
            .build();
        presets_btn.set_child(Some(&presets_content));
        presets_btn.add_css_class("pill");

        action_bar.pack_start(&add_rule_btn);
        action_bar.pack_start(&presets_btn);

        let save_btn = gtk4::Button::new();
        let save_content = adw::ButtonContent::builder()
            .icon_name("document-save-symbolic")
            .label(&gettext("Save and Rebuild"))
            .build();
        save_btn.set_child(Some(&save_content));
        save_btn.add_css_class("pill");
        save_btn.add_css_class("suggested-action");

        action_bar.pack_end(&save_btn);

        main_box.append(&action_bar);
        window.set_content(Some(&main_box));

        let window_rc = Rc::new(Self {
            window: window.clone(),
            config: config.clone(),
            firewall_path: firewall_path.clone(),
            default_nix_path: default_nix_path.clone(),
            needs_import: needs_import.clone(),
            rebuild_banner: rebuild_banner.clone(),
            rebuild_error_banner: rebuild_error_banner.clone(),
            toast_overlay: toast_overlay.clone(),
            rules_widget: rules_widget.clone(),
        });

        // Connect edit rule callback
        let config_for_edit = config.clone();
        let rules_widget_for_edit = rules_widget.clone();
        let window_for_edit = window.clone();
        rules_widget.set_on_edit(move |index| {
            let rule = {
                let cfg = config_for_edit.borrow();
                if index < cfg.rules.len() {
                    Some(cfg.rules[index].clone())
                } else {
                    None
                }
            };
            if let Some(rule) = rule {
                let dialog = AddRuleDialog::new_edit(config_for_edit.clone(), index, &rule);
                let rw = rules_widget_for_edit.clone();
                dialog.set_on_added(move || {
                    rw.refresh();
                });
                dialog.present(Some(&window_for_edit));
            }
        });

        // Connect add rule button
        let config_for_add = config.clone();
        let rules_widget_for_add = rules_widget.clone();
        let window_for_add = window.clone();
        add_rule_btn.connect_clicked(move |_| {
            let dialog = AddRuleDialog::new(config_for_add.clone());
            let rw = rules_widget_for_add.clone();
            dialog.set_on_added(move || {
                rw.refresh();
            });
            dialog.present(Some(&window_for_add));
        });

        // Connect presets button
        let config_for_presets = config.clone();
        let rules_widget_for_presets = rules_widget.clone();
        let window_for_presets = window.clone();
        presets_btn.connect_clicked(move |_| {
            let dialog = PresetsDialog::new(config_for_presets.clone());
            let rw = rules_widget_for_presets.clone();
            dialog.set_on_added(move || {
                rw.refresh();
            });
            dialog.present(Some(&window_for_presets));
        });

        // Connect save button
        let wrc = window_rc.clone();
        save_btn.connect_clicked(move |_| {
            wrc.do_save_and_rebuild();
        });

        window_rc
    }

    fn build_interfaces_card(config: Rc<RefCell<FirewallConfig>>) -> gtk4::Box {
        let card = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        card.add_css_class("card");

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        header.set_margin_top(12);
        header.set_margin_start(12);
        header.set_margin_end(12);

        let title = gtk4::Label::new(Some(&gettext("Trusted interfaces")));
        title.add_css_class("heading");
        title.set_halign(gtk4::Align::Start);
        title.set_hexpand(true);
        header.append(&title);

        card.append(&header);

        let ifaces_list = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        ifaces_list.set_margin_start(12);
        ifaces_list.set_margin_end(12);
        ifaces_list.set_margin_bottom(8);

        let cfg = config.borrow();
        if cfg.trusted_interfaces.is_empty() {
            let empty_label = gtk4::Label::new(Some(&gettext("No trusted interface configured")));
            empty_label.add_css_class("dim-label");
            empty_label.set_halign(gtk4::Align::Start);
            ifaces_list.append(&empty_label);
        } else {
            for iface in &cfg.trusted_interfaces {
                let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

                let label = gtk4::Label::new(Some(iface));
                label.set_halign(gtk4::Align::Start);
                label.set_hexpand(true);
                row.append(&label);

                let remove_btn = gtk4::Button::from_icon_name("user-trash-symbolic");
                remove_btn.add_css_class("flat");
                remove_btn.add_css_class("destructive-action");
                remove_btn.set_tooltip_text(Some(&gettext("Remove this interface")));

                let iface_clone = iface.clone();
                let config_for_remove = config.clone();
                let ifaces_list_clone = ifaces_list.clone();
                remove_btn.connect_clicked(move |_| {
                    config_for_remove
                        .borrow_mut()
                        .trusted_interfaces
                        .retain(|i| i != &iface_clone);
                    // Simple refresh: remove this row
                    while let Some(child) = ifaces_list_clone.first_child() {
                        ifaces_list_clone.remove(&child);
                    }
                    let empty_label =
                        gtk4::Label::new(Some(&gettext("No trusted interface configured")));
                    empty_label.add_css_class("dim-label");
                    empty_label.set_halign(gtk4::Align::Start);
                    ifaces_list_clone.append(&empty_label);
                });

                row.append(&remove_btn);
                ifaces_list.append(&row);
            }
        }

        card.append(&ifaces_list);

        // Add interface input
        let add_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        add_box.set_margin_start(12);
        add_box.set_margin_end(12);
        add_box.set_margin_bottom(12);

        let entry = gtk4::Entry::builder()
            .placeholder_text("docker0, virbr0, br0...")
            .hexpand(true)
            .build();

        let add_btn = gtk4::Button::from_icon_name("list-add-symbolic");
        add_btn.add_css_class("circular");
        add_btn.add_css_class("suggested-action");
        add_btn.set_tooltip_text(Some(&gettext("Add trusted interface")));

        let config_for_add = config.clone();
        let entry_clone = entry.clone();
        let add_handler = move |_: &gtk4::Button| {
            let text = entry_clone.text().to_string().trim().to_string();
            if !text.is_empty() {
                let mut cfg = config_for_add.borrow_mut();
                if !cfg.trusted_interfaces.contains(&text) {
                    cfg.trusted_interfaces.push(text);
                    entry_clone.set_text("");
                }
            }
        };

        add_btn.connect_clicked(add_handler.clone());

        let add_btn_for_enter = add_btn.clone();
        entry.connect_activate(move |_| {
            add_btn_for_enter.emit_clicked();
        });

        add_box.append(&entry);
        add_box.append(&add_btn);
        card.append(&add_box);

        card
    }

    fn do_save_and_rebuild(&self) {
        eprintln!("=== Saving firewall configuration ===");

        let config = self.config.borrow().clone();
        let nix_content = generate_firewall_nix(&config);

        eprintln!("Generated firewall.nix:\n{}", nix_content);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Write firewall.nix to /tmp (no root needed)
        let tmp_firewall = format!("/tmp/nix_firewall_{}.nix", timestamp);
        if let Err(e) = fs::write(&tmp_firewall, &nix_content) {
            eprintln!("Failed to write temp firewall.nix: {}", e);
            self.rebuild_error_banner.set_revealed(true);
            return;
        }
        eprintln!("Temp firewall.nix written to {}", tmp_firewall);

        // Prepare modified default.nix if import injection is needed
        let tmp_default = format!("/tmp/nix_firewall_default_{}.nix", timestamp);
        let inject_default = *self.needs_import.borrow();
        if inject_default {
            match fs::read_to_string(&self.default_nix_path) {
                Ok(content) => match inject_firewall_import(&content) {
                    Ok(new_content) => {
                        if let Err(e) = fs::write(&tmp_default, &new_content) {
                            eprintln!("Failed to write temp default.nix: {}", e);
                            let _ = fs::remove_file(&tmp_firewall);
                            self.rebuild_error_banner.set_revealed(true);
                            return;
                        }
                        eprintln!("Temp default.nix written to {}", tmp_default);
                    }
                    Err(e) => {
                        eprintln!("Failed to inject import: {}", e);
                        let _ = fs::remove_file(&tmp_firewall);
                        self.rebuild_error_banner.set_revealed(true);
                        return;
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read default.nix: {}", e);
                    let _ = fs::remove_file(&tmp_firewall);
                    self.rebuild_error_banner.set_revealed(true);
                    return;
                }
            }
        }

        // Launch nixos-rebuild via terminal with sudo
        self.rebuild_error_banner.set_revealed(false);
        self.rebuild_banner.set_revealed(true);

        let firewall_dest = self.firewall_path.display().to_string();
        let default_dest = self.default_nix_path.display().to_string();
        let needs_import_rc = self.needs_import.clone();
        let rebuild_banner = self.rebuild_banner.clone();
        let rebuild_error_banner = self.rebuild_error_banner.clone();
        let toast_overlay = self.toast_overlay.clone();

        glib::spawn_future_local(async move {
            let result = gio::spawn_blocking(move || {
                let wrapper_path = format!("/tmp/nix_firewall_rebuild_{}.sh", timestamp);
                let status_file = format!("/tmp/nix_firewall_rebuild_{}.done", timestamp);

                // Build copy commands for the script
                let mut copy_cmds = format!(
                    "sudo cp \"{}\" \"{}\"",
                    tmp_firewall, firewall_dest
                );
                if inject_default {
                    copy_cmds.push_str(&format!(
                        " && sudo cp \"{}\" \"{}\"",
                        tmp_default, default_dest
                    ));
                }

                let script_content = format!(
                    r#"#!/usr/bin/env bash

echo "======================================"
echo "  RECONSTRUCTION DE LA CONFIGURATION"
echo "  FIREWALL NIX FIREWALL MANAGER"
echo "======================================"
echo ""

echo "Copie des fichiers de configuration..."
{copy_cmds}
COPY_CODE=$?

if [ $COPY_CODE -ne 0 ]; then
    echo ""
    echo "ERREUR: Impossible de copier les fichiers de configuration."
    echo ""
    echo "Vous pouvez fermer cette console."
    exit 1
fi

echo "Fichiers copies. Lancement de nixos-rebuild..."
echo ""

# Detect flake configuration name for --flake flag
FLAKE_ATTR=""
if [ -f /etc/nixos/flake.nix ]; then
    FLAKE_ATTR=$(grep -oP 'nixosConfigurations\.\s*"?\K[^"= ]+' /etc/nixos/flake.nix | head -1)
fi

if [ -n "$FLAKE_ATTR" ]; then
    echo "Configuration flake detectee : $FLAKE_ATTR"
    sudo nixos-rebuild switch --flake "/etc/nixos#$FLAKE_ATTR"
else
    sudo nixos-rebuild switch
fi
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo ""
    echo "======================================"
    echo "  REBUILD TERMINE AVEC SUCCES"
    echo "======================================"
    touch {status_file}
else
    echo ""
    echo "======================================"
    echo "  ERREUR LORS DU REBUILD"
    echo "======================================"
fi

# Nettoyage des fichiers temporaires
rm -f "{tmp_firewall}"
rm -f "{tmp_default}"

echo ""
echo "Vous pouvez fermer cette console."
"#,
                    copy_cmds = copy_cmds,
                    status_file = status_file,
                    tmp_firewall = tmp_firewall,
                    tmp_default = tmp_default,
                );

                if let Err(e) = std::fs::write(&wrapper_path, &script_content) {
                    eprintln!("Failed to write rebuild script: {}", e);
                    return (false, status_file.clone(), wrapper_path.clone(), inject_default);
                }

                if let Err(e) = Command::new("chmod").arg("+x").arg(&wrapper_path).status() {
                    eprintln!("chmod error: {}", e);
                    let _ = std::fs::remove_file(&wrapper_path);
                    return (false, status_file.clone(), wrapper_path.clone(), inject_default);
                }

                // Try terminals in order with full NixOS paths
                let terminals: Vec<(&str, Vec<&str>)> = vec![
                    ("kgx", vec!["--", &wrapper_path]),
                    ("gnome-terminal", vec!["--", &wrapper_path]),
                    ("konsole", vec!["-e", &wrapper_path]),
                    ("xfce4-terminal", vec!["-e", &wrapper_path]),
                    ("alacritty", vec!["-e", &wrapper_path]),
                    ("kitty", vec![&wrapper_path]),
                    ("xterm", vec!["-e", &wrapper_path]),
                ];

                for (term, args) in &terminals {
                    eprintln!("Trying terminal {}...", term);
                    if Command::new(term).args(args).spawn().is_ok() {
                        eprintln!("Terminal {} opened", term);
                        return (true, status_file, wrapper_path, inject_default);
                    }
                }

                eprintln!("No terminal found for nixos-rebuild");
                let _ = std::fs::remove_file(&wrapper_path);
                (false, status_file, wrapper_path, inject_default)
            })
            .await
            .unwrap_or((false, String::new(), String::new(), false));

            let (terminal_opened, status_file_path, script_path, did_inject) = result;

            if !terminal_opened {
                rebuild_banner.set_revealed(false);
                rebuild_error_banner.set_revealed(true);
            } else {
                // Mark import as done if we injected it
                if did_inject {
                    *needs_import_rc.borrow_mut() = false;
                }

                // Watch for completion
                let rebuild_banner_watch = rebuild_banner.clone();
                let toast_watch = toast_overlay.clone();
                let check_count = Rc::new(RefCell::new(0u32));

                glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
                    *check_count.borrow_mut() += 1;
                    let count = *check_count.borrow();

                    if std::path::Path::new(&status_file_path).exists() {
                        eprintln!("Rebuild completed!");
                        rebuild_banner_watch.set_revealed(false);

                        let toast = adw::Toast::new(&gettext("Firewall configuration applied successfully"));
                        toast.set_timeout(3);
                        toast_watch.add_toast(toast);

                        let _ = std::fs::remove_file(&status_file_path);
                        let _ = std::fs::remove_file(&script_path);

                        return glib::ControlFlow::Break;
                    }

                    // Timeout after 10 minutes
                    if count > 300 {
                        eprintln!("Rebuild watcher timeout");
                        rebuild_banner_watch.set_revealed(false);
                        let _ = std::fs::remove_file(&script_path);
                        return glib::ControlFlow::Break;
                    }

                    glib::ControlFlow::Continue
                });
            }
        });
    }

    pub fn present(&self) {
        self.window.present();
    }
}
