mod models;
mod ui;
mod utils;

use ui::app::NixFirewallApp;

fn main() {
    // Initialize gettext
    let locale_dir = option_env!("LOCALE_DIR").unwrap_or("/usr/share/locale");
    gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain("nix-firewall-mngt", locale_dir).expect("Failed to bind textdomain");
    gettextrs::textdomain("nix-firewall-mngt").expect("Failed to set textdomain");

    let app = NixFirewallApp::new();
    std::process::exit(app.run());
}
