use std::io::{self, Write};
#[path = "args/args.rs"]
mod args;

mod grep;

use std::path::PathBuf;

use args::Args;

use crate::grep::MyGrep;

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let pattern = match args {
        Ok(a) => a.pattern,
        Err(e) => {
            eprint!("错误: {}", e);
            std::process::exit(1);
        }    
    };
    println!("pattern: {}", pattern);
    
    let obj:MyGrep = MyGrep::new(); 
    let running_path: &PathBuf = obj.get_current_running_path();
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();

    obj.list_all_dir(&running_path, &mut dirs, &mut files)?;

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    for file in files{
        if writeln!(out, "{}", file.display()).is_err() {
            break;   // 写失败就停，不 panic
        }    
    }
    let _ = out.flush();
    Ok(())
}
