// SPDX-License-Identifier: MIT OR Apache-2.0
//! PE/COFF helpers for Unified Kernel Images (UKI)
//!
//! Read-only introspection of PE/EFI images (UKIs) with small, ergonomic helpers.
//! We **own** the file bytes and parse with `goblin` on demand; methods then
//! slice into `self.data`, so the public API stays lifetime-free.
//!
//! ### UKI sections you’ll typically care about
//! - `.linux`   — kernel image (Image/bzImage)
//! - `.initrd`  — initramfs blob (often gzip/xz/zstd; can be concatenated cpio)
//! - `.cmdline` — kernel command line (ASCII/UTF-8, NUL-padded)
//! - `.osrel`   — os-release contents (text)
//! - `.sbat`    — SBAT CSV (shim)
//! - `.sdmagic` — systemd-stub marker
//!
//! ### Certificates / Authenticode (Secure Boot)
//! - Presence is indicated by the **Security** data directory (index 4).
//! - In PE/COFF, **only** this directory uses a **file offset** (not an RVA).
//! - `goblin` already parses certificates into `pe.certificates`, so you can
//!   inspect counts, lengths, types, and get the raw blobs directly.
//! - We DO NOT verify signatures here; presence ≠ validity.

use anyhow::{Context, Result};
use goblin::pe::{options::ParseOptions, PE};
use std::path::Path;

/// An owning wrapper around a PE/EFI image (UKI).
///
/// Holds the file bytes, parses with goblin on demand, and returns
/// borrowed slices tied to `&self`. This avoids lifetimes in the
/// public API and side-steps self-referential types.
#[derive(Debug)]
pub struct PeFile {
    /// Entire image bytes (owned).
    data: Box<[u8]>,
}

impl PeFile {
    /// Read a PE/EFI image from disk and own its bytes.
    pub fn from_path(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
        Ok(Self {
            data: bytes.into_boxed_slice(),
        })
    }

    /// Construct from a caller-provided byte vector.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            data: bytes.into_boxed_slice(),
        })
    }

    /// Access the full image buffer (read-only).
    pub fn image(&self) -> &[u8] {
        &self.data
    }

    // ---------- Parsing & basics ----------

    /// Parse PE headers using options appropriate for on-disk binaries.
    /// (We explicitly enable attribute certificate parsing.)
    fn parse_pe(&self) -> Result<PE<'_>> {
        let mut opts = ParseOptions::default();
        opts.parse_attribute_certificates = true; // ensure certs are parsed
        PE::parse_with_opts(&self.data, &opts).context("not a valid PE/EFI image")
    }

    /// Return a human-oriented architecture label and PE32+ flag.
    ///
    /// Common results:
    /// - `("x86_64", true)` for amd64 UKIs
    /// - `("aarch64", true)` for ARM64 UKIs
    /// - `("i386", false)` for 32-bit x86
    pub fn arch_summary(&self) -> Result<(&'static str, bool)> {
        use goblin::pe::header::*;
        let pe = self.parse_pe()?;
        let arch = match pe.header.coff_header.machine {
            COFF_MACHINE_X86_64 => "x86_64",
            COFF_MACHINE_ARM64 => "aarch64",
            COFF_MACHINE_ARM => "arm",
            COFF_MACHINE_X86 => "i386",
            _ => "unknown",
        };
        Ok((arch, pe.is_64))
    }

    // ---------- Sections ----------
    //
    /// Offset and file size of a named section, if it exists.
    /// (file_offset, file_size)
    pub fn section_info(&self, name: &str) -> Result<Option<(usize, usize)>> {
        let pe = self.parse_pe()?;
        let sec = pe.sections.iter().find(|t| t.name().ok() == Some(name));
        if let Some(s) = sec {
            Ok(Some((
                s.pointer_to_raw_data as usize,
                s.size_of_raw_data as usize,
            )))
        } else {
            Ok(None)
        }
    }

    /// Borrow raw bytes of a named section (e.g., ".initrd", ".linux", ".cmdline").
    ///
    /// Returns `Ok(None)` if the section is missing or coordinates are invalid.
    pub fn section_bytes(&self, name: &str) -> Result<Option<&[u8]>> {
        let pe = self.parse_pe()?;
        let sec = pe.sections.iter().find(|t| t.name().ok() == Some(name));
        if let Some(s) = sec {
            let off = usize::try_from(s.pointer_to_raw_data).ok();
            let sz = usize::try_from(s.size_of_raw_data).ok();
            if let (Some(off), Some(sz)) = (off, sz) {
                let end = off.checked_add(sz);
                return Ok(end.and_then(|e| self.data.get(off..e)));
            }
        }
        Ok(None)
    }

    /// Read a section as text (trim at first NUL). Ideal for `.cmdline` / `.osrel`.
    pub fn read_text(&self, name: &str) -> Result<Option<String>> {
        Ok(self.section_bytes(name)?.map(|b| {
            let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
            String::from_utf8_lossy(&b[..end]).to_string()
        }))
    }

    // ---------- Certificates (Authenticode) ----------

    /// True if the image contains one or more Attribute Certificates.
    ///
    /// Presence indicates a **Certificate Table** exists; it does **not** mean
    /// the signature is valid. Modifying sections (e.g., `.initrd`) will typically
    /// invalidate verification in Secure Boot.
    pub fn is_signed(&self) -> Result<bool> {
        let pe = self.parse_pe()?;
        Ok(!pe.certificates.is_empty())
    }

    /// Lightweight metadata for each attribute certificate: (length, revision, type).
    ///
    /// `revision` and `typ` come from the WIN_CERTIFICATE header. The blob itself is
    /// usually PKCS#7 SignedData (`typ` 0x0002).
    pub fn certificate_metadata(&self) -> Result<Vec<(u32, u16, u16)>> {
        let pe = self.parse_pe()?;
        Ok(pe
            .certificates
            .iter()
            .map(|c| (c.length, c.revision as u16, c.certificate_type as u16))
            .collect())
    }

    /// The raw certificate blobs (`&[u8]`) for each attribute certificate.
    pub fn certificate_blobs(&self) -> Result<Vec<&[u8]>> {
        let pe = self.parse_pe()?;
        Ok(pe.certificates.iter().map(|c| c.certificate).collect())
    }
}
