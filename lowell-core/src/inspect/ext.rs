//! Extension helpers for working with PE sections in the `uki` layer.
//!
//! Purpose: keep `formats::pe::PeFile` **format-agnostic** and small,
//! while giving `uki::inspect` an ergonomic way to fetch a section’s
//! bytes and file-location in one call.

use crate::formats::pe::PeFile;
use crate::inspect::uki::SectionInfo;
use anyhow::Result; // adjust path if you moved SectionInfo

// ---- Sealed extension trait (prevents external impls) ----
mod sealed {
    pub trait Sealed {}
    impl Sealed for crate::formats::pe::PeFile {}
}

/// Section lookup conveniences used by `uki::inspect`.
///
/// Intentionally **mechanical only**: offsets/sizes and borrowed bytes.
/// Policy (e.g., hashing, compression detection) stays outside this trait.
pub(crate) trait SectionLookupExt: sealed::Sealed {
    /// Borrow section bytes and get their (offset, size) in the file.
    ///
    /// Returns:
    /// - `Ok((&[u8], (offset, size)))` on success
    /// - `Err(..)` if the section is missing or coordinates are invalid
    fn section_bytes_and_location(&self, name: &str) -> Result<(&[u8], (usize, usize))>;

    /// Like [`section_bytes_and_location`], but also builds a `SectionInfo`
    /// (offset/size filled; caller can attach SHA or other metadata).
    fn section_info_and_bytes(&self, name: &str) -> Result<(SectionInfo, &[u8])>;
}

impl SectionLookupExt for PeFile {
    fn section_bytes_and_location(&self, name: &str) -> Result<(&[u8], (usize, usize))> {
        let bytes = self
            .section_bytes(name)?
            .ok_or_else(|| anyhow::anyhow!("no {name} section found in the UKI"))?;
        let (offset, size) = self
            .section_info(name)?
            .ok_or_else(|| anyhow::anyhow!("Couldn't get {name} offset and size in the UKI"))?;
        Ok((bytes, (offset, size)))
    }

    fn section_info_and_bytes(&self, name: &str) -> Result<(SectionInfo, &[u8])> {
        let (bytes, (offset, size)) = self.section_bytes_and_location(name)?;
        let sha256 = String::new(); // Placeholder, caller can fill this in
        Ok((
            SectionInfo {
                offset,
                size,
                sha256,
            },
            bytes,
        ))
    }
}
