# ChargeGuard

English | [简体中文](README.zh-CN.md)

Switch the battery charging modes already built into your laptop firmware right
from GNOME Quick Settings. ChargeGuard also provides focused battery details
and CLI diagnostics, while leaving general performance and power tuning to the
tools designed for those jobs.

ChargeGuard is useful only when the Linux kernel exposes firmware-native modes
such as `Fast`, `Standard`, or `Long_Life`. Support depends on the exact laptop,
firmware, and kernel driver—not the vendor name alone.

![ChargeGuard mode switcher in GNOME Quick Settings](docs/assets/chargeguard-gnome-quick-settings.png)

## What You Get

- A GNOME Quick Settings switcher for modes reported by your battery firmware.
- A battery details view for health, power, identity, and native-mode data when
  the hardware provides it.
- A CLI for read-first inspection, local diagnostics, and guarded mode changes.

ChargeGuard complements GNOME's battery indicator and Power Modes. It does not
create charging algorithms, manage percentage thresholds, or tune CPU, disk,
USB, Wi-Fi, and other system power settings.

## Will It Work on My Laptop?

Check the kernel interface before installing. This command only reads sysfs; it
does not change the charging mode:

```bash
for battery in /sys/class/power_supply/*; do
  [ -d "$battery" ] || continue
  [ -r "$battery/type" ] || continue
  [ "$(cat "$battery/type")" = "Battery" ] || continue
  echo "== $(basename "$battery") =="
  for attribute in charge_types charge_type \
    charge_control_start_threshold charge_control_end_threshold; do
    if [ -r "$battery/$attribute" ]; then
      printf '%s: ' "$attribute"
      cat "$battery/$attribute"
    fi
  done
done
```

Interpret the result conservatively:

| What you find | ChargeGuard behavior |
| --- | --- |
| `charge_types` (with or without a separate `charge_type`) | Native modes can be inspected and are offered for switching. A later write can still fail because of firmware, driver, or authorization behavior. |
| `charge_type` only | Current mode can be visible, but ChargeGuard treats the device as read-only because it cannot validate a safe list of choices. |
| Only `charge_control_*_threshold` or similar controls | This is threshold-based charging and is outside ChargeGuard's scope. |
| None of these files | The laptop is unsupported or its current firmware/kernel does not expose the interface. |

See [Compatibility](docs/compatibility.md) for the precise boundary. Do not
infer support from another model made by the same vendor.

## Install on Arch Linux

For the GNOME experience, install the extension package; it depends on the
backend automatically:

```bash
yay -S gnome-shell-extension-chargeguard
```

For CLI-only use:

```bash
yay -S chargeguard
```

The extension currently targets GNOME Shell 50. After installation or an
upgrade, log out and back in if GNOME Wayland has not loaded the new extension,
then enable it:

```bash
gnome-extensions enable chargeguard@github.com
gnome-extensions info chargeguard@github.com
```

## Safe Quick Start

Start with inspection. These commands do not change the charging mode:

```bash
chargeguard list
chargeguard current
chargeguard modes
chargeguard doctor
```

If more than one battery is present, pass the device shown by `list`, for
example `chargeguard current BAT1` and `chargeguard modes BAT1`.

In GNOME, open Quick Settings and expand ChargeGuard. The menu shows the current
mode and the firmware-reported choices. Open **Details...** for battery and
interface information.

![ChargeGuard battery health and native-mode details](docs/assets/chargeguard-battery-details.png)

Optional fields vary by hardware. Missing health, cycle, power, or identity
attributes are shown as unavailable; explicit collection failures remain visible
in diagnostics.

## How ChargeGuard Differs

| Tool | Primary job | Relationship to charging modes |
| --- | --- | --- |
| ChargeGuard | GNOME and CLI access to firmware-native `charge_type` / `charge_types` modes | Uses only the raw choices reported by the battery; no percentage-threshold policy |
| GNOME Power Modes | Select system performance/energy profiles such as power saver or performance | A different control; ChargeGuard does not replace it |
| TLP | Broad laptop power management and battery-care configuration | May manage battery features on supported hardware; avoid configuring two tools to write the same control |
| Threshold tools | Set charge start/stop percentages where the driver exposes threshold attributes | A different interface and policy model; ChargeGuard does not manage thresholds |

These tools can coexist when they do not manage the same firmware control. The
capabilities of TLP and threshold tools remain hardware- and driver-dependent.

## Advanced CLI Use

For a shareable structured diagnostic snapshot, use:

```bash
chargeguard doctor --json
```

Only use `set` after `modes` has shown the exact raw token for the target
battery:

```bash
chargeguard modes BAT1
chargeguard set Long_Life --device BAT1
chargeguard current BAT1
```

`set` is the only mutating command. The daemon validates the requested token
against the battery's current reported choices, writes it through the native
interface, and reads it back. A reported choice is not a guarantee that the
firmware or driver will accept the later write.

Run `man chargeguard` for the complete command, device-selection, diagnostics,
error, and exit-status reference.

## Safety, Privacy, and Limitations

- Inspection commands do not require set-mode polkit authorization. Mode
  changes go through the privileged `chargeguardd` service and polkit.
- ChargeGuard writes only exact native mode tokens currently reported for the
  selected battery. It does not write generic percentage thresholds.
- ChargeGuard has no telemetry and does not transmit diagnostics. `doctor` and
  `raw` output can contain device identifiers, model data, sysfs paths, and
  driver-provided values, so review it before sharing.
- Hardware fields, mode names, write behavior, and support can change with
  firmware or kernel updates. Multi-battery systems require explicit selection.

## Troubleshooting and Further Reading

- No mode switcher or an unsupported/read-only result: rerun the preflight and
  read [Compatibility](docs/compatibility.md).
- More than one battery: use the device ID from `chargeguard list` with
  `current`, `modes`, `raw`, or `set --device`.
- Failed inspection: run `chargeguard doctor` and keep its explicit failure
  entries when reporting the problem.
- Extension missing after installation: verify GNOME Shell 50, log out and back
  in on Wayland, then check `gnome-extensions info chargeguard@github.com`.
- System design and privilege boundaries: [Architecture](docs/architecture.md).
- Bugs and support requests: [GitHub Issues](https://github.com/Ca11back/ChargeGuard/issues).

Maintainer and packaging procedures are intentionally outside this user guide:
[release runbook](https://github.com/Ca11back/ChargeGuard/blob/main/docs/release.md)
and [Arch package notes](https://github.com/Ca11back/ChargeGuard/blob/main/packaging/arch/README.md).
