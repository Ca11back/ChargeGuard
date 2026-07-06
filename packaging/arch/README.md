# Arch Package Test

This directory contains a local Arch/AUR-style package definition for testing
ChargeGuard from the current checkout.

Build the package:

```bash
cd packaging/arch
CARGO_HOME=/tmp/cargo-home makepkg -f --cleanbuild --noconfirm
```

Install the generated package:

```bash
sudo pacman -U ./chargeguard-0.1.0-1-x86_64.pkg.tar.zst
```

Run basic hardware checks:

```bash
chargeguard list
chargeguard current
chargeguard modes
chargeguard doctor --json
```

Test a native mode write only after confirming the current mode and supported
modes on the target machine:

```bash
chargeguard set Standard --device BAT1
```

Uninstall:

```bash
sudo pacman -R chargeguard
```
