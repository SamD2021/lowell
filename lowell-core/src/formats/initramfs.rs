// SPDX-License-Identifier: MIT OR Apache-2.0
use std::fmt;
#[derive(serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Compression {
    Gzip,
    Xz,
    Zstd,
    Uncompressed,
    Unknown,
}

impl fmt::Display for Compression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Compression::Gzip => "gzip",
            Compression::Xz => "xz",
            Compression::Zstd => "zstd",
            Compression::Uncompressed => "uncompressed",
            Compression::Unknown => "unknown",
        };
        f.write_str(s)
    }
}

#[inline]
pub fn detect(bytes: &[u8]) -> Compression {
    match bytes {
        // gzip: 1F 8B
        [0x1F, 0x8B, ..] => Compression::Gzip,

        // xz: FD 37 7A 58 5A 00
        [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00, ..] => Compression::Xz,

        // zstd: 28 B5 2F FD
        [0x28, 0xB5, 0x2F, 0xFD, ..] => Compression::Zstd,

        // uncompressed cpio (newc): ASCII "070701"
        [b'0', b'7', b'0', b'7', b'0', b'1', ..] => Compression::Uncompressed,

        // anything else
        _ => Compression::Unknown,
    }
}
