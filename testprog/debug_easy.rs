fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 { eprintln!("usage: debug_easy <file>"); return; }
    let src = std::fs::read_to_string(&args[1]).unwrap();
    let result = oys_compiler::easy::transpile(&src);
    println!("{}", result);
}
