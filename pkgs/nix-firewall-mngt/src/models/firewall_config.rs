/// Represents a TCP/UDP port range
#[derive(Debug, Clone, PartialEq)]
pub struct PortRange {
    pub from: u16,
    pub to: u16,
}

/// Protocol type for firewall rules
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
    Both,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
            Protocol::Both => write!(f, "TCP+UDP"),
        }
    }
}

/// Specification of ports for a rule
#[derive(Debug, Clone, PartialEq)]
pub enum PortSpec {
    Single(u16),
    Range(u16, u16),
    Multiple(Vec<u16>),
}

impl std::fmt::Display for PortSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortSpec::Single(p) => write!(f, "{}", p),
            PortSpec::Range(from, to) => write!(f, "{}-{}", from, to),
            PortSpec::Multiple(ports) => {
                let parts: Vec<String> = ports.iter().map(|p| p.to_string()).collect();
                write!(f, "{}", parts.join(", "))
            }
        }
    }
}

/// A named firewall rule for display in the UI
#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub name: String,
    pub protocol: Protocol,
    pub ports: PortSpec,
    pub enabled: bool,
}

impl FirewallRule {
    pub fn new(name: &str, protocol: Protocol, ports: PortSpec) -> Self {
        Self {
            name: name.to_string(),
            protocol,
            ports,
            enabled: true,
        }
    }

    /// Get a display string like "SSH (TCP 22)"
    pub fn display_label(&self) -> String {
        format!("{} ({} {})", self.name, self.protocol, self.ports)
    }

    /// Extract individual TCP ports from this rule
    pub fn tcp_ports(&self) -> Vec<u16> {
        match self.protocol {
            Protocol::Udp => Vec::new(),
            _ => match &self.ports {
                PortSpec::Single(p) => vec![*p],
                PortSpec::Multiple(ports) => ports.clone(),
                PortSpec::Range(_, _) => Vec::new(), // ranges handled separately
            },
        }
    }

    /// Extract individual UDP ports from this rule
    pub fn udp_ports(&self) -> Vec<u16> {
        match self.protocol {
            Protocol::Tcp => Vec::new(),
            _ => match &self.ports {
                PortSpec::Single(p) => vec![*p],
                PortSpec::Multiple(ports) => ports.clone(),
                PortSpec::Range(_, _) => Vec::new(),
            },
        }
    }

    /// Extract TCP port ranges from this rule
    pub fn tcp_ranges(&self) -> Vec<PortRange> {
        match self.protocol {
            Protocol::Udp => Vec::new(),
            _ => match &self.ports {
                PortSpec::Range(from, to) => vec![PortRange { from: *from, to: *to }],
                _ => Vec::new(),
            },
        }
    }

    /// Extract UDP port ranges from this rule
    pub fn udp_ranges(&self) -> Vec<PortRange> {
        match self.protocol {
            Protocol::Tcp => Vec::new(),
            _ => match &self.ports {
                PortSpec::Range(from, to) => vec![PortRange { from: *from, to: *to }],
                _ => Vec::new(),
            },
        }
    }
}

/// The full firewall configuration, maps to NixOS networking.firewall options
#[derive(Debug, Clone)]
pub struct FirewallConfig {
    pub enabled: bool,
    pub rules: Vec<FirewallRule>,
    pub trusted_interfaces: Vec<String>,
}

impl Default for FirewallConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: Vec::new(),
            trusted_interfaces: Vec::new(),
        }
    }
}

impl FirewallConfig {
    /// Collect all TCP ports across all rules (deduped & sorted)
    pub fn all_tcp_ports(&self) -> Vec<u16> {
        let mut ports: Vec<u16> = self
            .rules
            .iter()
            .filter(|r| r.enabled)
            .flat_map(|r| r.tcp_ports())
            .collect();
        ports.sort();
        ports.dedup();
        ports
    }

    /// Collect all UDP ports across all rules (deduped & sorted)
    pub fn all_udp_ports(&self) -> Vec<u16> {
        let mut ports: Vec<u16> = self
            .rules
            .iter()
            .filter(|r| r.enabled)
            .flat_map(|r| r.udp_ports())
            .collect();
        ports.sort();
        ports.dedup();
        ports
    }

    /// Collect all TCP port ranges across all rules
    pub fn all_tcp_ranges(&self) -> Vec<PortRange> {
        self.rules
            .iter()
            .filter(|r| r.enabled)
            .flat_map(|r| r.tcp_ranges())
            .collect()
    }

    /// Collect all UDP port ranges across all rules
    pub fn all_udp_ranges(&self) -> Vec<PortRange> {
        self.rules
            .iter()
            .filter(|r| r.enabled)
            .flat_map(|r| r.udp_ranges())
            .collect()
    }
}
