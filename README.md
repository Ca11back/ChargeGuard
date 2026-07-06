# ChargeGuard

English | [简体中文](README.zh-CN.md)

ChargeGuard manages native Linux battery charging modes from GNOME Quick
Settings, backed by a small privileged D-Bus service.

![ChargeGuard in GNOME Quick Settings](docs/assets/chargeguard-gnome-quick-settings.png)

## What It Does

ChargeGuard is for laptops that expose firmware battery modes through the Linux
`power_supply` interface. On supported machines, it can switch modes such as
standard charging, fast charging, or battery care / long-life charging directly
from GNOME Quick Settings.

The current implementation targets native charging mode attributes:

```text
/sys/class/power_supply/<battery>/charge_types
/sys/class/power_supply/<battery>/charge_type
```

ChargeGuard validates writes against the device's current `charge_types`
allow-list. It does not write generic percentage thresholds or vendor legacy
controls.

The GNOME extension shows native mode names as reported by the firmware, for
example `Fast`, `Standard`, or `Long_Life`. These mode tokens are not translated
or reinterpreted by the UI.

## Components

- `chargeguard`: CLI for listing batteries, viewing modes, changing modes, and
  running diagnostics.
- `chargeguardd`: privileged system D-Bus service that owns polkit checks and
  sysfs writes.
- `gnome-shell-extension-chargeguard`: GNOME Quick Settings extension that talks
  to the system D-Bus backend.

## Install

The intended Arch/AUR package base is split into two packages:

- `chargeguard`
- `gnome-shell-extension-chargeguard`

After AUR publication, desktop users should install the extension package. It
depends on the backend package:

```bash
yay -S gnome-shell-extension-chargeguard
```

CLI-only users can install the backend package:

```bash
yay -S chargeguard
```

Until the AUR package is published, maintainers can build the Arch packages from
this checkout. See [packaging/arch/README.md](packaging/arch/README.md).

## Enable The GNOME Extension

The first extension targets GNOME Shell 50.

After installing the package, reload or restart the GNOME Shell session as
appropriate for your session type, then enable the extension:

```bash
gnome-extensions enable chargeguard@github.com
gnome-extensions info chargeguard@github.com
```

The backend is activated through system D-Bus when the CLI or extension queries
ChargeGuard.

## CLI

```bash
chargeguard list
chargeguard current
chargeguard modes
chargeguard doctor --json
chargeguard raw BAT1
```

Only run `set` after confirming the supported modes reported by your machine:

```bash
chargeguard set Long_Life --device BAT1
```

## Safety Model

- Read operations do not require polkit authorization.
- Mode changes go through `chargeguardd` and polkit.
- The daemon writes only native mode tokens that are currently reported by the
  target battery's `charge_types` file.
- Devices without an enumerable writable native mode list are reported as
  unsupported or read-only.

## Compatibility

ChargeGuard currently focuses on laptops with native `charge_type` /
`charge_types` firmware modes. It does not manage percentage thresholds such as
`charge_control_end_threshold`.

See [docs/compatibility.md](docs/compatibility.md) for details.

## Release And AUR Maintenance

See [docs/release.md](docs/release.md) for the maintainer release flow, GitHub
Release artifacts, and AUR publication steps.
