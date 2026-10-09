#[path = "args/args.rs"]
mod args;
use args::Args;

fn main() {
    let args = Args::parse();

    match args {
        Ok(a) => println!("pattern = {}", a.pattern),
        Err(e) => {
            eprint!("错误: {}", e);
            std::process::exit(1);
        }    
    }   
}
