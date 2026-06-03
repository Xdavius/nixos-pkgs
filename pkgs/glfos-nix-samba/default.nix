{
  lib,
  fetchgit,
  wrapGAppsHook4,
  meson,
  ninja,
  desktop-file-utils,
  gettext,
  gobject-introspection,
  python3Packages,
  adwaita-icon-theme,
  libadwaita,
  gtk4,
}:

python3Packages.buildPythonApplication rec {
  pname = "glfos-nix-samba";
  version = "1.0.11";

  src = fetchgit {
    url = "https://github.com/imikado/glfos-nix-samba";
    rev = version;
    sha256 = "sha256-w8kst5cpMddcqUz/F7lHUW60Mh44uYfNPyBv0ocoiLU=";
  };

  format = "other";

  nativeBuildInputs = [
    desktop-file-utils
    gettext
    gobject-introspection
    meson
    ninja
    wrapGAppsHook4
  ];

  buildInputs = [
    adwaita-icon-theme
    gtk4
    libadwaita
  ];

  propagatedBuildInputs = with python3Packages; [
    pygobject3
  ];

  meta = with lib; {
    description = "GTK application to configure Samba shares on GLF-OS";
    homepage = "https://github.com/imikado/glfos-nix-samba";
    license = licenses.gpl3Plus;
    mainProgram = "glfos-nix-samba";
    platforms = platforms.linux;
  };
}
