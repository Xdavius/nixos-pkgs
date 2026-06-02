use super::{FirewallRule, PortSpec, Protocol};

/// A preset category grouping related firewall rules
pub struct PresetCategory {
    pub name: String,
    pub rules: Vec<FirewallRule>,
}

/// Get all available preset categories
pub fn get_preset_categories() -> Vec<PresetCategory> {
    vec![
        PresetCategory {
            name: "Services courants".to_string(),
            rules: vec![
                FirewallRule::new("SSH", Protocol::Tcp, PortSpec::Single(22)),
                FirewallRule::new(
                    "HTTP / HTTPS",
                    Protocol::Tcp,
                    PortSpec::Multiple(vec![80, 443]),
                ),
                FirewallRule::new("DNS", Protocol::Udp, PortSpec::Single(53)),
            ],
        },
        PresetCategory {
            name: "Gaming".to_string(),
            rules: vec![
                FirewallRule::new("Steam", Protocol::Both, PortSpec::Range(27015, 27030)),
                FirewallRule::new(
                    "Sunshine / Moonlight",
                    Protocol::Both,
                    PortSpec::Range(47984, 47990),
                ),
                FirewallRule::new("Minecraft Serveur", Protocol::Tcp, PortSpec::Single(25565)),
            ],
        },
        PresetCategory {
            name: "Connectivite".to_string(),
            rules: vec![
                FirewallRule::new("KDE Connect", Protocol::Both, PortSpec::Range(1714, 1764)),
                FirewallRule::new(
                    "Syncthing",
                    Protocol::Tcp,
                    PortSpec::Multiple(vec![22000, 21027]),
                ),
                FirewallRule::new(
                    "Samba (TCP)",
                    Protocol::Tcp,
                    PortSpec::Multiple(vec![139, 445]),
                ),
                FirewallRule::new(
                    "Samba (UDP)",
                    Protocol::Udp,
                    PortSpec::Multiple(vec![137, 138]),
                ),
            ],
        },
    ]
}

/// Get all presets as a flat list
pub fn get_all_presets() -> Vec<FirewallRule> {
    get_preset_categories()
        .into_iter()
        .flat_map(|c| c.rules)
        .collect()
}

/// Known trusted interface presets
pub fn get_interface_presets() -> Vec<(&'static str, &'static str)> {
    vec![
        ("docker0", "Docker"),
        ("virbr0", "Libvirt / QEMU"),
        ("br0", "Bridge reseau"),
    ]
}
