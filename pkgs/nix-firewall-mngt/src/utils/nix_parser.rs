use anyhow::Result;
use regex::Regex;

use crate::models::{FirewallConfig, FirewallRule, PortRange, PortSpec, Protocol};

/// Metadata for a rule name, parsed from nfm comments
struct RuleNameMeta {
    name: String,
    proto: String,
    ports: String,
}

/// Parse nfm:rule comments to recover rule names
fn parse_nfm_comments(content: &str) -> Vec<RuleNameMeta> {
    let re = Regex::new(r"# nfm:rule:([^:]+):([^:]+):(.+)").unwrap();
    re.captures_iter(content)
        .map(|cap| RuleNameMeta {
            name: cap[1].to_string(),
            proto: cap[2].to_string(),
            ports: cap[3].to_string(),
        })
        .collect()
}

/// Find a saved rule name for a given protocol and port spec
fn find_saved_name(metas: &[RuleNameMeta], proto: &str, ports_str: &str) -> Option<String> {
    metas
        .iter()
        .find(|m| m.proto == proto && m.ports == ports_str)
        .map(|m| m.name.clone())
}

/// Parse a firewall.nix file content into a FirewallConfig
pub fn parse_firewall_nix(content: &str) -> Result<FirewallConfig> {
    let mut config = FirewallConfig::default();

    // Parse saved rule name metadata from nfm comments
    let metas = parse_nfm_comments(content);

    // Parse enable = true|false
    let enable_re = Regex::new(r"enable\s*=\s*(true|false)\s*;")?;
    if let Some(cap) = enable_re.captures(content) {
        config.enabled = &cap[1] == "true";
    }

    // Parse allowedTCPPorts
    let tcp_ports = parse_port_list(content, "allowedTCPPorts")?;
    for port in &tcp_ports {
        let name = find_saved_name(&metas, "tcp", &port.to_string())
            .unwrap_or_else(|| format!("TCP {}", port));
        config.rules.push(FirewallRule::new(
            &name,
            Protocol::Tcp,
            PortSpec::Single(*port),
        ));
    }

    // Parse allowedUDPPorts
    let udp_ports = parse_port_list(content, "allowedUDPPorts")?;
    for port in &udp_ports {
        let name = find_saved_name(&metas, "udp", &port.to_string())
            .unwrap_or_else(|| format!("UDP {}", port));
        config.rules.push(FirewallRule::new(
            &name,
            Protocol::Udp,
            PortSpec::Single(*port),
        ));
    }

    // Parse allowedTCPPortRanges
    let tcp_ranges = parse_port_ranges(content, "allowedTCPPortRanges")?;
    for range in &tcp_ranges {
        let ports_str = format!("{}-{}", range.from, range.to);
        let name = find_saved_name(&metas, "tcp", &ports_str)
            .unwrap_or_else(|| format!("TCP {}-{}", range.from, range.to));
        config.rules.push(FirewallRule::new(
            &name,
            Protocol::Tcp,
            PortSpec::Range(range.from, range.to),
        ));
    }

    // Parse allowedUDPPortRanges
    let udp_ranges = parse_port_ranges(content, "allowedUDPPortRanges")?;
    for range in &udp_ranges {
        let ports_str = format!("{}-{}", range.from, range.to);
        let name = find_saved_name(&metas, "udp", &ports_str)
            .unwrap_or_else(|| format!("UDP {}-{}", range.from, range.to));
        config.rules.push(FirewallRule::new(
            &name,
            Protocol::Udp,
            PortSpec::Range(range.from, range.to),
        ));
    }

    // Parse trustedInterfaces
    config.trusted_interfaces = parse_string_list(content, "trustedInterfaces")?;

    // Merge TCP+UDP rule pairs that were originally Protocol::Both
    merge_both_rules(&mut config, &metas);

    // Try to match rules against known presets for better naming
    // (only for rules that didn't get a saved name from nfm comments)
    apply_preset_names(&mut config);

    Ok(config)
}

/// Merge TCP and UDP rules that were originally a Protocol::Both rule,
/// based on nfm:rule comments with proto "both".
fn merge_both_rules(config: &mut FirewallConfig, metas: &[RuleNameMeta]) {
    let both_metas: Vec<&RuleNameMeta> = metas.iter().filter(|m| m.proto == "both").collect();
    if both_metas.is_empty() {
        return;
    }

    let mut merged_rules: Vec<FirewallRule> = Vec::new();
    let mut consumed: Vec<usize> = Vec::new();

    for meta in &both_metas {
        // Find matching TCP and UDP rules for this "both" entry
        let mut tcp_idx = None;
        let mut udp_idx = None;

        for (i, rule) in config.rules.iter().enumerate() {
            if consumed.contains(&i) {
                continue;
            }
            let ports_match = match &rule.ports {
                PortSpec::Single(p) => p.to_string() == meta.ports,
                PortSpec::Range(from, to) => format!("{}-{}", from, to) == meta.ports,
                PortSpec::Multiple(ports) => {
                    ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",") == meta.ports
                }
            };
            if ports_match {
                if rule.protocol == Protocol::Tcp && tcp_idx.is_none() {
                    tcp_idx = Some(i);
                } else if rule.protocol == Protocol::Udp && udp_idx.is_none() {
                    udp_idx = Some(i);
                }
            }
        }

        if let (Some(ti), Some(ui)) = (tcp_idx, udp_idx) {
            consumed.push(ti);
            consumed.push(ui);
            merged_rules.push(FirewallRule::new(
                &meta.name,
                Protocol::Both,
                config.rules[ti].ports.clone(),
            ));
        }
    }

    if !consumed.is_empty() {
        let remaining: Vec<FirewallRule> = config
            .rules
            .iter()
            .enumerate()
            .filter(|(i, _)| !consumed.contains(i))
            .map(|(_, r)| r.clone())
            .collect();
        config.rules = merged_rules;
        config.rules.extend(remaining);
    }
}

/// Parse a list of ports like: allowedTCPPorts = [ 22 80 443 ];
fn parse_port_list(content: &str, key: &str) -> Result<Vec<u16>> {
    let pattern = format!(r"{}\s*=\s*\[([^\]]*)\]", regex::escape(key));
    let re = Regex::new(&pattern)?;

    if let Some(cap) = re.captures(content) {
        let inner = &cap[1];
        let port_re = Regex::new(r"\d+")?;
        let ports: Vec<u16> = port_re
            .find_iter(inner)
            .filter_map(|m| m.as_str().parse::<u16>().ok())
            .collect();
        Ok(ports)
    } else {
        Ok(Vec::new())
    }
}

/// Parse port ranges like: allowedTCPPortRanges = [ { from = 8000; to = 8100; } ];
fn parse_port_ranges(content: &str, key: &str) -> Result<Vec<PortRange>> {
    let pattern = format!(r"(?s){}\s*=\s*\[(.*?)\]\s*;", regex::escape(key));
    let re = Regex::new(&pattern)?;

    if let Some(cap) = re.captures(content) {
        let inner = &cap[1];
        let range_re = Regex::new(r"\{\s*from\s*=\s*(\d+)\s*;\s*to\s*=\s*(\d+)\s*;\s*\}")?;
        let ranges: Vec<PortRange> = range_re
            .captures_iter(inner)
            .filter_map(|c| {
                let from = c[1].parse::<u16>().ok()?;
                let to = c[2].parse::<u16>().ok()?;
                Some(PortRange { from, to })
            })
            .collect();
        Ok(ranges)
    } else {
        Ok(Vec::new())
    }
}

/// Parse a list of strings like: trustedInterfaces = [ "docker0" "virbr0" ];
fn parse_string_list(content: &str, key: &str) -> Result<Vec<String>> {
    let pattern = format!(r"{}\s*=\s*\[([^\]]*)\]", regex::escape(key));
    let re = Regex::new(&pattern)?;

    if let Some(cap) = re.captures(content) {
        let inner = &cap[1];
        let str_re = Regex::new(r#""([^"]+)""#)?;
        let strings: Vec<String> = str_re
            .captures_iter(inner)
            .map(|c| c[1].to_string())
            .collect();
        Ok(strings)
    } else {
        Ok(Vec::new())
    }
}

/// Check if a rule name is a generic auto-generated name (e.g. "TCP 22", "UDP 5000-6000")
fn is_generic_name(name: &str) -> bool {
    let re = Regex::new(r"^(TCP|UDP|TCP\+UDP)\s+\d+").unwrap();
    re.is_match(name)
}

/// Try to match parsed rules against known presets for better naming.
/// For example, TCP 22 becomes "SSH", TCP 80+443 becomes "HTTP / HTTPS", etc.
/// Only renames rules that have generic auto-generated names.
fn apply_preset_names(config: &mut FirewallConfig) {
    use crate::models::presets::get_all_presets;

    let presets = get_all_presets();

    // Try to consolidate individual port rules into known presets
    let mut matched_indices: Vec<usize> = Vec::new();
    let mut new_rules: Vec<FirewallRule> = Vec::new();

    for preset in &presets {
        match &preset.ports {
            PortSpec::Single(port) => {
                // Find matching single-port rule
                for (i, rule) in config.rules.iter().enumerate() {
                    if matched_indices.contains(&i) {
                        continue;
                    }
                    if rule.protocol == preset.protocol {
                        if let PortSpec::Single(rp) = &rule.ports {
                            if rp == port {
                                matched_indices.push(i);
                                // Only use preset name if the current name is generic
                                let name = if is_generic_name(&rule.name) {
                                    &preset.name
                                } else {
                                    &rule.name
                                };
                                new_rules.push(FirewallRule::new(
                                    name,
                                    preset.protocol,
                                    preset.ports.clone(),
                                ));
                                break;
                            }
                        }
                    }
                }
            }
            PortSpec::Multiple(ports) => {
                // Check if all ports of the preset are present
                let mut found_all = true;
                let mut indices_for_preset: Vec<usize> = Vec::new();

                for port in ports {
                    let mut found = false;
                    for (i, rule) in config.rules.iter().enumerate() {
                        if matched_indices.contains(&i) || indices_for_preset.contains(&i) {
                            continue;
                        }
                        if rule.protocol == preset.protocol {
                            if let PortSpec::Single(rp) = &rule.ports {
                                if rp == port {
                                    indices_for_preset.push(i);
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !found {
                        found_all = false;
                        break;
                    }
                }

                if found_all && !indices_for_preset.is_empty() {
                    // Check if any of the matched rules has a custom name
                    let has_custom_name = indices_for_preset.iter().any(|&i| !is_generic_name(&config.rules[i].name));
                    let name = if has_custom_name {
                        // Keep the first custom name found
                        indices_for_preset
                            .iter()
                            .find_map(|&i| {
                                if !is_generic_name(&config.rules[i].name) {
                                    Some(config.rules[i].name.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| preset.name.clone())
                    } else {
                        preset.name.clone()
                    };
                    matched_indices.extend(&indices_for_preset);
                    new_rules.push(FirewallRule::new(
                        &name,
                        preset.protocol,
                        preset.ports.clone(),
                    ));
                }
            }
            PortSpec::Range(from, to) => {
                // Find matching range rule
                for (i, rule) in config.rules.iter().enumerate() {
                    if matched_indices.contains(&i) {
                        continue;
                    }
                    if rule.protocol == preset.protocol {
                        if let PortSpec::Range(rf, rt) = &rule.ports {
                            if rf == from && rt == to {
                                matched_indices.push(i);
                                let name = if is_generic_name(&rule.name) {
                                    &preset.name
                                } else {
                                    &rule.name
                                };
                                new_rules.push(FirewallRule::new(
                                    name,
                                    preset.protocol,
                                    preset.ports.clone(),
                                ));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // Keep unmatched rules and add matched ones with better names
    let remaining: Vec<FirewallRule> = config
        .rules
        .iter()
        .enumerate()
        .filter(|(i, _)| !matched_indices.contains(i))
        .map(|(_, r)| r.clone())
        .collect();

    config.rules = new_rules;
    config.rules.extend(remaining);
}

/// Check if the customConfig/default.nix already imports firewall.nix
pub fn default_nix_has_firewall_import(content: &str) -> bool {
    content.contains("./firewall.nix")
}

/// Inject the firewall.nix import into customConfig/default.nix
/// Also removes any existing networking.firewall.allowed*Ports lines
/// to avoid NixOS option conflicts with firewall.nix
///
/// Handles three cases:
/// 1. firewall.nix already imported -> no-op
/// 2. An `imports = [ ... ];` block already exists -> append ./firewall.nix to it
/// 3. No imports block -> create one after the module body opening `{`
pub fn inject_firewall_import(content: &str) -> Result<String> {
    let mut working = content.to_string();

    // Remove existing firewall port rules that would conflict with firewall.nix
    let firewall_line_re = Regex::new(
        r"(?m)^\s*networking\.firewall\.allowed(?:TCP|UDP)Ports\s*=\s*\[[^\]]*\]\s*;\s*\n?"
    )?;
    working = firewall_line_re.replace_all(&working, "").to_string();

    let firewall_range_re = Regex::new(
        r"(?ms)^\s*networking\.firewall\.allowed(?:TCP|UDP)PortRanges\s*=\s*\[.*?\]\s*;\s*\n?"
    )?;
    working = firewall_range_re.replace_all(&working, "").to_string();

    if default_nix_has_firewall_import(&working) {
        return Ok(working);
    }

    // Case 1: An imports block already exists -> append ./firewall.nix to it
    let imports_re = Regex::new(r"(?s)(imports\s*=\s*\[)(.*?)(]\s*;)")?;
    if imports_re.is_match(&working) {
        let result = imports_re.replace(&working, |caps: &regex::Captures| {
            let opening = &caps[1];
            let existing = &caps[2];
            let closing = &caps[3];
            format!("{}{}\n    ./firewall.nix\n  {}", opening, existing.trim_end(), closing)
        });
        return Ok(result.to_string());
    }

    // Case 2: No imports block -> create one after the module body opening `{`
    //
    // File structure:
    // { lib, config, pkgs, ... }:   <- first {
    // {                              <- second { (module body)
    //   ...
    // }

    let mut result = String::new();
    let mut brace_count = 0;
    let mut injected = false;

    for ch in working.chars() {
        result.push(ch);
        if ch == '{' {
            brace_count += 1;
            if brace_count == 2 && !injected {
                result.push_str("\n  imports = [\n    ./firewall.nix\n  ];\n");
                injected = true;
            }
        }
    }

    if !injected {
        anyhow::bail!("Could not find module body in default.nix to inject firewall import");
    }

    Ok(result)
}
