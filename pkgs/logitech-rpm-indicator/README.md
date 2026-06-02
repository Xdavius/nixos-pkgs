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

The derivation currently pins upstream release `v1.3.1`.

## Layout (post-install)

```
$out/
├── bin/logitech-rpm-indicator           ← wrapper that wraps Python + GTK4
├── lib/logitech-rpm-indicator/
│   ├── main.py
│   ├── games/
│   ├── wheels/
│   ├── icons/
│   └── scs-plugin/
│       ├── logitech_rpm_telemetry.so    ← native Linux plugin
│       └── logitech_rpm_telemetry.dll   ← Windows/Proton plugin
├── share/applications/
│   └── io.github.IvanVojtko.LogitechRpmIndicator.desktop
├── share/icons/hicolor/
│   ├── 256x256/apps/
│   │   └── logitech-rpm-indicator.png
│   └── scalable/apps/
│       └── logitech-rpm-indicator.svg
└── share/doc/logitech-rpm-indicator/
    ├── README.md
    └── LICENSE
```

## Permissions

The package does NOT ship an udev rule. On GLF-OS the default user
`nobodyz` belongs to the `input` group which is sufficient to access
`/dev/hidraw*` of Logitech wheels. If you need to grant access to a
different user on a non-GLF-OS system, add them to `input` or ship
an udev rule on your own.

## ETS2 SCS plugin

The package builds both upstream ETS2 telemetry plugin variants:
`logitech_rpm_telemetry.so` for native Linux and
`logitech_rpm_telemetry.dll` for Windows/Proton. In the GUI, select
Euro Truck Simulator 2 and use the plugin installation button to copy
the bundled files into the detected Steam installation.

## License

GPL-3.0 (same as upstream).
