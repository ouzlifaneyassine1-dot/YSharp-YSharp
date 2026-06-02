fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: yo <command>");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  install <package>    Install a Y# package");
        eprintln!("  remove <package>     Remove an installed package");
        eprintln!("  publish              Publish the current project (not yet available)");
        eprintln!("  list                 List installed packages");
        std::process::exit(1);
    }

    let result = match args[1].as_str() {
        "install" if args.len() >= 3 => oys_compiler::pkg::install(&args[2]),
        "remove" if args.len() >= 3 => oys_compiler::pkg::remove(&args[2]),
        "publish" => oys_compiler::pkg::publish(),
        "list" => oys_compiler::pkg::list(),
        "install" => {
            eprintln!("Usage: yo install <package>");
            return;
        }
        "remove" => {
            eprintln!("Usage: yo remove <package>");
            return;
        }
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
