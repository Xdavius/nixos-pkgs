# Guide du developpeur

## Mise en route

### 1. Cloner le depot

```bash
git clone https://framagit.org/gaming-linux-fr/glf-os/app-glf-os/nix-firewall-mngt.git
cd nix-firewall-mngt
```

### 2. Entrer dans l'environnement de developpement

```bash
nix develop
```

Cela fournit toutes les dependances (Rust, GTK4, Libadwaita, Meson, etc.) sans polluer le systeme.

### 3. Compiler et lancer

```bash
# Via Meson (installation complete avec data files)
meson setup builddir --prefix=$HOME/.local
meson compile -C builddir
meson install -C builddir

# Ou directement via Cargo (binaire seul)
cargo build --release
./target/release/nix-firewall-mngt
```

## Organisation du code

### Modules principaux

**`src/models/`** - Structures de donnees pures, sans logique UI :
- `firewall_config.rs` : types `FirewallConfig`, `FirewallRule`, `Protocol`, `PortSpec`, `PortRange`
- `presets.rs` : presets de regles predefinies par categorie

**`src/utils/`** - Logique metier sans dependance UI :
- `nix_parser.rs` : parsing du format Nix (firewall.nix → FirewallConfig)
- `nix_writer.rs` : generation du format Nix (FirewallConfig → String)

**`src/ui/`** - Interface graphique GTK4/Libadwaita :
- `app.rs` : cycle de vie de l'application, chargement initial
- `window.rs` : fenetre principale, logique de sauvegarde/rebuild
- `widgets/rules_list.rs` : widget personnalise pour la liste des regles
- `dialogs/add_rule.rs` : dialogue d'ajout/edition de regle
- `dialogs/presets.rs` : dialogue de selection de presets

### Pattern de partage d'etat

L'etat mutable (`FirewallConfig`) est partage entre les composants UI via `Rc<RefCell<>>` :

```rust
let config = Rc::new(RefCell::new(FirewallConfig::default()));

// Lecture
let cfg = config.borrow();
println!("{} rules", cfg.rules.len());

// Modification
config.borrow_mut().rules.push(new_rule);
```

Chaque composant UI recoit un clone du `Rc`, ce qui permet a plusieurs callbacks de lire et modifier la meme configuration.

## Ajouter un nouveau preset

Editer `src/models/presets.rs` :

```rust
PresetCategory {
    name: "Ma categorie".to_string(),
    rules: vec![
        FirewallRule::new("Mon service", Protocol::Tcp, PortSpec::Single(8080)),
        FirewallRule::new("Mon autre service", Protocol::Both, PortSpec::Range(9000, 9100)),
    ],
},
```

Les presets apparaitront automatiquement dans le dialogue de presets et seront reconnus par le parser (`apply_preset_names()`). Les noms personnalises sauvegardes via les commentaires nfm ne seront pas ecrases par les presets.

## Ajouter une traduction

### 1. Marquer les chaines dans le code

```rust
use gettextrs::gettext;

let label = gettext("My translatable string");
```

### 2. Ajouter le fichier source dans po/POTFILES

```
src/ui/mon_nouveau_fichier.rs
```

### 3. Mettre a jour les fichiers .po

```bash
cd builddir
meson compile nix-firewall-mngt-update-po
```

### 4. Traduire

Editer `po/fr.po` et remplir les `msgstr` manquants.

### 5. Ajouter une nouvelle langue

Ajouter le code langue dans `po/LINGUAS`, puis creer le fichier `.po` correspondant. Langues actuelles : en, fr, br, de, es, it, nl, pt, wa.

## Modifier le parsing Nix

Le parser dans `nix_parser.rs` utilise des expressions regulieres pour extraire les valeurs du format Nix. Les patterns principaux :

| Pattern | Ce qu'il parse |
|---------|---------------|
| `# nfm:rule:([^:]+):([^:]+):(.+)` | Metadonnees de nom de regle |
| `enable\s*=\s*(true\|false)\s*;` | Etat du firewall |
| `allowedTCPPorts\s*=\s*\[([^\]]*)\]` | Liste de ports TCP |
| `allowedTCPPortRanges\s*=\s*\[(.*?)\]\s*;` | Plages de ports TCP |
| `\{\s*from\s*=\s*(\d+)\s*;\s*to\s*=\s*(\d+)\s*;\s*\}` | Une plage individuelle |
| `trustedInterfaces\s*=\s*\[([^\]]*)\]` | Interfaces de confiance |

Le parser suit ce pipeline :
1. `parse_nfm_comments()` : extrait les noms sauvegardes des commentaires
2. Parsing des ports TCP/UDP avec association aux noms sauvegardes
3. `merge_both_rules()` : fusionne les paires TCP+UDP en `Protocol::Both`
4. `apply_preset_names()` : renomme les regles generiques avec les noms de presets (preserve les noms personnalises)

Pour ajouter une nouvelle option NixOS au parser, suivre le meme pattern :
1. Definir la regex correspondante
2. Extraire les captures
3. Mapper vers les structures de donnees

## Modifier la generation Nix

Le generateur dans `nix_writer.rs` est une construction de chaine simple. Il ecrit d'abord les commentaires de metadonnees `nfm` pour la persistance des noms, puis les blocs Nix :

```rust
// Commentaires nfm pour la persistance des noms
for rule in &config.rules {
    out.push_str(&format!("# nfm:rule:{}:{}:{}\n", rule.name, proto, ports_str));
}

// allowedTCPPorts
let tcp_ports = config.all_tcp_ports();
out.push_str(&format!(
    "    allowedTCPPorts = [ {} ];\n",
    tcp_ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" ")
));
```

Pour ajouter une nouvelle option :
1. Ajouter le champ dans `FirewallConfig`
2. Ajouter le parsing dans `nix_parser.rs`
3. Ajouter la generation dans `nix_writer.rs`
4. Ajouter l'UI correspondante dans `window.rs`

## Modifier l'UI

### Ajouter un widget

1. Creer le fichier dans `src/ui/widgets/` ou `src/ui/dialogs/`
2. Exporter dans le `mod.rs` correspondant
3. Instancier dans `window.rs`

### Conventions UI

- Utiliser les widgets Libadwaita (`adw::`) quand disponibles
- Les cartes utilisent la classe CSS `card`
- Les boutons d'action utilisent `pill` et `suggested-action`
- Les labels secondaires utilisent `dim-label` et `caption`
- Les chaines visibles a l'utilisateur doivent etre dans `gettext()`
- Les boutons d'action principaux sont dans l'`ActionBar` fixe en bas de fenetre
- Le contenu scrollable est dans un `ScrolledWindow`, l'`ActionBar` est en dehors

## Tests

Le projet n'a actuellement pas de tests automatises. Les zones testables en priorite :

1. **nix_parser.rs** : parsing de differents formats de firewall.nix
2. **nix_writer.rs** : generation et coherence aller-retour (parse → generate → parse)
3. **inject_firewall_import()** : injection dans differentes structures de default.nix (avec ou sans bloc imports existant)
4. **apply_preset_names()** : reconnaissance correcte des presets

Exemple de test a ajouter :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let content = r#"
# nfm:rule:SSH:tcp:22
# nfm:rule:Mon serveur web:tcp:80,443
{ lib, config, pkgs, ... }:
{
  networking.firewall = {
    enable = true;
    allowedTCPPorts = [ 22 80 443 ];
    allowedUDPPorts = [];
    allowedTCPPortRanges = [];
    allowedUDPPortRanges = [];
    trustedInterfaces = [];
  };
}
"#;
        let config = parse_firewall_nix(content).unwrap();
        assert!(config.enabled);
        assert_eq!(config.all_tcp_ports(), vec![22, 80, 443]);
        // Les noms sauvegardes sont restaures
        assert_eq!(config.rules[0].name, "SSH");
    }
}
```

## Depots et branches

| Depot | Hebergement | Role |
|-------|-------------|------|
| `nix-firewall-mngt` | Framagit (gaming-linux-fr/glf-os/app-glf-os) | Code source de l'application |
| `glf-os` | Framagit (gaming-linux-fr) | Distribution GLF-OS (contient le paquet Nix) |
| `glf-wiki` | Framagit (gaming-linux-fr) | Wiki de la documentation utilisateur |

### Workflow de contribution

1. Creer une branche feature : `git checkout -b feature/ma-feature`
2. Developper et tester localement
3. Pousser sur Framagit
4. Mettre a jour le paquet dans `glf-os` (rev + hash + Cargo.lock)
5. Tester sur une VM GLF-OS avec `nixos-rebuild switch`

## Variables d'environnement

| Variable | Role | Defaut |
|----------|------|--------|
| `LOCALE_DIR` | Chemin vers les fichiers de traduction .mo | `/usr/share/locale` |
| `RUST_SRC_PATH` | Chemin vers les sources Rust (pour IDE) | Set par `nix develop` |
| `CARGO_PKG_VERSION` | Version du paquet | Depuis Cargo.toml |

## Decisions de conception notables

### Fichiers temporaires pour l'elevation de privileges

L'application ecrit dans `/tmp/` puis utilise `sudo cp` plutot que d'ecrire directement dans `/etc/nixos/`. Cela evite de lancer l'application entiere en root.

### Regex plutot que parser Nix complet

Le parsing de `firewall.nix` utilise des regex plutot qu'un parser Nix complet. C'est suffisant car le fichier est genere par l'application elle-meme et a un format previsible.

### Reconnaissance des presets au parsing

Quand l'application relit un `firewall.nix`, elle tente de reconnaitre les ports correspondant a des presets connus (ex: port 22 → "SSH"). Cela donne des noms lisibles aux regles meme si le fichier a ete edite manuellement.

### kgx comme terminal prioritaire

kgx (GNOME Console) est le terminal par defaut sur GLF-OS. Les autres terminaux sont testes en fallback pour la compatibilite avec d'autres environnements de bureau.
