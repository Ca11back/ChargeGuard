# Arch and AUR Packaging

This directory contains the Arch/AUR package source for ChargeGuard.

The package is intentionally split:

- `chargeguard` installs the privileged backend, CLI, D-Bus activation files,
  polkit policy, systemd unit, docs, and license.
- `gnome-shell-extension-chargeguard` installs the GNOME Quick Settings
  extension, gettext translations, and depends on `chargeguard`.

## Local Package Check

Before the first GitHub tag exists, create a local source archive from the
current checkout so `makepkg` can exercise the release-style `PKGBUILD` without
downloading from GitHub:

```bash
pkgver=$(awk -F= '/^pkgver=/{print $2}' packaging/arch/PKGBUILD)
git archive --format=tar --prefix="ChargeGuard-${pkgver}/" HEAD \
  | gzip -n > "packaging/arch/chargeguard-${pkgver}.tar.gz"
cd packaging/arch
CARGO_HOME=/tmp/cargo-home makepkg -f --cleanbuild --noconfirm
```

Install the generated packages:

```bash
sudo pacman -U ./chargeguard-0.1.1-1-x86_64.pkg.tar.zst \
  ./gnome-shell-extension-chargeguard-0.1.1-1-x86_64.pkg.tar.zst
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

## AUR Publication

After the release tag exists on GitHub, regenerate the integrity field from the
real tag archive before pushing to AUR:

```bash
cd packaging/arch
updpkgsums
makepkg --printsrcinfo > .SRCINFO
makepkg -f --cleanbuild --noconfirm
```

Review `PKGBUILD` and `.SRCINFO`, then copy them and `chargeguard.install` into
the AUR Git repository for `chargeguard`.

The checked-in `sha256sums=("SKIP")` keeps local pre-tag validation possible.
Do not push that value to AUR for a stable release; replace it with the checksum
generated from the published tag archive.
