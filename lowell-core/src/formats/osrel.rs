// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::formats::pe::PeFile;
use anyhow::Result;
use rs_release::parse_os_release_str;

#[derive(Debug, serde::Serialize)]
pub struct OsRelease {
    pub name: Option<String>,
    pub id: Option<String>,
    pub version_id: Option<String>,
}

pub fn read_os_release_from_str(text: &str) -> Result<Option<OsRelease>> {
    let m = parse_os_release_str(text)?;
    let name = m
        .get("PRETTY_NAME")
        .cloned()
        .or_else(|| m.get("NAME").cloned());
    let id = m.get("ID").cloned();
    let version_id = m.get("VERSION_ID").cloned();
    Ok(Some(OsRelease {
        name,
        id,
        version_id,
    }))
}

pub fn read_os_release(pef: &PeFile) -> Result<Option<OsRelease>> {
    let Some(text) = pef.read_text(".osrel")? else {
        return Ok(None);
    };
    let m = parse_os_release_str(&text)?;
    let name = m
        .get("PRETTY_NAME")
        .cloned()
        .or_else(|| m.get("NAME").cloned());
    let id = m.get("ID").cloned();
    let version_id = m.get("VERSION_ID").cloned();
    Ok(Some(OsRelease {
        name,
        id,
        version_id,
    }))
}
