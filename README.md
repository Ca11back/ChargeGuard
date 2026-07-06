# ChargeGuard

ChargeGuard manages native Linux battery charging modes from a privileged
backend and desktop front ends.

The current implementation targets batteries that expose native charging modes
through Linux `power_supply` attributes:

```text
/sys/class/power_supply/<battery>/charge_types
/sys/class/power_supply/<battery>/charge_type
```

ChargeGuard validates writes against the device's current `charge_types`
allow-list. It does not write generic percentage thresholds or vendor legacy
controls.

## Components

- `chargeguard`: CLI for listing batteries, viewing modes, changing modes, and
  running diagnostics.
- `chargeguardd`: privileged system D-Bus service that owns polkit checks and
  sysfs writes.
- `gnome-shell-extension-chargeguard`: GNOME Quick Settings extension that talks
  to the system D-Bus backend.

## CLI

```bash
chargeguard list
chargeguard current
chargeguard modes
chargeguard set Long_Life --device BAT1
chargeguard doctor --json
chargeguard raw BAT1
```

Only run `set` after confirming the supported modes reported by your machine.

## Arch Packaging

The Arch/AUR package source lives in `packaging/arch`.

The package base is split into:

- `chargeguard`
- `gnome-shell-extension-chargeguard`

See `docs/release.md` for the maintainer release flow.

## Compatibility

See `docs/compatibility.md`.
