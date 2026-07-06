# ChargeGuard Release Process

This document describes the first public release flow for maintainers.

## Release Artifacts

Required for the first release:

- Git tag, for example `v0.1.0`
- GitHub Release created from that tag
- GNOME Shell extension zip built by `gnome-extensions pack`
- SHA-256 checksum file for uploaded release assets
- AUR source package metadata: `PKGBUILD`, `.SRCINFO`, and `chargeguard.install`

Standalone CLI binary archives are optional. A working ChargeGuard installation
also needs the daemon, D-Bus activation, polkit policy, and systemd integration,
so the AUR packages are the primary install path for Arch users.

## Version Bump

1. Update `Cargo.toml` workspace version.
2. Update `gnome-extension/metadata.json` `version` when the extension changes.
3. Update `packaging/arch/PKGBUILD` `pkgver` and reset `pkgrel=1`.
4. Regenerate `Cargo.lock` if Cargo metadata changes.

## Local Validation

Run these before tagging:

```bash
cargo fmt --all --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
python3 -m json.tool gnome-extension/metadata.json
mkdir -p /tmp/chargeguard-extension-pack
gnome-extensions pack --force --out-dir=/tmp/chargeguard-extension-pack gnome-extension
```

Build the Arch packages from the current checkout:

```bash
pkgver=$(awk -F= '/^pkgver=/{print $2}' packaging/arch/PKGBUILD)
git archive --format=tar --prefix="ChargeGuard-${pkgver}/" HEAD \
  | gzip -n > "packaging/arch/chargeguard-${pkgver}.tar.gz"
cd packaging/arch
CARGO_HOME=/tmp/cargo-home makepkg -f --cleanbuild --noconfirm
```

The `CI` workflow runs lightweight checks on pushes and pull requests. The
heavier `Package Check` workflow builds the GNOME extension zip and Arch packages
only when run manually or when a version tag is pushed.

## GitHub Release

1. Commit release metadata.
2. Create and push the version tag:

   ```bash
   git tag v0.1.0
   git push origin main v0.1.0
   ```

3. Run the `Release` GitHub Actions workflow, or let the tag trigger create a
   draft release.
4. Review the generated GNOME extension zip and checksum file.
5. Publish the GitHub Release when the assets are correct.

## AUR

After the tag is reachable from GitHub:

```bash
cd packaging/arch
updpkgsums
makepkg --printsrcinfo > .SRCINFO
makepkg -f --cleanbuild --noconfirm
```

Then update the AUR Git repository:

```bash
# From the ChargeGuard repository root:
cp packaging/arch/PKGBUILD /path/to/aur-chargeguard/PKGBUILD
cp packaging/arch/.SRCINFO /path/to/aur-chargeguard/.SRCINFO
cp packaging/arch/chargeguard.install /path/to/aur-chargeguard/chargeguard.install
cd /path/to/aur-chargeguard
git diff
git add PKGBUILD .SRCINFO chargeguard.install
git commit -m "Release 0.1.0"
git push
```

Use one AUR package base, `chargeguard`, with two generated packages:

- `chargeguard`
- `gnome-shell-extension-chargeguard`

Do not automate the AUR push until the manual release has been reviewed. A later
automation task should use a manual GitHub Actions trigger, a protected
environment, and an AUR SSH deploy key.

## Post-Install Smoke Check

On a supported machine:

```bash
chargeguard list
chargeguard current
chargeguard modes
chargeguard doctor --json
```

Enable the GNOME extension through the GNOME Extensions app or:

```bash
gnome-extensions enable chargeguard@github.com
gnome-extensions info chargeguard@github.com
```
