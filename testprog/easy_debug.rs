fn main() {
    let s = r#"fn greet $name
    print Hello $name

fn main
    var x = 42
    if x > 10
        println x is large
    loop i from 1 to 3
        print 'Value: '
        println i
    greet World
    return 0"#;
    println!("=== INPUT ===");
    println!("{}", s);
    println!("\n=== TRANSPILED ===");
    println!("{}", ys_transpiler::transpile(s));
}
