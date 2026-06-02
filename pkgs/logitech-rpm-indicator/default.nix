{
  lib,
  fetchFromGitHub,
  fetchurl,
  python3Packages,
  python3,
  hidapi,
  gtk4,
  libadwaita,
  glib,
  gobject-introspection,
  wrapGAppsHook4,
  makeWrapper,
  pkgsCross,
  unzip,
}:

# Packaging of IvanVojtko/logitech-linux-rpm-led
#
# Reads telemetry from racing games (Forza, F1, DiRT, Assetto Corsa,
# Automobilista 2, ETS2...) and drives the RPM shift-light LED bar of
# Logitech steering wheels via hidraw.
#
# Upstream: https://github.com/IvanVojtko/logitech-linux-rpm-led
#
# Notes Nix:
#  - Upstream has no pyproject.toml / setup.py. Its Makefile installs
#    the application under $out/lib/<pname>/ and creates a launcher
#    under $out/bin/<pname>.
#  - The `hid` PyPI package (Apmadsen ctypes binding) is not in nixpkgs.
#    We package it inline below and patch it to find libhidapi-hidraw.so
#    in the Nix store at build time.
#  - The wrapper runs under wrapGAppsHook4 so GTK4 + libadwaita +
#    GI_TYPELIB_PATH are correctly set for the GUI.
#  - The bundled ETS2 SCS telemetry plugin is built for native Linux and
#    Windows/Proton. Its SDK headers are fetched from the official SCS URL.
#  - Logitech wheel hidraw devices are accessible by users in the
#    `input` group on GLF-OS (default for `nobodyz`). No extra udev
#    rule needed for the MVP.

let
  scsSdk = fetchurl {
    url = "https://download.eurotrucksimulator2.com/scs_sdk_1_14.zip";
    hash = "sha256-xsH3N2tzJJlNn5xWfzxBQfu/MFtr+AO8TP7vJDeyAjo=";
  };

  # Apmadsen's ctypes binding for hidapi (PyPI package "hid").
  # Different from nixpkgs' python3Packages.hidapi (Cython binding).
  hid = python3Packages.buildPythonPackage rec {
    pname = "hid";
    version = "1.0.4";
    pyproject = true;

    src = python3Packages.fetchPypi {
      inherit pname version;
      hash = "sha256-9hsDgvN6M0vIuoYEvIS5SHXuT1lPu6+CssOz6CeIP8E=";
    };

    build-system = [ python3Packages.setuptools ];

    # Patch ctypes library lookup to point at our hidapi out-path so the
    # binding does not fall back to system "libhidapi-hidraw.so.0" lookup
    # (which fails on NixOS where libraries live under /nix/store).
    postPatch = ''
      find hid -name '*.py' -print0 | xargs -0 -r sed -i \
        -e 's|libhidapi-hidraw\.so\.0|${hidapi}/lib/libhidapi-hidraw.so.0|g' \
        -e 's|libhidapi-hidraw\.so|${hidapi}/lib/libhidapi-hidraw.so|g' \
        -e 's|libhidapi-libusb\.so\.0|${hidapi}/lib/libhidapi-libusb.so.0|g' \
        -e 's|libhidapi-libusb\.so|${hidapi}/lib/libhidapi-libusb.so|g'
    '';

    doCheck = false;
    pythonImportsCheck = [ "hid" ];

    meta = {
      description = "Python ctypes binding for hidapi";
      homepage = "https://github.com/apmorton/pyhidapi";
      license = lib.licenses.bsd3;
    };
  };
in
python3Packages.buildPythonApplication rec {
  pname = "logitech-rpm-indicator";
  version = "1.3.1";
  format = "other";

  src = fetchFromGitHub {
    owner = "IvanVojtko";
    repo = "logitech-linux-rpm-led";
    rev = "v${version}";
    hash = "sha256-H22bDClBmjRjg8QdaFev9ZuHkX9d9ABZB9c1QhlcPOY=";
  };

  nativeBuildInputs = [
    gobject-introspection
    makeWrapper
    pkgsCross.mingwW64.stdenv.cc
    unzip
    wrapGAppsHook4
  ];

  buildInputs = [
    gtk4
    libadwaita
    glib
  ];

  propagatedBuildInputs = with python3Packages; [
    hid
    pycairo
    pygobject3
  ];

  makeFlags = [
    "PREFIX=$(out)"
    "PYTHON_EXECUTABLE=${python3.interpreter}"
  ];

  # Avoid wrapGAppsHook4 double-wrapping the Python wrapper script.
  dontWrapGApps = true;

  postBuild = ''
    unzip -q ${scsSdk} -d scs_sdk_1_14
    make -C scs-plugin \
      SCS_SDK_DIR="$PWD/scs_sdk_1_14"
  '';

  # The wrapper MUST be created in postFixup, NOT installPhase: that's
  # where wrapGAppsHook4 (`gappsWrapperArgsHook`) has finished populating
  # `gappsWrapperArgs[@]` with GI_TYPELIB_PATH, XDG_DATA_DIRS and the
  # other env vars GTK4 + libadwaita need. If we wrapped in installPhase
  # the array was still empty and the launched python imports failed on
  # `gi.require_version('Gtk', '4.0')` -> "Namespace Gtk not available".
  postFixup = ''
    makeWrapper ${python3.interpreter} $out/bin/${pname} \
      --add-flags "$out/lib/${pname}/main.py" \
      --prefix PYTHONPATH : "${python3Packages.makePythonPath [
        hid
        python3Packages.pycairo
        python3Packages.pygobject3
      ]}" \
      "''${gappsWrapperArgs[@]}"
  '';

  meta = {
    description = "Drive the RPM shift LEDs of Logitech steering wheels on Linux";
    homepage = "https://github.com/IvanVojtko/logitech-linux-rpm-led";
    license = lib.licenses.gpl3Plus;
    platforms = lib.platforms.linux;
    mainProgram = pname;
  };
}
