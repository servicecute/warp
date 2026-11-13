# Custom Warp release workflow

The ServiceCute fork can be built and installed as a separate macOS app from
the stable Warp installation.

## Build and install

From the repository root:

```bash
./script/build-custom-release
```

The workflow builds the current checkout as:

```text
/Applications/WarpOss.app
```

The custom app uses the existing OSS channel identity:

```text
Application name:   WarpOss
Bundle identifier:  dev.warp.WarpOss
Binary:             warp-oss
```

This is intentionally separate from the stable installation:

```text
Stable Warp:  /Applications/Warp.app       dev.warp.Warp-Stable
Custom Warp:  /Applications/WarpOss.app    dev.warp.WarpOss
```

The two applications therefore use separate app data profiles and can be run
side by side without confusing the custom build with stable Warp.

## Common options

```bash
# Replace an existing custom app without prompting.
./script/build-custom-release --force

# Build, install, and open the custom app.
./script/build-custom-release --force --open

# Install into a user-owned Applications directory.
./script/build-custom-release \
  --install-dir "$HOME/Applications" \
  --force

# Build only and leave the bundle under target/release.
./script/build-custom-release --no-install
```

The script uses Warp's existing release bundling path and local/ad-hoc
development signing. It does not modify or replace `/Applications/Warp.app`.
It also disables the common-skills installation check because a build should
not mutate the user's skills installation as a side effect.

If Cargo needs the Apple Metal toolchain on the host, run it with the same
environment used by the local build, for example:

```bash
TOOLCHAINS=Metal MACOSX_DEPLOYMENT_TARGET=15.0 \
  ./script/build-custom-release --force --open
```

The workflow is intended for local review, not notarized public distribution.
