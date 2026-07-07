# ChargeGuard

English | [简体中文](README.zh-CN.md)

ChargeGuard puts your laptop's built-in battery charging modes in GNOME Quick
Settings. If your firmware already exposes modes such as `Fast`, `Standard`, or
`Long_Life` through Linux, ChargeGuard lets you see and switch those modes
without remembering sysfs paths or running privileged commands by hand.

![ChargeGuard in GNOME Quick Settings](docs/assets/chargeguard-gnome-quick-settings.png)

## What It Does

ChargeGuard is for laptops that expose native firmware charging modes through
the Linux `power_supply` interface. It does not invent a battery-care policy of
its own. It reads the mode list reported by the machine, shows it in GNOME Quick
Settings, and asks the firmware to switch to one of those reported modes.
In other words, ChargeGuard is a front end for firmware-provided controls, not a
software charging algorithm.

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

## How It Differs From TLP

[TLP](https://linrunner.de/tlp/index.html) is a broad laptop power-management
tool. It can apply power-saving profiles, tune many kernel power settings,
manage radios and devices, and, on supported laptops, configure battery charge
thresholds.

ChargeGuard is intentionally narrower:

- It only manages firmware charging modes exposed as `charge_type` /
  `charge_types`.
- It does not tune CPU, disk, USB, Wi-Fi, platform, or runtime power-saving
  settings.
- It does not set start/stop charge percentages such as `75%` / `80%`.
- It presents the firmware's own mode names and writes only a mode token that
  the device currently reports as supported.

In practice, use TLP or your desktop power-profile tool when you want general
power saving or performance profiles. Use ChargeGuard when your laptop already
has firmware charging modes and you want a simple GNOME switch for them.

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
