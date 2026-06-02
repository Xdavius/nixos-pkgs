# Compilation et packaging

## Prerequis

### Dependances systeme

| Paquet | Role |
|--------|------|
| rustc, cargo | Compilateur et gestionnaire de paquets Rust |
| meson, ninja | Systeme de build |
| pkg-config | Detection des bibliotheques systeme |
| gtk4-dev | Toolkit graphique |
| libadwaita-dev | Widgets modernes GNOME |
| glib-dev | Bibliotheque de base GLib |
| librsvg | Rendu SVG pour les icones |
| gettext | Outils d'internationalisation |
| desktop-file-utils | Validation des fichiers .desktop |
| appstream-glib | Validation des metadonnees AppStream |
| gobject-introspection | Introspection GObject |
| polkit | Gestion des privileges |

### Via le flake Nix (recommande)

L'environnement de developpement complet est fourni par le flake :

```bash
cd nix-firewall-mngt
nix develop
```

Cela fournit toutes les dependances necessaires, y compris `RUST_SRC_PATH` pour les IDE.

## Chaine de compilation

Le projet utilise un systeme hybride Meson + Cargo :

```
meson.build (orchestrateur)
├── build-rust.sh → cargo build --release
│   └── produit: target/release/nix-firewall-mngt
├── po/meson.build → compilation gettext (.po → .mo)
└── data/meson.build
    ├── .desktop.in → .desktop (substitution @bindir@)
    ├── .in → .policy (substitution @bindir@)
    └── icons/meson.build → installation des icones
```

### Pourquoi Meson + Cargo ?

- **Cargo** : compile le code Rust avec ses dependances
- **Meson** : gere tout ce qui n'est pas du Rust : traductions, fichiers .desktop, icones, politique PolicyKit, et l'installation finale

### Le script build-rust.sh

Ce script fait le pont entre Meson et Cargo :
1. Recoit `SOURCE_DIR`, `BUILD_DIR` et `OUTPUT` de Meson
2. Execute `cargo build --release` avec le target-dir dans le build Meson
3. Copie le binaire compile vers l'emplacement attendu par Meson

## Compilation manuelle

### Configuration

```bash
meson setup builddir
```

Options courantes :
```bash
meson setup builddir --prefix=/usr
meson setup builddir --prefix=$HOME/.local
```

### Compilation

```bash
meson compile -C builddir
```

Ou directement Cargo (binaire seul, sans data files) :
```bash
cargo build --release
```

### Installation

```bash
meson install -C builddir
```

Installe :
- `/usr/bin/nix-firewall-mngt` - le binaire
- `/usr/share/applications/org.glfos.nixfirewall.desktop` - lanceur
- `/usr/share/polkit-1/actions/org.glfos.nixfirewall.policy` - politique PolicyKit
- `/usr/share/icons/hicolor/*/apps/org.glfos.nixfirewall.{svg,png}` - icones
- `/usr/share/locale/*/LC_MESSAGES/nix-firewall-mngt.mo` - traductions

## Packaging NixOS

### Le fichier default.nix

Defini dans `default.nix`, il utilise `stdenv.mkDerivation` avec le build Meson :

- **nativeBuildInputs** : outils de compilation (meson, ninja, cargo, rustc, pkg-config, etc.)
- **buildInputs** : bibliotheques runtime (gtk4, libadwaita, glib, librsvg, polkit)
- **LOCALE_DIR** : variable d'environnement pointant vers `$out/share/locale`

### Le fichier flake.nix

Le flake expose :
- `packages.default` : le paquet construit via `default.nix`
- `devShells.default` : environnement de developpement avec toutes les dependances

### Construction via flake

```bash
nix build
```

Le resultat est un lien symbolique `result/` pointant vers le paquet dans le store Nix.

## Integration dans GLF-OS

Le paquet est reference dans le depot `glf-os` (Framagit) :

**Fichier** : `glf-os/pkgs/nix-firewall-mngt/default.nix`

```nix
src = fetchFromGitLab {
  domain = "framagit.org";
  owner = "gaming-linux-fr/glf-os/app-glf-os";
  repo = "nix-firewall-mngt";
  rev = "<tag>";
  hash = "<sha256-hash>";
};

cargoDeps = rustPlatform.importCargoLock {
  lockFile = ./Cargo.lock;
};
```

Points importants :
- Le `Cargo.lock` est copie dans le repertoire du paquet glf-os pour `importCargoLock`
- Le `rev` pointe vers un tag specifique sur Framagit
- Le `hash` est le hash SHA256 de l'archive source telechargee

### Mise a jour du paquet dans GLF-OS

1. Pousser les modifications sur Framagit
2. Creer un tag (ex: `v1.0.3`) et le pousser
3. Copier le `Cargo.lock` du projet dans `glf-os/pkgs/nix-firewall-mngt/`
4. Mettre a jour `rev` avec le nouveau tag
4. Mettre a jour `hash` avec `nix shell nixpkgs#nix-prefetch-git -c nix-prefetch-git --url <url> --rev <tag> --quiet`
5. Tester avec `nix flake update glf && sudo nixos-rebuild switch`

## Profil de compilation release

Le `Cargo.toml` configure un profil release optimise :

```toml
[profile.release]
opt-level = 3     # Optimisation maximale
lto = true         # Link-Time Optimization (binaire plus petit et rapide)
codegen-units = 1  # Meilleure optimisation (compilation plus lente)
strip = true       # Suppression des symboles de debug
```

## Dependances Rust

Les principales dependances (definies dans `Cargo.toml`) :

| Crate | Usage |
|-------|-------|
| gtk4 | Bindings GTK4 pour Rust |
| libadwaita | Bindings Libadwaita pour Rust |
| glib | Bindings GLib (types, mainloop) |
| gio | Bindings GIO (I/O, actions) |
| gettext-rs | Integration gettext pour i18n |
| regex | Parsing des fichiers Nix via expressions regulieres |
| anyhow | Gestion d'erreurs simplifiee (type Result generique) |
| thiserror | Derives pour les types d'erreur personnalises |
| once_cell | Initialisation paresseuse de variables statiques |

Toutes les versions sont verrouillees dans `Cargo.lock` pour la reproductibilite.
