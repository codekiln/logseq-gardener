# Installation

The foundation is available from a source checkout. Install mise, enter the checkout, and run `mise install`, then `mise run build:release`. mise may ask you to trust the checked-out configuration before installing its pinned tools.

Run `./target/release/lsg help` to verify the build. On Windows, use `target/release/lsg.exe`. With the pinned environment installed, `mise exec -- cargo install --path . --locked` installs the executable into Cargo's binary directory. Add that directory (usually `~/.cargo/bin`) to PATH and run `lsg version`.

The build requires the native linker for your platform: Xcode command-line tools on macOS, a C build toolchain on Linux, or Visual Studio C++ build tools on Windows. The current full contributor task suite uses Bash and is exercised by Linux CI.

## Binary releases

There is no published binary release yet. Cargo-dist is configured to produce archives for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`, each containing `lsg` or `lsg.exe`, license text, and notices.

When a release is published, download the archive and its SHA-256 checksum from the same GitHub Release. Compare the archive's hash with `shasum -a 256` on macOS, `sha256sum` on Linux, or `Get-FileHash -Algorithm SHA256` in PowerShell before extracting it. Keep license and notice files with redistributed binaries. See [releases](releases.md) for source and build provenance.
