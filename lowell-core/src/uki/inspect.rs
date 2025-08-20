// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::formats::initramfs::{detect, Compression};
use crate::formats::osrel::{read_os_release, OsRelease};
use crate::formats::pe::PeFile;
use crate::uki::ext::SectionLookupExt;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::Instant;
use tracing::{debug, debug_span};

#[derive(Debug)]
pub struct InspectOptions {
    /// Path to the UKI to inspect
    pub file: PathBuf,
}

#[derive(Debug, serde::Serialize)]
pub struct Report {
    pub arch: String,        // e.g. "aarch64"
    pub pe32_plus: bool,     // PE32+?
    pub has_signature: bool, // Authenticode present?
    pub cert_count: usize,   // number of certs (if has_signature)
    pub cmdline: String,
    pub os_release: Option<OsRelease>,
    pub linux: SectionInfo,
    pub initrd: InitrdInfo,
}

#[derive(Debug, serde::Serialize)]
pub struct SectionInfo {
    pub offset: usize,
    pub size: usize,
    pub sha256: String,
}

#[derive(Debug, serde::Serialize)]
pub struct InitrdInfo {
    #[serde(flatten)]
    pub section: SectionInfo,
    pub compression: Compression,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries_estimate: Option<usize>,
}

pub fn inspect(InspectOptions { file: uki }: InspectOptions) -> Result<Report> {
    // Parent span
    let _inspect_span = debug_span!("inspect", path = %uki.display()).entered();

    // 1) File read
    let t0 = Instant::now();
    let bytes = std::fs::read(&uki).with_context(|| format!("read {}", uki.display()))?;
    debug!(
        len = bytes.len(),
        elapsed_ms = t0.elapsed().as_millis(),
        "read_file"
    );

    // 2) Parse PE + arch
    let t = Instant::now();
    let pef = PeFile::from_bytes(bytes)?;
    let (arch, pe32p) = pef.arch_summary()?;
    debug!(
        arch,
        pe32_plus = pe32p,
        elapsed_ms = t.elapsed().as_millis(),
        "parse_pe"
    );

    // 3) cmdline + os-release
    let t = Instant::now();
    let cmdline = pef
        .read_text(".cmdline")?
        .unwrap_or_default()
        .trim()
        .to_string();
    let os_release: Option<OsRelease> = read_os_release(&pef)?;
    debug!(elapsed_ms = t.elapsed().as_millis(), "metadata");

    // 4) .linux: fetch + hash
    let (mut linux_info, linux_bytes) = pef.section_info_and_bytes(".linux")?;
    let t = Instant::now();
    linux_info.sha256 = format!("{:x}", Sha256::digest(linux_bytes));
    debug!(
        size = linux_bytes.len(),
        elapsed_ms = t.elapsed().as_millis(),
        "sha256_linux"
    );

    // 5) .initrd: fetch + hash + detect
    let (mut initrd_info, initrd_bytes) = pef.section_info_and_bytes(".initrd")?;
    let t = Instant::now();
    initrd_info.sha256 = format!("{:x}", Sha256::digest(initrd_bytes));
    let detect_t = Instant::now();
    let compression = detect(initrd_bytes);
    debug!(
        size = initrd_bytes.len(),
        hash_ms = t.elapsed().as_millis(),
        detect_ms = detect_t.elapsed().as_millis(),
        "initrd_hash_and_detect"
    );

    // 6) Certificates (do once; reuse for has_signature + count)
    let t = Instant::now();
    let cert_count = pef.certificate_blobs()?.len();
    let has_signature = cert_count > 0;
    debug!(
        cert_count,
        elapsed_ms = t.elapsed().as_millis(),
        "certificates"
    );

    let initrd = InitrdInfo {
        section: initrd_info,
        compression,
        entries_estimate: None,
    };

    Ok(Report {
        arch: arch.to_string(),
        pe32_plus: pe32p,
        has_signature,
        cert_count,
        cmdline,
        os_release,
        linux: linux_info,
        initrd,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::initramfs::{detect, Compression};
    use crate::formats::osrel::read_os_release_from_str;

    // ---- initramfs detection (pure unit tests) ----

    #[test]
    fn initramfs_detects_gzip_xz_zstd_newc_unknown() {
        // gzip magic: 1F 8B
        assert!(matches!(
            detect(&[0x1F, 0x8B, 0x08, 0x00]),
            Compression::Gzip
        ));

        // xz magic: FD 37 7A 58 5A 00
        assert!(matches!(
            detect(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]),
            Compression::Xz
        ));

        // zstd magic: 28 B5 2F FD
        assert!(matches!(
            detect(&[0x28, 0xB5, 0x2F, 0xFD]),
            Compression::Zstd
        ));

        // newc cpio (uncompressed): ASCII "070701" at start
        assert!(matches!(detect(b"070701..."), Compression::Uncompressed));

        // unknown / too short
        assert!(matches!(detect(&[]), Compression::Unknown));
        assert!(matches!(detect(&[0x00, 0x01]), Compression::Unknown));
    }

    // ---- os-release parsing (pure unit tests) ----

    #[test]
    fn osrelease_parses_fedora41_and_prefers_pretty_name() {
        // Realistic snippet (trimmed)
        let fedora = r#"NAME="Fedora Linux"
VERSION="41 (Forty One)"
ID=fedora
VERSION_ID=41
PRETTY_NAME="Fedora Linux 41 (Forty One)"
"#;

        let os = read_os_release_from_str(fedora)
            .expect("parse ok")
            .expect("Some(os-release)");

        // PRETTY_NAME takes priority for human display
        assert_eq!(os.name.as_deref(), Some("Fedora Linux 41 (Forty One)"));
        // Stable fields used for tooling/logic
        assert_eq!(os.id.as_deref(), Some("fedora"));
        assert_eq!(os.version_id.as_deref(), Some("41"));
    }

    #[test]
    fn osrelease_falls_back_to_name_when_pretty_missing() {
        let minimal = r#"NAME="MyOS"
ID=myos
VERSION_ID="1.2.3"
"#;
        let os = read_os_release_from_str(minimal)
            .expect("parse ok")
            .expect("Some(os-release)");

        // PRETTY_NAME absent → fall back to NAME
        assert_eq!(os.name.as_deref(), Some("MyOS"));
        assert_eq!(os.id.as_deref(), Some("myos"));
        assert_eq!(os.version_id.as_deref(), Some("1.2.3"));
    }

    // ---- optional integration smoke test (ignored by default) ----
    //
    // Run with:  UKI_PATH=/full/path/to/vmlinuz.efi  cargo test -- --ignored
    // or:        cargo test inspect_real_uki_smoke -- --ignored
    #[test]
    #[ignore = "requires UKI_PATH"]
    fn inspect_real_uki_smoke() {
        let uki_path = std::env::var("UKI_PATH").expect("set UKI_PATH to a real UKI");
        let report = inspect(InspectOptions {
            file: uki_path.into(),
        })
        .expect("inspect report");

        // Sanity checks that don’t depend on a specific distro
        assert!(!report.arch.is_empty());
        assert!(report.linux.size > 0);
        assert!(report.initrd.section.size > 0);
        assert_ne!(report.initrd.compression, Compression::Unknown);

        // sha256 fields should be 64 hex chars
        assert_eq!(report.linux.sha256.len(), 64);
        assert!(report.linux.sha256.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(report.initrd.section.sha256.len(), 64);
        assert!(report
            .initrd
            .section
            .sha256
            .chars()
            .all(|c| c.is_ascii_hexdigit()));

        // If the UKI embeds .cmdline, it should be trimmed
        assert_eq!(report.cmdline, report.cmdline.trim());
    }
}
