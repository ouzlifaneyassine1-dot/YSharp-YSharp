// Y# v8.0.0 Installer — Windows self-extracting installer
// Supports: --silent, --dir=<path>, --no-path, --uninstall, --help

const DIST_ZIP: &[u8] = include_bytes!("../../../dist/ys-v8.0.0-windows-x64.zip");

use std::path::{Path, PathBuf};
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let prog = args.first().map(|s| s.as_str()).unwrap_or("install-ys");

    let opts = match parse_args(&args[1..]) {
        Ok(o) => o,
        Err(msg) => {
            if msg.is_empty() {
                print_help(prog);
                return;
            }
            eprintln!("error: {}", msg);
            print_help(prog);
            std::process::exit(1);
        }
    };

    match opts.mode {
        Mode::Help => print_help(prog),
        Mode::Uninstall => cmd_uninstall(&opts),
        Mode::Install => cmd_install(&opts),
    }
}

#[derive(Default)]
struct Opts {
    mode: Mode,
    dir: PathBuf,
    silent: bool,
    no_path: bool,
}

enum Mode {
    Help,
    Install,
    Uninstall,
}

impl Default for Mode {
    fn default() -> Self { Mode::Install }
}

fn parse_args(args: &[String]) -> Result<Opts, String> {
    let mut opts = Opts::default();

    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "/?" | "-?" | "--help" | "/help" | "-h" => opts.mode = Mode::Help,
            "/S" | "/silent" | "--silent" | "-s" => opts.silent = true,
            "/NOPATH" | "/no-path" | "--no-path" | "-n" => opts.no_path = true,
            "/UNINSTALL" | "/uninstall" | "--uninstall" | "-u" => opts.mode = Mode::Uninstall,
            d if d.starts_with("/D=") || d.starts_with("--dir=") => {
                let val = d.split_once('=').map(|(_, v)| v).unwrap_or("");
                opts.dir = PathBuf::from(val);
            }
            d if d.starts_with("-d=") || d.starts_with("--dest=") => {
                let val = d.split_once('=').map(|(_, v)| v).unwrap_or("");
                opts.dir = PathBuf::from(val);
            }
            _ => return Err(format!("unknown argument: {}", a)),
        }
        i += 1;
    }

    if opts.dir.as_os_str().is_empty() {
        opts.dir = default_install_dir();
    }

    Ok(opts)
}

fn print_help(prog: &str) {
    println!("Y# v8.0.0 — Windows Installer");
    println!();
    println!("Usage: {} [options]", prog);
    println!();
    println!("Options:");
    println!("  /D=<path>    Install to <path> (default: %LOCALAPPDATA%\\YS-Lang)");
    println!("  /S           Silent mode (no prompts)");
    println!("  /NOPATH      Do not add to PATH");
    println!("  /UNINSTALL   Remove Y# from the system");
    println!("  /?           Show this help");
    println!();
    println!("Examples:");
    println!("  {}                        Interactive install", prog);
    println!("  {} /D=C:\\YS /S            Silent install to C:\\YS", prog);
    println!("  {} /UNINSTALL              Remove Y#", prog);
}

fn default_install_dir() -> PathBuf {
    std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\Program Files"))
        .join("YS-Lang")
}

// ---- Install ----

fn cmd_install(opts: &Opts) {
    let target = &opts.dir;

    if !opts.silent {
        println!("Y# v8.0.0 Installer");
        println!("{}", "=" .repeat(40));
        println!("  Install dir: {}", target.display());
        if opts.no_path {
            println!("  PATH:        skipped");
        } else {
            println!("  PATH:        add user PATH");
        }
        print!("  Proceed? [Y/n] ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_lowercase();
        if input == "n" || input == "no" {
            println!("Installation cancelled.");
            return;
        }
    }

    // Create directories
    let bin_dir = target.join("bin");
    let std_dir = target.join("std");
    let examples_dir = target.join("examples");

    if let Err(e) = create_dirs(&[target, &bin_dir, &std_dir, &examples_dir]) {
        eprintln!("error: failed to create directories: {}", e);
        if !opts.silent {
            eprintln!("  Try running as administrator or choose a different path.");
        }
        std::process::exit(1);
    }

    // Extract zip
    eprint!("  Extracting files...");
    io::stdout().flush().ok();
    match extract_zip(DIST_ZIP, target) {
        Ok(count) => eprintln!(" {} files extracted", count),
        Err(e) => {
            eprintln!(" failed: {}", e);
            std::process::exit(1);
        }
    }

    // Add to PATH
    if !opts.no_path {
        eprint!("  Adding to PATH...");
        io::stdout().flush().ok();
        add_to_path(&bin_dir);
        eprintln!(" done");
    }

    // Create uninstall registry key
    eprint!("  Creating uninstall entry...");
    io::stdout().flush().ok();
    create_uninstall_reg(target);
    eprintln!(" done");

    println!();
    println!("Y# v8.0.0 installed successfully!");
    println!();
    println!("  Location: {}", target.display());
    if !opts.no_path {
        println!("  Restart your terminal, then type: oys --help");
    } else {
        println!("  Add {} to your PATH manually.", bin_dir.display());
    }
}

fn create_dirs(dirs: &[&Path]) -> io::Result<()> {
    for d in dirs {
        std::fs::create_dir_all(d)?;
    }
    Ok(())
}

fn extract_zip(data: &[u8], out_dir: &Path) -> Result<usize, String> {
    let reader = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| format!("invalid zip: {}", e))?;

    let mut count = 0usize;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("zip entry {}: {}", i, e))?;
        let name = file.name().to_string();

        // Skip directories (they end with /)
        if name.ends_with('/') {
            continue;
        }

        let out_path = out_dir.join(&name);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {}", parent.display(), e))?;
        }

        let mut out_file = std::fs::File::create(&out_path)
            .map_err(|e| format!("create {}: {}", out_path.display(), e))?;

        std::io::copy(&mut file, &mut out_file)
            .map_err(|e| format!("write {}: {}", out_path.display(), e))?;

        count += 1;
    }

    Ok(count)
}

fn add_to_path(dir: &Path) {
    let dir_str = dir.to_string_lossy().replace('/', "\\");

    // Use PowerShell to modify user PATH (avoids duplicate entries)
    let ps = format!(
        "[Environment]::SetEnvironmentVariable('Path', \
         (@([Environment]::GetEnvironmentVariable('Path','User') -split ';' \
            | Where-Object {{ $_ -and $_ -ne '{}' }}) \
          + @('{}')) -join ';', \
         'User')",
        dir_str, dir_str
    );
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
        .output();
}

fn create_uninstall_reg(install_dir: &Path) {
    let dir_str = install_dir.to_string_lossy().replace('/', "\\");
    let bin_exe = format!("{}\\bin\\oys.exe", dir_str);

    let ps = format!(
        r#"
$path = 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\YS-Lang'
if (-not (Test-Path $path)) {{
    New-Item -Path $path -Force | Out-Null
}}
New-ItemProperty -Path $path -Name DisplayName -Value 'Y# Programming Language v8.0.0' -Force | Out-Null
New-ItemProperty -Path $path -Name DisplayVersion -Value '8.0.0' -Force | Out-Null
New-ItemProperty -Path $path -Name Publisher -Value 'Y# Language Team' -Force | Out-Null
New-ItemProperty -Path $path -Name InstallLocation -Value '{}' -Force | Out-Null
New-ItemProperty -Path $path -Name DisplayIcon -Value '{}' -Force | Out-Null
New-ItemProperty -Path $path -Name UninstallString -Value '"{}" /UNINSTALL' -Force | Out-Null
New-ItemProperty -Path $path -Name QuietUninstallString -Value '"{}" /S /UNINSTALL' -Force | Out-Null
"#,
        dir_str, bin_exe, bin_exe, bin_exe
    );

    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
        .output();
}

// ---- Uninstall ----

fn cmd_uninstall(opts: &Opts) {
    let target = &opts.dir;

    if !opts.silent {
        println!("Y# v8.0.0 Uninstaller");
        println!("{}", "=" .repeat(40));
        println!("  Remove from: {}", target.display());
        print!("  Proceed? [y/N] ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            println!("Uninstall cancelled.");
            return;
        }
    }

    // Remove files
    eprint!("  Removing files...");
    io::stdout().flush().ok();
    let _ = std::fs::remove_dir_all(target);
    eprintln!(" done");

    // Remove from PATH
    eprint!("  Cleaning PATH...");
    io::stdout().flush().ok();
    remove_from_path(target);
    eprintln!(" done");

    // Remove uninstall registry
    eprint!("  Removing registry entry...");
    io::stdout().flush().ok();
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command",
            "Remove-Item -Path 'HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\YS-Lang' -Force -ErrorAction SilentlyContinue"])
        .output();
    eprintln!(" done");

    println!();
    println!("Y# v8.0.0 has been removed.");
}

fn remove_from_path(dir: &Path) {
    let dir_str = dir.to_string_lossy().replace('/', "\\");

    let ps = format!(
        "[Environment]::SetEnvironmentVariable('Path', \
         (@([Environment]::GetEnvironmentVariable('Path','User') -split ';' \
            | Where-Object {{ $_ -and $_ -ne '{}' -and $_ -ne '{}\\bin' }}) \
          -join ';', \
         'User')",
        dir_str, dir_str
    );
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
        .output();
}
