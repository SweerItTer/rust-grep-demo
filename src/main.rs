#[path = "args/args.rs"]
mod args;
use args::Args;

fn main() {
    let args = Args::parse();

    println!("pattern = {}", args.pattern);
}
