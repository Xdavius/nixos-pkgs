{ lib
, stdenv
, fetchurl
, appimageTools
, autoPatchelfHook
, makeWrapper
, pcsclite
, libusb1
, udev
, glib
, musl
, gtk3
, dbus-glib
, libdbusmenu-gtk2
, libdbusmenu
, libgbm
, libdrm
, nss
, alsa-lib
}:

let
  pname = "tachyfy";
  version = "1.0.29";

  src = fetchurl {
    url = "https://tachyfy.app/updates/Tachyfy-${version}.AppImage";
    hash = "sha256-YYy72bKByNGq8GpLkKFHMbf8FpJ0/KEyKs1csYzy3VU=";
  };

  appimageContents = appimageTools.extractType2 {
    inherit pname version src;
  };
in stdenv.mkDerivation {
  inherit pname version src;

  nativeBuildInputs = [ autoPatchelfHook makeWrapper ];
  buildInputs = [ pcsclite
  		  libusb1 
  		  udev 
                  glib
                  musl 
                  gtk3 
                  dbus-glib
                  libdbusmenu-gtk2
                  libdbusmenu
                  libgbm
                  libdrm
                  nss
                  alsa-lib
                  ];

  unpackPhase = "true";

  installPhase = ''
    mkdir -p $out/share/tachyfy
    cp -r ${appimageContents}/* $out/share/tachyfy

    mkdir -p $out/bin
    if [ -f "$out/share/tachyfy/AppRun" ]; then
      APPBIN=$out/share/tachyfy/AppRun
    else
      APPBIN=$(find $out/share/tachyfy -maxdepth 2 -type f -executable | head -n 1)
    fi

    makeWrapper $APPBIN $out/bin/tachyfy \
      --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [ pcsclite libusb1 udev ]}

    # Créer un .desktop minimal si absent
  mkdir -p $out/share/applications
  DESKTOP_FILE="$out/share/applications/tachyfy.desktop"
  cat > $DESKTOP_FILE <<EOF
[Desktop Entry]
Name=Tachyfy
Comment=Tachyfy AppImage
Exec=$out/bin/tachyfy
Icon=$out/share/icons/tachyfy-electron.png
Type=Application
Categories=Utility;
EOF

    if [ -d "$out/share/tachyfy/usr/share/icons/hicolor/0x0/apps/" ]; then
      mkdir -p $out/share/icons
      cp -r $out/share/tachyfy/usr/share/icons/hicolor/0x0/apps/* $out/share/icons/
    fi
  '';

  postFixup = ''
    autoPatchelf $out/share/tachyfy || true
  '';

  meta = with lib; {
    description = "Tachyfy AppImage repackaged for NixOS";
    platforms = platforms.linux;
  };
}
