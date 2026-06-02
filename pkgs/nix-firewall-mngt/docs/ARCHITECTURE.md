# Architecture de Nix Firewall Manager

## Vue d'ensemble

Nix Firewall Manager est une application GUI permettant de gerer les regles de pare-feu NixOS/GLF-OS. Elle est ecrite en Rust avec GTK4 et Libadwaita, construite via Meson + Cargo, et empaquetee pour NixOS.

**ID d'application** : `org.glfos.nixfirewall`
**Binaire** : `nix-firewall-mngt`
**Licence** : GPL-3.0-or-later

## Pile technologique

| Composant | Technologie | Version |
|-----------|-------------|---------|
| Langage | Rust | Edition 2021 |
| Toolkit GUI | GTK4 | 0.9 (v4_10) |
| Widgets modernes | Libadwaita | 0.7 (v1_4) |
| Systeme de build | Meson + Cargo | - |
| Internationalisation | gettext-rs | 0.7 |
| Parsing | regex | 1.10 |
| Gestion d'erreurs | anyhow + thiserror | 1.0 |

## Structure du projet

```
nix-firewall-mngt/
├── src/
│   ├── main.rs                     # Point d'entree
│   ├── models/
│   │   ├── mod.rs                  # Exports du module
│   │   ├── firewall_config.rs      # Structures de donnees (FirewallConfig, FirewallRule, etc.)
│   │   └── presets.rs              # Presets de regles predefinies
│   ├── utils/
│   │   ├── mod.rs                  # Exports du module
│   │   ├── nix_parser.rs           # Lecture et parsing de firewall.nix
│   │   └── nix_writer.rs           # Generation du contenu firewall.nix
│   └── ui/
│       ├── mod.rs                  # Exports du module
│       ├── app.rs                  # Controlleur applicatif (lifecycle GTK)
│       ├── window.rs               # Fenetre principale et logique de sauvegarde/rebuild
│       ├── widgets/
│       │   ├── mod.rs
│       │   └── rules_list.rs       # Widget liste des regles
│       └── dialogs/
│           ├── mod.rs
│           ├── add_rule.rs         # Dialogue ajout/edition de regle
│           └── presets.rs          # Dialogue selection de presets
├── data/
│   ├── meson.build                 # Installation des fichiers data
│   ├── org.glfos.nixfirewall.desktop.in  # Fichier .desktop (template)
│   ├── org.glfos.nixfirewall.in          # Politique PolicyKit (template)
│   └── icons/
│       ├── meson.build             # Installation des icones
│       ├── nix-firewall-mngt.svg   # Icone SVG (scalable)
│       └── nix-firewall-mngt-*.png # Icones PNG (16 a 512px)
├── po/
│   ├── meson.build                 # Configuration i18n
│   ├── POTFILES                    # Liste des fichiers traduisibles
│   ├── LINGUAS                     # Langues supportees
│   ├── en.po / fr.po / br.po      # Traductions (en, fr, br, de, es, it, nl, pt, wa)
│   └── ...
├── Cargo.toml                      # Manifest Rust
├── Cargo.lock                      # Versions verrouillees
├── meson.build                     # Build principal Meson
├── build-rust.sh                   # Script de compilation Cargo
├── flake.nix                       # Environnement de dev Nix
└── default.nix                     # Definition du paquet NixOS
```

## Architecture en couches

```
┌─────────────────────────────────────────────────┐
│  Couche UI (GTK4 + Libadwaita)                  │
│  ├── NixFirewallWindow (window.rs)              │
│  │   ├── Banners (rebuild en cours / erreur)    │
│  │   ├── Header Bar (theme)                     │
│  │   ├── ScrolledWindow                         │
│  │   │   ├── Switch firewall enable/disable     │
│  │   │   ├── Trusted Interfaces card            │
│  │   │   └── RulesListWidget                    │
│  │   └── ActionBar fixe (bas de fenetre)        │
│  │       ├── Boutons Ajouter / Presets          │
│  │       └── Bouton Sauvegarder et Reconstruire │
│  ├── RulesListWidget (rules_list.rs)            │
│  ├── AddRuleDialog (add_rule.rs)                │
│  └── PresetsDialog (presets.rs)                 │
└──────────────────┬──────────────────────────────┘
                   │ Rc<RefCell<FirewallConfig>>
┌──────────────────▼────────────────────────────────┐
│  Couche Modeles                                   │
│  ├── FirewallConfig { enabled, rules, interfaces} │
│  ├── FirewallRule { name, protocol, ports, on }   │
│  ├── Protocol (Tcp | Udp | Both)                  │
│  ├── PortSpec (Single | Range | Multiple)         │
│  └── PresetCategory { name, rules }               │
└──────────────────┬────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│  Couche Utilitaires                             │
│  ├── parse_firewall_nix()  : String → Config    │
│  │   ├── parse_nfm_comments() : noms sauvegardes│
│  │   └── merge_both_rules() : fusion TCP+UDP    │
│  ├── generate_firewall_nix() : Config → String  │
│  │   └── Ecrit commentaires # nfm:rule:...      │
│  ├── inject_firewall_import() : default.nix     │
│  └── default_nix_has_firewall_import()          │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│  Integration systeme NixOS                      │
│  ├── /etc/nixos/customConfig/firewall.nix       │
│  ├── /etc/nixos/customConfig/default.nix        │
│  ├── /etc/nixos/flake.nix (detection flake)     │
│  ├── nixos-rebuild switch [--flake] (terminal)  │
│  └── PolicyKit (elevation de privileges)        │
└─────────────────────────────────────────────────┘
```

## Diagrammes de flux

### Initialisation de l'application

Au demarrage, l'application charge la configuration existante ou cree une configuration vierge.

```mermaid
flowchart TD
    start([Lancement nix-firewall-mngt])
    init_gettext[Initialisation gettext / i18n]
    create_app["Création NixFirewallApp<br/>ID: org.glfos.nixfirewall"]
    load_theme["Lecture ~/.config/nix-firewall-mngt/theme.conf"]
    apply_theme["Application du thème<br/>(System / Light / Dark)"]
    check_fw{"/etc/nixos/customConfig/<br/>firewall.nix existe ?"}
    read_fw["Lecture de firewall.nix"]
    parse_fw["parse_firewall_nix()<br/>1. Lecture commentaires nfm (noms sauvegardes)<br/>2. Extraction regles, ports, interfaces<br/>3. Fusion regles Both (TCP+UDP)"]
    apply_presets["apply_preset_names()<br/>Reconnaissance des presets connus<br/>(preserve les noms personnalises)"]
    default_config["Création FirewallConfig::default()<br/>enabled=true, rules=[], interfaces=[]"]
    parse_error["Erreur de parsing<br/>→ Config par defaut"]
    check_default{"/etc/nixos/customConfig/<br/>default.nix existe ?"}
    check_import{"default.nix contient<br/>./firewall.nix ?"}
    needs_import["needs_import = true<br/>(injection requise au premier save)"]
    no_import["needs_import = false"]
    create_window["Création NixFirewallWindow<br/>avec Rc&lt;RefCell&lt;FirewallConfig&gt;&gt;"]
    show_ui["Affichage de la fenetre<br/>Switch, regles, interfaces, boutons"]

    start --> init_gettext --> create_app --> load_theme --> apply_theme
    apply_theme --> check_fw
    check_fw -->|Oui| read_fw --> parse_fw
    parse_fw -->|Succes| apply_presets --> check_default
    parse_fw -->|Erreur| parse_error --> check_default
    check_fw -->|Non| default_config --> check_default
    check_default -->|Oui| check_import
    check_default -->|Non| no_import
    check_import -->|Non| needs_import
    check_import -->|Oui| no_import
    needs_import --> create_window
    no_import --> create_window
    create_window --> show_ui
```

### Ajout d'une regle manuelle

L'utilisateur peut ajouter une regle via le dialogue dedie.

```mermaid
flowchart TD
    click_add["Clic sur 'Ajouter une regle'"]
    open_dialog["Ouverture AddRuleDialog"]
    input_name["Saisie du nom de la regle"]
    select_proto["Selection du protocole<br/>TCP / UDP / TCP+UDP"]
    select_mode{"Type de ports ?"}
    single["Saisie port unique<br/>(ex: 22)"]
    range["Saisie plage<br/>De: ... A: ..."]
    multiple["Saisie ports multiples<br/>(ex: 80, 443)"]
    validate{"Validation :<br/>- Nom non vide<br/>- Ports 1-65535<br/>- Pas de doublon"}
    error_toast["Toast d'erreur"]
    create_rule["Création FirewallRule<br/>{name, protocol, ports, enabled=true}"]
    push_config["config.borrow_mut().rules.push(rule)"]
    refresh_list["RulesListWidget::refresh()<br/>Mise a jour de l'affichage"]

    click_add --> open_dialog --> input_name --> select_proto --> select_mode
    select_mode -->|Port unique| single --> validate
    select_mode -->|Plage| range --> validate
    select_mode -->|Multiples| multiple --> validate
    validate -->|Invalide| error_toast --> input_name
    validate -->|Valide| create_rule --> push_config --> refresh_list
```

### Ajout via les presets

L'utilisateur peut importer des regles predefinies par categorie.

```mermaid
flowchart TD
    click_presets["Clic sur 'Presets'"]
    open_presets["Ouverture PresetsDialog"]
    load_cats["get_preset_categories()<br/>Services courants, Gaming, Connectivite"]
    display["Affichage par categorie<br/>avec cases a cocher"]
    select["Selection des presets souhaites"]
    click_ok["Clic sur Ajouter"]
    loop_start{"Encore des<br/>presets selectionnes ?"}
    get_next["Preset suivant"]
    check_dup{"Regle deja<br/>presente ?"}
    skip["Ignore (pas de doublon)"]
    add_rule["config.borrow_mut().rules.push(preset)"]
    loop_end["Preset suivant"]
    refresh["RulesListWidget::refresh()"]

    click_presets --> open_presets --> load_cats --> display --> select --> click_ok
    click_ok --> loop_start
    loop_start -->|Oui| get_next --> check_dup
    check_dup -->|Oui| skip --> loop_end --> loop_start
    check_dup -->|Non| add_rule --> loop_end
    loop_start -->|Non| refresh
```

### Edition et suppression d'une regle

```mermaid
flowchart TD
    subgraph Edition
        click_edit["Clic sur le bouton Edit<br/>d'une regle existante"]
        open_edit["AddRuleDialog::new_edit()<br/>Pre-rempli avec les valeurs actuelles"]
        modify["Modification des champs"]
        save_edit["config.rules[index] = updated_rule"]
        refresh_edit["RulesListWidget::refresh()"]

        click_edit --> open_edit --> modify --> save_edit --> refresh_edit
    end

    subgraph Suppression
        click_del["Clic sur le bouton Supprimer"]
        confirm["Confirmation de suppression"]
        remove["config.borrow_mut().rules.remove(index)"]
        refresh_del["RulesListWidget::refresh()"]

        click_del --> confirm --> remove --> refresh_del
    end

    subgraph Toggle
        click_check["Clic sur la checkbox<br/>d'une regle"]
        toggle["rule.enabled = !rule.enabled"]
        note["Les regles desactivees sont<br/>exclues de firewall.nix"]

        click_check --> toggle --> note
    end
```

### Sauvegarde et reconstruction NixOS

Le flux complet de `do_save_and_rebuild()`, de la generation du fichier a la reconstruction systeme.

```mermaid
flowchart TD
    click_save["Clic sur 'Sauvegarder et Reconstruire'"]
    gen_nix["generate_firewall_nix(config)<br/>1. Ecrit commentaires # nfm:rule:...<br/>2. Genere le contenu Nix"]
    write_tmp["Ecriture /tmp/nix_firewall_TIMESTAMP.nix"]
    write_err{Ecriture OK ?}
    banner_err1["Banniere d'erreur"]

    check_inject{"needs_import<br/>== true ?"}
    read_default["Lecture de default.nix"]
    inject["inject_firewall_import()<br/>1. Supprime networking.firewall.allowed*<br/>2. Si imports existe : append ./firewall.nix<br/>3. Sinon : cree imports = ./firewall.nix"]
    write_default_tmp["Ecriture /tmp/nix_firewall_default_TIMESTAMP.nix"]
    inject_err{"Injection OK ?"}
    banner_err2["Banniere d'erreur<br/>+ nettoyage /tmp"]

    gen_script["Creation du script bash<br/>/tmp/nix_firewall_rebuild_TIMESTAMP.sh"]
    script_content["Contenu du script :<br/>1. sudo cp firewall.nix → /etc/nixos/customConfig/<br/>2. sudo cp default.nix (si injection)<br/>3. Detection flake : grep nixosConfigurations<br/>4. sudo nixos-rebuild switch [--flake]<br/>5. touch .done si succes<br/>6. Nettoyage des fichiers /tmp"]
    chmod["chmod +x sur le script"]

    try_term{"Recherche d'un terminal"}
    term_list["Ordre de tentative :<br/>kgx → gnome-terminal →<br/>konsole → xfce4-terminal →<br/>alacritty → kitty → xterm"]
    term_found{Terminal trouve ?}
    no_term["Banniere d'erreur :<br/>Aucun terminal disponible"]

    show_banner["Banniere : Reconstruction en cours..."]
    term_exec["Le terminal execute le script :<br/>┌────────────────────────────────┐<br/>│ sudo cp des fichiers config    │<br/>│ sudo nixos-rebuild switch      │<br/>│ → succes : touch .done         │<br/>│ → echec : message d'erreur     │<br/>│ Nettoyage fichiers temporaires │<br/>│ 'Vous pouvez fermer...'        │<br/>└────────────────────────────────┘"]

    poll_start["Polling toutes les 2 secondes"]
    check_done{".done existe ?"}
    check_timeout{"300 checks<br/>(10 min) ?"}
    poll_wait["Attente 2s"]
    success["Banniere masquee<br/>Toast : Configuration appliquee"]
    timeout["Banniere masquee<br/>Timeout silencieux"]
    cleanup["Nettoyage :<br/>rm .done, rm script.sh"]

    click_save --> gen_nix --> write_tmp --> write_err
    write_err -->|Non| banner_err1
    write_err -->|Oui| check_inject

    check_inject -->|Oui| read_default --> inject --> write_default_tmp --> inject_err
    inject_err -->|Non| banner_err2
    inject_err -->|Oui| gen_script
    check_inject -->|Non| gen_script

    gen_script --> script_content --> chmod --> try_term
    try_term --> term_list --> term_found
    term_found -->|Non| no_term
    term_found -->|Oui| show_banner --> term_exec

    term_exec --> poll_start --> check_done
    check_done -->|Oui| success --> cleanup
    check_done -->|Non| check_timeout
    check_timeout -->|Oui| timeout
    check_timeout -->|Non| poll_wait --> check_done
```

### Interactions avec les fichiers systeme

Vue d'ensemble des fichiers lus et ecrits par l'application.

```mermaid
flowchart LR
    subgraph Application
        app["nix-firewall-mngt"]
    end

    subgraph "Fichiers utilisateur (~)"
        theme["~/.config/nix-firewall-mngt/<br/>theme.conf"]
    end

    subgraph "Fichiers temporaires (/tmp)"
        tmp_fw["/tmp/nix_firewall_TS.nix"]
        tmp_def["/tmp/nix_firewall_default_TS.nix"]
        tmp_sh["/tmp/nix_firewall_rebuild_TS.sh"]
        tmp_done["/tmp/nix_firewall_rebuild_TS.done"]
    end

    subgraph "Terminal (kgx)"
        terminal["Execution du script<br/>avec sudo"]
    end

    subgraph "Configuration NixOS (/etc/nixos)"
        fw_nix["/etc/nixos/customConfig/<br/>firewall.nix"]
        def_nix["/etc/nixos/customConfig/<br/>default.nix"]
    end

    subgraph "Systeme NixOS"
        rebuild["nixos-rebuild switch"]
        iptables["Regles iptables/nftables<br/>appliquees par NixOS"]
    end

    app -->|"R/W"| theme
    app -->|"Lecture au demarrage"| fw_nix
    app -->|"Lecture au demarrage"| def_nix
    app -->|"Ecriture"| tmp_fw
    app -->|"Ecriture (si injection)"| tmp_def
    app -->|"Ecriture"| tmp_sh
    app -->|"Polling"| tmp_done
    app -->|"Ouvre"| terminal
    terminal -->|"Execute"| tmp_sh
    tmp_sh -->|"sudo cp"| fw_nix
    tmp_sh -->|"sudo cp"| def_nix
    tmp_sh -->|"sudo"| rebuild
    tmp_sh -->|"touch si succes"| tmp_done
    rebuild -->|"Applique"| iptables
```

### Cycle de vie complet : du premier lancement a l'exploitation

```mermaid
flowchart TD
    subgraph "Premier lancement"
        first_start(["1er lancement"])
        no_fw["Pas de firewall.nix<br/>→ Config vierge"]
        check_def["Lecture default.nix<br/>→ needs_import = true"]
        user_adds["L'utilisateur ajoute<br/>des regles / presets"]
        first_save["Clic Sauvegarder"]
        gen_fw["Generation firewall.nix"]
        gen_def["Injection import dans default.nix"]
        first_rebuild["nixos-rebuild switch<br/>→ Firewall actif"]

        first_start --> no_fw --> check_def --> user_adds --> first_save
        first_save --> gen_fw --> gen_def --> first_rebuild
    end

    subgraph "Lancements suivants"
        next_start(["Lancement suivant"])
        load_fw["Lecture firewall.nix existant<br/>→ Parse des regles<br/>→ Restauration noms via commentaires nfm"]
        preset_match["Reconnaissance des presets<br/>(port 22 → SSH, etc.)<br/>Preserve les noms personnalises"]
        import_ok["default.nix a deja l'import<br/>→ needs_import = false"]
        display["Affichage des regles<br/>existantes dans l'UI"]
        user_modifies["L'utilisateur modifie :<br/>- Ajouter / supprimer des regles<br/>- Changer les interfaces<br/>- Activer / desactiver"]
        next_save["Clic Sauvegarder"]
        regen_fw["Re-generation firewall.nix<br/>(ecrase le precedent)"]
        no_inject["Pas d'injection<br/>(import deja present)"]
        next_rebuild["nixos-rebuild switch<br/>→ Nouvelles regles appliquees"]

        next_start --> load_fw --> preset_match --> import_ok --> display
        display --> user_modifies --> next_save
        next_save --> regen_fw --> no_inject --> next_rebuild
    end

    first_rebuild -.->|"Prochain demarrage"| next_start
    next_rebuild -.->|"Prochain demarrage"| next_start
```

## Modele de donnees

### FirewallConfig
Structure centrale representant l'etat complet du pare-feu :
- `enabled: bool` - pare-feu actif ou non
- `rules: Vec<FirewallRule>` - liste des regles
- `trusted_interfaces: Vec<String>` - interfaces de confiance (ex: docker0)

### FirewallRule
Une regle de pare-feu nommee :
- `name: String` - nom affiche (ex: "SSH", "Steam")
- `protocol: Protocol` - TCP, UDP ou Both
- `ports: PortSpec` - specification des ports
- `enabled: bool` - regle active ou non

### PortSpec
Trois variantes pour specifier les ports :
- `Single(u16)` - un port unique (ex: 22)
- `Range(u16, u16)` - une plage (ex: 27015-27030)
- `Multiple(Vec<u16>)` - plusieurs ports (ex: 80, 443)

### Protocol
- `Tcp` - protocole TCP uniquement
- `Udp` - protocole UDP uniquement
- `Both` - les deux protocoles (genere des regles TCP et UDP)

## Flux de sauvegarde et reconstruction

Le flux de sauvegarde (`do_save_and_rebuild()` dans `window.rs`) est le coeur de l'application :

```
1. Generation du contenu firewall.nix
   Config → generate_firewall_nix() → String Nix
   → Ecrit les commentaires # nfm:rule:<nom>:<proto>:<ports> pour chaque regle
   → Genere les blocs allowedTCPPorts, allowedUDPPorts, etc.

2. Ecriture en fichier temporaire
   → /tmp/nix_firewall_<timestamp>.nix

3. Si besoin, injection de l'import dans default.nix
   → inject_firewall_import() retire les regles firewall existantes
   → si un bloc `imports` existe deja, ajoute `./firewall.nix` a la liste
   → sinon, cree `imports = [ ./firewall.nix ];` apres le 2e `{`
   → ecrit dans /tmp/nix_firewall_default_<timestamp>.nix

4. Creation du script de rebuild
   → /tmp/nix_firewall_rebuild_<timestamp>.sh
   → Contient : sudo cp des fichiers
   → Detection flake : grep nixosConfigurations dans /etc/nixos/flake.nix
   → Si flake detecte : sudo nixos-rebuild switch --flake "/etc/nixos#<attr>"
   → Sinon : sudo nixos-rebuild switch

5. Ouverture d'un terminal
   → Essaie dans l'ordre : kgx, gnome-terminal, konsole, xfce4-terminal, alacritty, kitty, xterm
   → Le terminal execute le script de rebuild

6. Surveillance de la completion
   → Polling toutes les 2s du fichier /tmp/nix_firewall_rebuild_<timestamp>.done
   → Cree par le script si rebuild reussi (touch)
   → Timeout apres 10 minutes (300 checks)
   → Toast de succes a la detection du fichier
```

### Pourquoi des fichiers temporaires ?

L'ecriture dans `/etc/nixos/` necessite les droits root. L'application ecrit d'abord dans `/tmp/` (accessible sans droits), puis le script utilise `sudo cp` pour copier vers la destination finale. Cela permet de dissocier la logique applicative de l'elevation de privileges.

## Systeme de presets

Les presets sont definis dans `models/presets.rs` et organises par categories :

| Categorie | Preset | Protocole | Ports |
|-----------|--------|-----------|-------|
| Services courants | SSH | TCP | 22 |
| Services courants | HTTP / HTTPS | TCP | 80, 443 |
| Services courants | DNS | UDP | 53 |
| Gaming | Steam | TCP+UDP | 27015-27030 |
| Gaming | Sunshine / Moonlight | TCP+UDP | 47984-47990 |
| Gaming | Minecraft Serveur | TCP | 25565 |
| Connectivite | KDE Connect | TCP+UDP | 1714-1764 |
| Connectivite | Syncthing | TCP | 22000, 21027 |
| Connectivite | Samba (TCP) | TCP | 139, 445 |
| Connectivite | Samba (UDP) | UDP | 137, 138 |

Le parser (`apply_preset_names()`) tente de reconnaitre les regles parsees correspondant a des presets connus pour leur donner un nom lisible. Les noms personnalises (restaures depuis les commentaires nfm) ne sont pas ecrases par les presets.

## Persistance des noms de regles

Les noms de regles personnalises sont sauvegardes en commentaires dans `firewall.nix` :

```nix
# Fichier genere par Nix Firewall Manager - Ne pas editer manuellement
# Generated by Nix Firewall Manager - Do not edit manually
# nfm:rule:SSH:tcp:22
# nfm:rule:Mon serveur web:tcp:80,443
# nfm:rule:Steam:both:27015-27030
{ lib, config, pkgs, ... }:
...
```

### Format des commentaires nfm

```
# nfm:rule:<nom>:<protocole>:<ports>
```

- `<nom>` : nom de la regle tel qu'affiche dans l'UI
- `<protocole>` : `tcp`, `udp` ou `both`
- `<ports>` : port unique (`22`), plage (`27015-27030`) ou multiples (`80,443`)

### Processus de restauration au parsing

1. `parse_nfm_comments()` : extrait les metadonnees des commentaires
2. `find_saved_name()` : associe chaque port/plage parse a son nom sauvegarde
3. `merge_both_rules()` : fusionne les paires TCP/UDP en regles `Protocol::Both` quand un commentaire `both` est present
4. `apply_preset_names()` : ne renomme que les regles avec des noms generiques (ex: "TCP 22"), preserve les noms personnalises

### Verification des noms generiques

La fonction `is_generic_name()` detecte les noms auto-generes (format `<PROTO> <port>`) pour eviter d'ecraser un nom personnalise avec un nom de preset.

## Detection flake pour le rebuild

Le script de rebuild detecte automatiquement si le systeme utilise un flake NixOS :

1. Verifie l'existence de `/etc/nixos/flake.nix`
2. Extrait le nom de la `nixosConfiguration` via `grep -oP`
3. Si trouve : utilise `nixos-rebuild switch --flake "/etc/nixos#<nom>"`
4. Sinon : utilise `nixos-rebuild switch` classique

Cela resout le probleme ou le hostname de la machine ne correspond pas au nom declare dans `nixosConfigurations` du flake.

## Integration GNOME

Pour que l'icone apparaisse dans la barre des taches GNOME, trois elements doivent correspondre :

1. **Application ID** (`org.glfos.nixfirewall` dans `app.rs`)
2. **Fichier .desktop** (`org.glfos.nixfirewall.desktop`)
3. **Icones** installees sous le nom `org.glfos.nixfirewall.{svg,png}`

GNOME Shell associe les fenetres aux fichiers .desktop via l'application ID. Si le nom du .desktop ne correspond pas, l'icone generique est utilisee.

## Gestion du theme

L'application supporte 3 modes de theme :
- **System** : suit la preference du systeme (defaut)
- **Light** : force le theme clair
- **Dark** : force le theme sombre

La preference est sauvegardee dans `~/.config/nix-firewall-mngt/theme.conf` (fichier texte contenant "system", "light" ou "dark"). Le theme est applique au demarrage via `adw::StyleManager`.

## Internationalisation (i18n)

L'application utilise `gettext-rs` pour la traduction :
- Les chaines traduisibles sont marquees avec `gettext()` dans le code
- Les fichiers `.po` contiennent les traductions
- Meson compile les `.po` en `.mo` (format binaire) a l'installation
- `LOCALE_DIR` est passe a la compilation pour localiser les fichiers `.mo`

Langues supportees : francais (fr), anglais (en), breton (br), allemand (de), espagnol (es), italien (it), neerlandais (nl), portugais (pt), wallon (wa)

## PolicyKit

La politique PolicyKit (`org.glfos.nixfirewall.policy`) definit les regles d'elevation de privileges :
- **Utilisateur actif** : `auth_admin_keep` - demande le mot de passe admin, garde la session
- **Utilisateur inactif** : `auth_admin` - demande le mot de passe admin
- **Tout utilisateur** : `auth_admin` - demande le mot de passe admin

Cela permet l'execution de `sudo nixos-rebuild switch` dans le terminal ouvert par l'application.
