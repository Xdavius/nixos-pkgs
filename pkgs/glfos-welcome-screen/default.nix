{
  lib,
  fetchgit,
  wrapGAppsHook4,
  meson,
  ninja,
  glib,
  desktop-file-utils,
  python3Packages,
  libadwaita,
  gtk4,
  gobject-introspection,
}:

python3Packages.buildPythonApplication rec {
  pname = "glfos-welcome-screen";
  version = "2.0.6";

  src = fetchgit {
    url = "https://github.com/imikado/glfos-welcome-screen";
    rev = version;
    sha256 = "sha256-rtZkPK99CVgHmyGXSvGaJOfaDE1GGfYIMA048KcGag4=";
  };

  format = "other";

  nativeBuildInputs = [
    desktop-file-utils
    glib
    gobject-introspection
    gtk4
    meson
    ninja
    wrapGAppsHook4
  ];

  buildInputs = [
    libadwaita
    gtk4
  ];

  propagatedBuildInputs = with python3Packages; [
    pygobject3
  ];

  postInstall = ''
    mkdir -p "$out/etc/xdg/autostart"
    cat > "$out/etc/xdg/autostart/glfos-welcome-screen.desktop" <<'EOF'
[Desktop Entry]
Name=GLF OS Welcome Screen
Exec=/run/current-system/sw/bin/glfos-welcome-screen
Icon=glfos-welcome-screen
Terminal=false
Type=Application
StartupNotify=true
StartupWMClass=org.dupot.glfos_welcome_screen
X-GNOME-Autostart-enabled=true
EOF
  '';

  meta = with lib; {
    description = "Welcome Screen";
    homepage = "https://github.com/imikado/glfos-welcome-screen";
    license = licenses.gpl3Plus;
    mainProgram = "glfos-welcome-screen";
    platforms = platforms.linux;
  };
}
