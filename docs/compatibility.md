# ChargeGuard Compatibility

## GNOME Shell

The GNOME extension supports GNOME Shell 50 and 51. After installing or updating
the extension, log out and log back in so GNOME Shell reloads its metadata, then
enable `chargeguard@github.com` in GNOME Extensions.

## Battery Support

The MVP targets Linux batteries that expose native charging modes through:

```text
/sys/class/power_supply/<battery>/charge_types
/sys/class/power_supply/<battery>/charge_type
```

Writable support requires an enumerable `charge_types` list. Devices that expose only `charge_type` are reported as read-only because ChargeGuard cannot validate a safe writable mode allow-list.

Threshold and vendor legacy attributes such as `charge_control_end_threshold`, `charge_control_start_threshold`, `conservation_mode`, and `charge_behaviour` may appear in diagnostics, but they are informational only for this MVP.
