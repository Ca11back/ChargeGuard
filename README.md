# ChargeGuard

English | [简体中文](README.zh-CN.md)

ChargeGuard brings your laptop's native battery charging modes into GNOME Quick
Settings. It is for machines whose firmware already exposes charging modes such
as `Fast`, `Standard`, or `Long_Life` through Linux `power_supply`.

The Quick Settings tile shows the current charging mode and battery-side power
when the battery reports it. Expand the tile to switch between firmware-reported
modes or open the battery details page.

![ChargeGuard Quick Settings tile and menu](docs/assets/chargeguard-gnome-quick-settings.png)

## GNOME Experience

ChargeGuard is designed to sit beside GNOME's built-in battery indicator and
power mode control without duplicating them.

- The compact Quick Settings tile shows the current native charging mode, such
  as `Long Life`, plus current battery-side watts when available.
- The expanded menu identifies the tool as ChargeGuard, shows the current
  mode/power summary, lists the writable firmware modes, and provides a
  `Details...` action.
- The mode list uses raw firmware tokens for the actual choices, such as
  `Fast`, `Standard`, and `Long_Life`.

## Battery Details

The details page is for information that does not need to occupy the top-level
Quick Settings surface: battery identity, health, current power, and the exact
Linux interface ChargeGuard is using.

![ChargeGuard battery details page](docs/assets/chargeguard-battery-details.png)

On machines that report the data through `power_supply`, the page can show:

- selected battery, model, manufacturer, status, and capacity
- battery-side power from `power_now`
- cycle count and capacity level
- full capacity, design capacity, and calculated battery health
- current native mode, raw mode token, supported raw tokens, and the full native
  mode source path

These fields are hardware-dependent. Missing optional sysfs attributes are shown
as unavailable rather than treated as errors.

## Scope

ChargeGuard is a front end for firmware-provided charging modes. It does not
invent a software charging policy, replace GNOME's power mode control, or act as
a general laptop power-tuning tool.

The supported native mode interface is:

```text
/sys/class/power_supply/<battery>/charge_types
/sys/class/power_supply/<battery>/charge_type
```

Mode changes are validated against the battery's current `charge_types`
allow-list. ChargeGuard does not write generic percentage thresholds such as
`charge_control_end_threshold`, and it does not tune CPU, disk, USB, Wi-Fi,
platform profiles, keyboard backlight, or runtime power-saving settings.

Use TLP, GNOME's power mode control, or another power-management tool for broad
performance and power-saving policy. Use ChargeGuard when your laptop already
has firmware charging modes and you want them exposed cleanly in GNOME and CLI
diagnostics.

## Components

- `chargeguard`: CLI for listing batteries, viewing modes, changing modes, and
  running diagnostics.
- `chargeguardd`: privileged system D-Bus service that owns polkit checks and
  sysfs writes.
- `gnome-shell-extension-chargeguard`: GNOME Quick Settings extension that talks
  to the system D-Bus backend.

## Install

Arch users should install the GNOME extension package for the desktop
experience. It depends on the backend package:

```bash
yay -S gnome-shell-extension-chargeguard
```

CLI-only users can install the backend package:

```bash
yay -S chargeguard
```

Maintainers can build the Arch packages from this checkout. See
[packaging/arch/README.md](packaging/arch/README.md).

## Enable The GNOME Extension

The extension targets GNOME Shell 50.

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
`charge_types` firmware modes. Health and power details depend on the optional
attributes reported by each battery under `power_supply`.

See [docs/compatibility.md](docs/compatibility.md) for details.

## Release And AUR Maintenance

See [docs/release.md](docs/release.md) for the maintainer release flow, GitHub
Release artifacts, and AUR publication steps.
