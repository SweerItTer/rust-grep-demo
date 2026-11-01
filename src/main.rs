use std::io::IsTerminal;
use std::path::PathBuf;
use clap::Parser;

mod grep;
mod cli;
mod colorful;
use grep::MyGrep;
use cli::Cli;
use std::sync::LazyLock;

static IS_TERMINAL: LazyLock<bool> = LazyLock::new(|| std::io::stdout().is_terminal());

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();    
    let obj:MyGrep = MyGrep::new(cli); 

    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();

    let _ = obj.list_all_files(&mut dirs, &mut files)?;

    obj.grep(&files);

    Ok(())
}
