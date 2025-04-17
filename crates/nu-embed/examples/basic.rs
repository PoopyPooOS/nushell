use nu_embed::Engine;

fn main() {
    let source = "http get https://api.github.com/repos/nushell/nushell | get license";
    println!("Running: {source}");
    Engine::new(source).eval();
}
