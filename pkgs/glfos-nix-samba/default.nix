{
  lib,
  fetchgit,
  wrapGAppsHook4,
  meson,
  ninja,
  glib,
  desktop-file-utils,
  gettext,
  python3Packages,
  adwaita-icon-theme,
  libadwaita,
  gtk4,
}:

python3Packages.buildPythonApplication rec {
  pname = "glfos-nix-samba";
  version = "1.0.10";

  src = fetchgit {
    url = "https://github.com/imikado/glfos-nix-samba";
    rev = version;
    sha256 = "sha256-WtNVTtfY9agbD/4EnTEPwSb/7yWlT2d3KR0yyhTg5wU=";
  };

  format = "other";

  nativeBuildInputs = [
    desktop-file-utils
    gettext
    glib
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

  # Let buildPythonApplication create a single wrapper containing both the
  # Python path and the GTK environment collected by wrapGAppsHook4.
  dontWrapGApps = true;

  preFixup = ''
    makeWrapperArgs+=("''${gappsWrapperArgs[@]}")
  '';

  meta = with lib; {
    description = "GTK application to configure Samba shares on GLF-OS";
    homepage = "https://github.com/imikado/glfos-nix-samba";
    license = licenses.gpl3Plus;
    mainProgram = "glfos-nix-samba";
    platforms = platforms.linux;
  };
}
