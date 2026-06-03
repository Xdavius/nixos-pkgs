{
  lib,
  stdenv,
  rustPlatform,
  wrapGAppsHook4,
  meson,
  ninja,
  pkg-config,
  glib,
  gettext,
  adwaita-icon-theme,
  libadwaita,
  gtk4,
  polkit,
  gobject-introspection,
  cargo,
  rustc,
}:

stdenv.mkDerivation rec {
  pname = "nix-firewall-mngt";
  version = (lib.importTOML ./Cargo.toml).package.version;

  src = lib.cleanSource ./.;

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [
    gettext
    glib
    gobject-introspection
    meson
    ninja
    wrapGAppsHook4
    pkg-config
    rustPlatform.cargoSetupHook
    cargo
    rustc
  ];

  buildInputs = [
    adwaita-icon-theme
    gtk4
    libadwaita
    glib
    polkit
  ];

  # Set environment variables for build
  LOCALE_DIR = "${placeholder "out"}/share/locale";

  meta = with lib; {
    description = "A simple GUI to manage firewall rules on NixOS / GLF-OS";
    license = licenses.gpl3Plus;
    mainProgram = "nix-firewall-mngt";
  };
}
