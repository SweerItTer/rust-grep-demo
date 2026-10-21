use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// 关键词
    pub pattern: String,
    /// 目标路径
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// 递归
    #[arg(short = 'r', long)]
    pub recursive: bool,
    /// 追溯 symlink
    #[arg(short = 'L', long = "follow")]
    pub follow: bool,
}
