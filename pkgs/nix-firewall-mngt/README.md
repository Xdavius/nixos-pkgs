# Nix Firewall Manager

Interface graphique pour gerer les regles de pare-feu sur NixOS / GLF-OS.

![License](https://img.shields.io/badge/license-GPL--3.0--or--later-blue)
![Platform](https://img.shields.io/badge/platform-NixOS%20%2F%20GLF--OS-green)
![Rust](https://img.shields.io/badge/rust-2021-orange)

## Fonctionnalites

- Activer / desactiver le pare-feu NixOS
- Ajouter, modifier et supprimer des regles (ports TCP/UDP, plages, ports multiples)
- Persistance des noms de regles personnalises (via commentaires `nfm` dans firewall.nix)
- Presets integres : SSH, HTTP/HTTPS, DNS, Steam, Sunshine/Moonlight, KDE Connect, Syncthing, Samba, etc.
- Gestion des interfaces de confiance (docker0, virbr0, br0...)
- Detection automatique de la configuration flake NixOS pour le rebuild
- Sauvegarde et reconstruction automatique via `nixos-rebuild switch`
- Barre d'action fixe avec les boutons toujours accessibles (sans scroll)
- Theme clair / sombre / systeme
- Interface traduite en 9 langues (fr, en, br, de, es, it, nl, pt, wa)

## Capture d'ecran

*A venir*

## Prerequis

- NixOS ou GLF-OS
- GTK4 et Libadwaita
- Un terminal installe (kgx, gnome-terminal, konsole, etc.)

## Installation

### Sur GLF-OS

L'application est incluse dans la distribution GLF-OS. Elle est disponible dans le menu Applications > Systeme.

### Depuis les sources

```bash
git clone https://framagit.org/gaming-linux-fr/glf-os/app-glf-os/nix-firewall-mngt.git
cd nix-firewall-mngt
nix develop
meson setup builddir --prefix=/usr
meson compile -C builddir
sudo meson install -C builddir
```

### Via le flake Nix

```bash
nix build
```

## Utilisation

1. Lancer l'application depuis le menu ou via `nix-firewall-mngt`
2. Activer le pare-feu avec le switch principal
3. Ajouter des regles manuellement ou via les presets
4. Configurer les interfaces de confiance si necessaire
5. Cliquer sur **Sauvegarder et Reconstruire**
6. Un terminal s'ouvre pour executer `sudo nixos-rebuild switch`
7. Fermer le terminal une fois la reconstruction terminee

## Architecture

```
src/
├── main.rs              # Point d'entree
├── models/              # Structures de donnees (FirewallConfig, FirewallRule, presets)
├── utils/               # Parsing et generation de firewall.nix (avec commentaires nfm)
└── ui/                  # Interface GTK4 / Libadwaita
    ├── app.rs           # Cycle de vie applicatif
    ├── window.rs        # Fenetre principale, ActionBar fixe, logique de rebuild
    ├── widgets/         # Widget liste des regles
    └── dialogs/         # Dialogues ajout de regle et presets
```

L'application genere un fichier `/etc/nixos/customConfig/firewall.nix` et l'injecte dans `default.nix` via un import NixOS standard.

Pour plus de details, voir le repertoire [docs/](docs/) :

- [Architecture](docs/ARCHITECTURE.md) - Conception et architecture detaillees
- [Build](docs/BUILD.md) - Compilation et packaging
- [Deployment](docs/DEPLOYMENT.md) - Deploiement et integration NixOS
- [Development](docs/DEVELOPMENT.md) - Guide du developpeur

## Pile technologique

| Composant | Technologie |
|-----------|-------------|
| Langage | Rust (edition 2021) |
| Toolkit GUI | GTK4 + Libadwaita |
| Systeme de build | Meson + Cargo |
| Internationalisation | gettext |
| Packaging | Nix flake |

## Contribuer

1. Creer une branche : `git checkout -b feature/ma-feature`
2. Developper et tester avec `nix develop`
3. Pousser et creer une Merge Request sur Framagit

Voir [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) pour le guide complet du developpeur.

## Licence

Ce projet est distribue sous licence [GPL-3.0-or-later](https://www.gnu.org/licenses/gpl-3.0.html).

## Auteurs

- [Gaming Linux FR](https://www.gaminglinux.fr/)
