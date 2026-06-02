use std::path::PathBuf;

const REGISTRY_URL: &str = "https://packages.ys-lang.org";

fn ys_home() -> PathBuf {
    let base = std::env::var("YS_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs_fallback().join(".ys")
        });
    base
}

fn dirs_fallback() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        return PathBuf::from(home);
    }
    PathBuf::from(".")
}

fn packages_dir() -> PathBuf {
    ys_home().join("packages")
}

fn manifest_path() -> PathBuf {
    ys_home().join("manifest.toml")
}

fn load_manifest() -> toml::Value {
    let path = manifest_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        content.parse::<toml::Value>().unwrap_or(toml::Value::Table(toml::Table::new()))
    } else {
        toml::Value::Table(toml::Table::new())
    }
}

fn save_manifest(manifest: &toml::Value) {
    let path = manifest_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, manifest.to_string()).ok();
}

fn download(url: &str, dest: &std::path::Path) -> Result<(), String> {
    // Try curl first, then PowerShell
    if let Ok(output) = std::process::Command::new("curl")
        .args(["-sSL", "-o"])
        .arg(dest)
        .arg(url)
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }

    let ps_script = format!(
        "Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
        url.replace('\'', "''"),
        dest.to_string_lossy().replace('\'', "''")
    );
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output()
        .map_err(|e| format!("Failed to run download command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Download failed: {}", stderr.trim()))
    }
}

pub fn install(name: &str) -> Result<(), String> {
    let dir = packages_dir().join(name);
    if dir.exists() {
        return Err(format!("Package '{}' is already installed", name));
    }

    let url = format!("{}/{}/latest/download", REGISTRY_URL, name);
    eprintln!("Downloading {}...", name);

    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create package dir: {}", e))?;

    let archive_path = dir.join("package.tar.gz");
    download(&url, &archive_path)?;

    if archive_path.metadata().map(|m| m.len()).unwrap_or(0) == 0 {
        std::fs::remove_dir_all(&dir).ok();
        return Err(format!("Package '{}' not found at {}", name, url));
    }

    let file = std::fs::File::open(&archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(&dir).map_err(|e| format!("Failed to extract package: {}", e))?;

    std::fs::remove_file(&archive_path).ok();

    let mut manifest = load_manifest();
    if let toml::Value::Table(ref mut table) = manifest {
        table.insert(name.to_string(), toml::Value::String(dir.to_string_lossy().into()));
    }
    save_manifest(&manifest);

    eprintln!("Installed package '{}'", name);
    Ok(())
}

pub fn remove(name: &str) -> Result<(), String> {
    let dir = packages_dir().join(name);
    if !dir.exists() {
        return Err(format!("Package '{}' is not installed", name));
    }

    std::fs::remove_dir_all(&dir).map_err(|e| format!("Failed to remove package: {}", e))?;

    let mut manifest = load_manifest();
    if let toml::Value::Table(ref mut table) = manifest {
        table.remove(name);
    }
    save_manifest(&manifest);

    eprintln!("Removed package '{}'", name);
    Ok(())
}

pub fn publish() -> Result<(), String> {
    Err("Publishing is not yet implemented. Use the web UI at https://packages.ys-lang.org".into())
}

pub fn list() -> Result<(), String> {
    let manifest = load_manifest();
    if let toml::Value::Table(table) = &manifest {
        if table.is_empty() {
            eprintln!("No packages installed");
        } else {
            eprintln!("Installed packages:");
            for name in table.keys() {
                eprintln!("  {}", name);
            }
        }
    }
    Ok(())
}
