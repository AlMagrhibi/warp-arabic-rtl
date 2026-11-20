# Building this fork

## Prerequisites (Fedora)

```bash
sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config protobuf-compiler \
  openssl-devel fontconfig-devel freetype-devel \
  wayland-devel libxkbcommon-devel mesa-libEGL-devel \
  libX11-devel libXcursor-devel libXi-devel libXrandr-devel \
  gtk3-devel libappindicator-gtk3-devel \
  alsa-lib-devel \
  fuse-libs lld
```

On Debian/Ubuntu the equivalent list is in `script/linux/install_build_deps` upstream.

Rust: the repository pins a toolchain in `rust-toolchain.toml`. Either install that version
with `rustup`, or use a recent stable toolchain — this fork was built with `rustc 1.100.0`
(system toolchain, no rustup).

## Build the binary

```bash
cargo build --release --bin warp-oss --features release_bundle
```

Notes:

- `--offline` works if the crate registry cache is already populated.
- On a 16 GB machine, limit parallelism (`CARGO_BUILD_JOBS=4`) and use `lld`
  (`RUSTFLAGS="-C link-arg=-fuse-ld=lld"`) to keep peak memory down.
- The OSS channel ships without Sentry, hence the `release_bundle` (no `crash_reporting`)
  feature set.

Result: `target/release/warp-oss`.

## Package the AppImage

The upstream packaging script expects the binary at the profile directory it derives from the
channel. For the `oss` channel that is `target/release-lto/`:

```bash
mkdir -p target/release-lto
cp target/release/warp-oss target/release-lto/warp-oss
./script/bundle -c oss --packages appimage --skip-build
```

The script downloads `linuxdeploy` into `~/.local/bin` on first use and writes the AppImage to
`target/release-lto/bundle/linux/`.

## Run the tests

```bash
cargo test -p warp --lib terminal::bidi_row
cargo test -p warp --lib terminal::grid_renderer
```

## Reproduce the verification done for a release

```bash
# 1. automated
cargo test -p warp --lib terminal::bidi_row        # 5 pass
cargo test -p warp --lib terminal::grid_renderer   # 15 pass, 1 ignored (macOS-only font)

# 2. binary really contains the fork
strings target/release/warp-oss | grep -c grid_rows_need_bidi_rendering || true

# 3. run it on a scratch profile and look at Arabic output
HOME=/tmp/warp-scratch ./target/release-lto/bundle/linux/*.AppImage
```
