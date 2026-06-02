use crate::models::FirewallConfig;
use crate::ui::window::NixFirewallWindow;
use crate::utils::{parse_firewall_nix, default_nix_has_firewall_import};
use gtk4::prelude::*;
use gtk4::glib;
use libadwaita as adw;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

const FIREWALL_NIX_PATH: &str = "/etc/nixos/customConfig/firewall.nix";
const DEFAULT_NIX_PATH: &str = "/etc/nixos/customConfig/default.nix";

/// Load saved theme preference from XDG config directory.
/// Returns the adw::ColorScheme to apply.
pub fn load_theme_preference() -> adw::ColorScheme {
    let config_dir = glib::user_config_dir().join("nix-firewall-mngt");
    let config_file = config_dir.join("theme.conf");

    if let Ok(content) = fs::read_to_string(&config_file) {
        match content.trim() {
            "light" => adw::ColorScheme::ForceLight,
            "dark" => adw::ColorScheme::ForceDark,
            _ => adw::ColorScheme::Default,
        }
    } else {
        adw::ColorScheme::Default
    }
}

/// Save theme preference to XDG config directory.
pub fn save_theme_preference(scheme: adw::ColorScheme) {
    let config_dir = glib::user_config_dir().join("nix-firewall-mngt");
    let _ = fs::create_dir_all(&config_dir);
    let config_file = config_dir.join("theme.conf");

    let value = match scheme {
        adw::ColorScheme::ForceLight => "light",
        adw::ColorScheme::ForceDark => "dark",
        _ => "system",
    };
    let _ = fs::write(&config_file, value);
}

pub struct NixFirewallApp {
    app: adw::Application,
}

impl NixFirewallApp {
    pub fn new() -> Self {
        let app = adw::Application::builder()
            .application_id("org.glfos.nixfirewall")
            .build();

        glib::set_application_name("Nix Firewall Manager");
        glib::set_prgname(Some("nix-firewall-mngt"));

        app.connect_activate(move |app| {
            Self::on_activate(app);
        });

        Self { app }
    }

    fn on_activate(app: &adw::Application) {
        if let Some(settings) = gtk4::Settings::default() {
            settings.set_property("gtk-icon-theme-name", "Adwaita");
        }

        // Apply saved theme preference (works on GNOME via libadwaita,
        // and on KDE Plasma via XDG Desktop Portal color-scheme)
        let style_manager = adw::StyleManager::default();
        let saved_scheme = load_theme_preference();
        style_manager.set_color_scheme(saved_scheme);

        let firewall_path = PathBuf::from(FIREWALL_NIX_PATH);
        let default_nix_path = PathBuf::from(DEFAULT_NIX_PATH);

        // Load or create firewall config
        let config = if firewall_path.exists() {
            match fs::read_to_string(&firewall_path) {
                Ok(content) => match parse_firewall_nix(&content) {
                    Ok(cfg) => {
                        eprintln!("Firewall config loaded: {} rules", cfg.rules.len());
                        cfg
                    }
                    Err(e) => {
                        eprintln!("Failed to parse firewall.nix: {}", e);
                        FirewallConfig::default()
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read firewall.nix: {}", e);
                    FirewallConfig::default()
                }
            }
        } else {
            eprintln!("No firewall.nix found, starting with default config");
            FirewallConfig::default()
        };

        // Check if default.nix needs the import
        let needs_import = if default_nix_path.exists() {
            match fs::read_to_string(&default_nix_path) {
                Ok(content) => !default_nix_has_firewall_import(&content),
                Err(_) => false,
            }
        } else {
            false
        };

        let config = Rc::new(RefCell::new(config));
        let needs_import = Rc::new(RefCell::new(needs_import));

        let window = NixFirewallWindow::new(
            app,
            config,
            firewall_path,
            default_nix_path,
            needs_import,
        );

        window.present();
    }

    pub fn run(&self) -> i32 {
        self.app.run().into()
    }
}
