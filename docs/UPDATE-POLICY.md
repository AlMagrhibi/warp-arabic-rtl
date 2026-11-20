# Update policy

**This build never updates itself. Nothing can silently replace it with an official Warp
release.**

## What is compiled in

The updater is gated on the `autoupdate` Cargo feature:

```rust
// app/src/features.rs
flags.extend([
    #[cfg(feature = "autoupdate")]
    FeatureFlag::Autoupdate,
    ...
]);
```

This fork is built with `--features release_bundle` only. `autoupdate` is **not** in the
feature list, so `FeatureFlag::Autoupdate` is never enabled, the poll loop never registers and
no update endpoint is ever contacted.

## What upstream would do even if the feature were on

The binary is built on the **OSS channel**. Upstream refuses updates for that channel in two
places:

```rust
// app/src/autoupdate/mod.rs
Channel::Integration | Channel::Local | Channel::Oss => {
    anyhow::bail!("Local, integration, and open-source channel binaries don't support autoupdate");
}

// release_assets_directory_url
Channel::Local | Channel::Integration | Channel::Oss => {
    unreachable!("local/integration/oss autoupdate not supported");
}
```

## What is installed on the system

The AppImage is a single file that the user copies by hand. This project installs:

- no RPM/DEB package,
- no systemd unit or timer,
- no repository file and no package-manager hook,
- no background updater process.

Therefore no package manager, cron job, or system service is able to replace the binary.

## How updates actually happen

Manually, from this fork's GitHub Releases:

1. Read the release notes in <https://github.com/AlMagrhibi/warp-arabic-rtl/releases>.
2. Download the new AppImage **and** its `.sha256`.
3. Verify: `sha256sum -c warp-terminal-arabic-<version>-x86_64.AppImage.sha256`.
4. Replace your local copy.
5. Confirm the Arabic rendering still works (`test-arabic.sh` in the release assets).

## Source of truth

| Question | Answer |
|---|---|
| Which source builds the released AppImage? | The git tag in this repository, commit recorded in each release note. |
| Is upstream Warp authoritative for this build? | No. Upstream is a **development reference only**; it is read, never merged automatically, and never used as an update channel. |
| May an official Warp build replace this one? | No. Nothing in this project ever downloads a Warp release asset. |
| If an RPM is produced later? | It must be built and published by this fork, from this fork's source, in this fork's own repository — an official Warp RPM must never be used as an update path. |

## Preserving the BiDi patches across future releases

The feature lives in three places; any future sync with upstream must re-apply them:

- `app/src/terminal/bidi_row.rs` (new file, carry over as-is)
- `app/src/terminal/grid_renderer.rs` — `grid_rows_need_bidi_rendering()`,
  `row_has_bidi`, `paint_line(..., use_shaped_positions)`
- `app/Cargo.toml` — `unicode-bidi` dependency, and `mod bidi_row;` in
  `app/src/terminal/mod.rs`

The two guards to run after every upstream sync:

```bash
cargo test -p warp --lib terminal::bidi_row      # must stay 5/5 green
grep -c grid_rows_need_bidi_rendering app/src/terminal/grid_renderer.rs   # must be >= 2
```
