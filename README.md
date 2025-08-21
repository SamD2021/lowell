# lowell

OCI-native tools for building hermetic, host-agnostic boot artifacts (UKIs and initramfs).

## Motivation

Bootable containers and Unified Kernel Images (UKIs) package the OS user space and the boot chain (kernel, initramfs, and command line), respectively, as portable artifacts. Image-based systems typically build the initramfs with `dracut`, which inspects the build host; that coupling can work against sealed, reproducible builds. `lowell`, a modern, cloud-native alternative to `dracut`, aims to produce hermetic, OCI-pinned boot artifacts that are easy to audit (SBOM-friendly), distribute via registries, and can optionally be signed/sealed.

### Why “Lowell”?

Following the tradition of naming software after Massachusetts towns (e.g., Dracut, Wayland, Weston), the project is named after Lowell, Dracut’s younger, neighboring mill city, which continues to modernize and is home to a vibrant community of engineers and university students active in free and open-source software (FOSS).


## Status

Still under early active development:

* **Works today**

  * CLI: `lowell uki inspect /path/to/vmlinuz.efi`
  * Flags: `--format human|json|json-pretty`, `--verbose`, global `--log-level {error|warn|info|debug|trace}`
  * Reports:
    * `arch`, `pe32_plus`
    * Signature presence and `cert_count`
    * Kernel `cmdline`
    * `os-release` fields
    * `.linux` and `.initrd` offsets, sizes, SHA-256
    * initrd compression detection (gzip/xz/zstd/uncompressed) and cpio format (newc)

* **Planned next**

  * `lowell uki inject` — modify initramfs and rebuild a UKI
  * `lowell uki build` — hermetic initramfs + UKI, using OCI-pinned inputs where it helps

## Documentation

**Build and run**

```bash
# build from source
cargo build -p lowell-cli --release

# inspect a UKI
target/release/lowell inspect uki --file /path/to/vmlinuz.efi

# useful flags
--format json
--verbose
--log-level debug
```

**JSON example**

```json
{
  "arch": "aarch64",
  "pe32_plus": true,
  "has_signature": false,
  "cert_count": 0,
  "cmdline": "console=tty0 console=ttyS0",
  "os_release": {
    "name": "Fedora Linux 41 (Forty One)",
    "id": "fedora",
    "version_id": "41"
  },
  "linux": {
    "offset": 66560,
    "size": 15843840,
    "sha256": "2daad44f201454a9e4578ee879c4afe314162d05902564254693cb6824ef1aa7"
  },
  "initrd": {
    "offset": 15910400,
    "size": 41312768,
    "sha256": "d96c7a6ebd5376476114b66a9be10a2e6f7c57898e92e15bd53ae6f8f5e976b0",
    "compression": "xz"
  }
}
```

## Versioning

* Pre-1.0: rapid iteration; breaking changes may occur.
* 1.0 and later: Semantic Versioning.

## Community discussion

GitHub issues/discussions will be the main source for now, but we’re open to any suggestions for better communication.

## Contributing

Thanks for considering a contribution! Bug reports, docs, tests, and features are all welcome.

**Before you start**

* Search existing issues/discussions to avoid duplicates.
* For larger changes, open an issue first to align on scope.

**Dev setup**

* Rust: stable toolchain (`rustup default stable`)
* Recommended: `just` for common tasks

**Common tasks**

```bash
# build
just build            # or: cargo build --release

# run the CLI
target/release/lowell uki inspect /path/to/vmlinuz.efi

# format, lint, test (pre-PR checklist)
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

**Style & guidelines**

* Use `tracing` for logs and prefer structured logs over `println!`.
* Favor `anyhow`/`thiserror` for error handling; avoid `unwrap()` in library code.
* Keep commits focused; Conventional Commits are appreciated but not required.
* Please remember to add the DCO `Signed-off-by` line to the end of your commit messages.

**Submitting a PR**

1. Make sure `fmt`, `clippy`, and tests pass.
2. Add/adjust tests when changing behavior.
3. Update docs/README flags or examples if needed.
4. Fill in a short rationale in the PR description and link any related issues.

**Security**

If you believe you’ve found a vulnerability, please open a private GitHub security advisory (preferred) or contact the maintainers directly. Please do not open a public issue for security reports.

## License

Licensed under either of the following options at your choice:

* **Apache License, Version 2.0** (`LICENSE-APACHE` or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* **MIT license** (`LICENSE-MIT` or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))


Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual-licensed as above, without any additional terms or conditions.

*Add SPDX headers to new source files (recommended):*

```text
// SPDX-License-Identifier: Apache-2.0 OR MIT
```

