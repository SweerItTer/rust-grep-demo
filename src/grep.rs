use std::path::PathBuf;
use memchr::memmem;
use crate::cli::Cli;


pub struct MyGrep {
    cli: Cli,
}

impl MyGrep {

    // 初始化
    #[inline]
    #[must_use]
    pub const fn new(cli:Cli) -> Self {
        MyGrep {
            cli: cli
        }
    }

    pub fn list_all_files(
        &self,
        dirs: &mut Vec<PathBuf>, 
        files: &mut Vec<PathBuf>
    ) -> std::io::Result<()> {
        self.list_all_files_(
            &self.cli.path,
            dirs,
            files            
        )
    }

    /* 列出所有可继续递归的文件夹 */
    fn list_all_files_(
        &self, 
        path: &PathBuf, 
        dirs: &mut Vec<PathBuf>, 
        files: &mut Vec<PathBuf>
    ) -> std::io::Result<()> {  
        // 读取目录, 拿到所有条目
        let entries = match std::fs::read_dir(path) {
            Ok(a) => a,
            Err(e) => {
                // 可能存在权限或其他问题, 需要捕获异常
                eprintln!("{}: {}", path.display(), e);
                return Ok(());
            }
        };
        for entry in entries {
            // 取出 entry.Ok, 并自动上报错误
            let entry = entry?;

            // 获取文件类型
            let file_type = entry.file_type()?; 
            // 获取完整的文件路径
            let full_path: PathBuf = entry.path();

            // 判断文件类型
            if file_type.is_dir() && self.cli.recursive {
                // 递归文件夹
                self.list_all_files_(&full_path, dirs, files)?;
                // TODO: 后续类似 --type d 时才记录
                // 缓存对应的类型
                // dirs.push(full_path);
            } else if file_type.is_symlink() && self.cli.follow {
                // TODO: 跟随软链接
            } else if file_type.is_file() {
                files.push(full_path);
            } 
        }
        Ok(())
    }
}

// struct MatchLine {
//     line_number: usize,   // 行号，usize 更自然
//     text: String,         // 这一行的内容
// }

// /* 遍历所有可读文件，做关键词查找 */
// impl MyGrep {
//     pub fn find_str() -> std::io::Result<()> {
//         let res = Matched {
//             line_num: 0,
//             line_text: ""
//         };
//         Ok(res)
//     }
// }


/* 收集完后把文件路径缓存，再去读取来匹配 */
/* 因为缓存了路径，匹配到后获取行号和当前行的内容 */
/* 一个文件不是匹配到一次就停止，而是一直匹配直到没了 */
/* 不缓存匹配结果了，匹配到直接输出 */
/* 进一步进入可递归的文件夹，重复步骤以获取所有文件和文件夹 */
/* 重复直到无可递归文件夹 */
/* 改成读的时候尝试，读不了就跳过 */