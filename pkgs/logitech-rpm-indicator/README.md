# logitech-rpm-indicator

NixOS packaging of [IvanVojtko/logitech-linux-rpm-led](https://github.com/IvanVojtko/logitech-linux-rpm-led),
shipped with [GLF-OS](https://framagit.org/gaming-linux-fr/glf-os/glf-os) for
users with Logitech racing wheels.

The upstream project listens to telemetry from supported racing games
(Forza Horizon 5/6, F1 2019/20/22/23, DiRT Rally 2.0, Automobilista 2,
Assetto Corsa, Euro Truck Simulator 2…) and drives the RPM shift-light
LED bar of Logitech steering wheels via hidraw.

## Build

```bash
nix-build ./.
```

The first build will fail with two `lib.fakeHash` mismatches (one for
the GitHub source, one for the `hid` PyPI package). Replace each hash
with the value Nix prints in the error and rebuild.

## Layout (post-install)

```
$out/
├── bin/logitech-rpm-indicator           ← wrapper that wraps Python + GTK4
├── share/logitech-rpm-indicator/
│   ├── main.py
│   ├── games/
│   ├── wheels/
│   └── icons/
├── share/applications/
│   └── logitech-rpm-indicator.desktop
├── share/icons/hicolor/256x256/apps/
│   └── logitech-rpm-indicator.png
└── share/doc/logitech-rpm-indicator/
    ├── README.md
    ├── LICENSE
    └── scs-plugin/                      ← ETS2 SCS plugin source (not built)
```

## Permissions

The package does NOT ship an udev rule. On GLF-OS the default user
`nobodyz` belongs to the `input` group which is sufficient to access
`/dev/hidraw*` of Logitech wheels. If you need to grant access to a
different user on a non-GLF-OS system, add them to `input` or ship
an udev rule on your own.

## ETS2 SCS plugin

The upstream `scs-plugin/` (Euro Truck Simulator 2 telemetry plugin,
compiled Windows DLL) is shipped as documentation/source under
`$out/share/doc/logitech-rpm-indicator/scs-plugin/`. Building it
requires `mingw-w64`. If you actually use ETS2 over Proton, you can
build it manually:

```bash
make -C $out/share/doc/logitech-rpm-indicator/scs-plugin/ \
  SCS_SDK_DIR=/path/to/scs_sdk_1_14
```

## License

GPL-3.0 (same as upstream).
