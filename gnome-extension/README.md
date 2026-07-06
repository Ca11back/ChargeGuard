# ChargeGuard GNOME Extension

This directory contains the first ChargeGuard GNOME Shell extension. It uses
GNOME Quick Settings as the primary surface and talks to the existing
`io.github.ChargeGuard1` system D-Bus backend.

Packaging and release publication are intentionally outside this directory.

## Local Development

Install the extension for the current user from this checkout:

```bash
mkdir -p ~/.local/share/gnome-shell/extensions
ln -sfn "$PWD/gnome-extension" \
  ~/.local/share/gnome-shell/extensions/chargeguard@github.com
```

Reload the GNOME Shell session as appropriate for the session type, then enable
the extension:

```bash
gnome-extensions enable chargeguard@github.com
gnome-extensions info chargeguard@github.com
```

Do not run `gnome-extensions install --force` while the user extension path is a
symlink into this repository. On some setups the install command can replace or
remove the symlink target. Use the symlink workflow for development, or use a
copy/bundle workflow separately.

The backend must already be installed and activatable on the system D-Bus. Use
the CLI for backend smoke checks:

```bash
chargeguard list
chargeguard current
chargeguard modes
```
