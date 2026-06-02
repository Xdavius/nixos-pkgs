{
  lib,
  fetchgit,
  wrapGAppsHook4,
  meson,
  ninja,
  glib,
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
  version = "1.0.10";

  src = fetchgit {
    url = "https://github.com/imikado/glfos-nix-samba";
    rev = version;
    sha256 = "sha256-WtNVTtfY9agbD/4EnTEPwSb/7yWlT2d3KR0yyhTg5wU=";
  };

  postPatch = ''
    substituteInPlace src/infrastructure/ui/app_window.py \
      --replace-fail \
        '    def do_activate(self):
        win = MainWindow(application=self)' \
        '    def do_activate(self):
        Gtk.Settings.get_default().set_property("gtk-icon-theme-name", "Adwaita")
        win = MainWindow(application=self)'
  '';

  format = "other";

  nativeBuildInputs = [
    desktop-file-utils
    gettext
    glib
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

  # The application uses symbolic icons from Adwaita. This is applied by the
  # Python wrapper before wrapGAppsHook4 adds the remaining GTK environment.
  makeWrapperArgs = [
    "--prefix"
    "XDG_DATA_DIRS"
    ":"
    "${adwaita-icon-theme}/share"
  ];

  meta = with lib; {
    description = "GTK application to configure Samba shares on GLF-OS";
    homepage = "https://github.com/imikado/glfos-nix-samba";
    license = licenses.gpl3Plus;
    mainProgram = "glfos-nix-samba";
    platforms = platforms.linux;
  };
}
