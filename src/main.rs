// use std::io::{self, Write};
use std::path::PathBuf;
use clap::Parser;

mod grep;
mod cli;

use grep::MyGrep;
use cli::Cli;

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();    
    let obj:MyGrep = MyGrep::new(cli); 

    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();

    let _ = obj.list_all_files(&mut dirs, &mut files)?;

    obj.grep(&files);

    // let stdout = io::stdout();
    // let mut out = io::BufWriter::new(stdout.lock());
    // for file in files{
    //     if writeln!(out, "{}", file.display()).is_err() {
    //         break;   // 写失败就停，不 panic
    //     }    
    // }
    // _ = out.flush();
    Ok(())
}
