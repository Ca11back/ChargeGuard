# ChargeGuard Architecture

ChargeGuard is split into a thin CLI frontend, a privileged D-Bus daemon, and a shared Rust core.

```text
chargeguard CLI
  -> system D-Bus
  -> chargeguardd
  -> polkit authorization for SetNativeMode
  -> validated power_supply sysfs access
```

The core crate owns battery discovery, `charge_types` parsing, mode normalization, diagnostics models, and sysfs read/write validation. The daemon owns D-Bus, polkit, privileged writes, and logging. The CLI owns command parsing and rendering.

The MVP writes only native mode tokens selected from the current `charge_types` allow-list. It does not write generic threshold or vendor legacy controls.
