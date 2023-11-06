use std::{ffi::OsStr, path::Path};

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;

pub fn upload_file(file: &str) -> Result<()> {
    let path = Path::new(file);

    if !path.exists() {
        return Err(anyhow!("Path does not exist"));
    }

    let filename = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or(anyhow!("Path is not a file"))?;

    let data = std::fs::read(path)?;

    Client::new()
        .put(format!(
            "https://10.8.0.1/remote.php/dav/files/daxcess/Feestje/{filename}"
        ))
        .header("Authorization", "Basic ZFF3NHc5V2dYY1E6ZFF3NHc5V2dYY1E=")
        .header("Content-Type", "application/octet-stream")
        .body(data)
        .send()?;

    Ok(())
}
