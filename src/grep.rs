use std::{io::BufReader, io::BufRead, path::PathBuf};
use memchr::memmem;
use crate::cli::Cli;
use content_inspector::inspect;


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
                if entry.file_name() == ".git" {
                    continue;
                }
                // 递归文件夹
                self.list_all_files_(&full_path, dirs, files)?;
                // TODO: 后续类似 --type d 时才记录
                // 缓存对应的类型
                // dirs.push(full_path);
            } else if file_type.is_symlink() && self.cli.follow { // bool 类型的值, 在形如 `self.cli.follow` 的处理下, 本身值的所有权不变, 在 if 里的是隐式 Copy 后的
                // TODO: 跟随软链接
            } else if file_type.is_file() {
                files.push(full_path);
            } 
        }
        Ok(())
    }
}


/* 遍历所有可读文件，做关键词查找 */
impl MyGrep {
    fn single_file_finder(&self, finder: &memmem::Finder<'_>, file_path: &PathBuf) -> std::io::Result<()> {
        // 打开文件
        let file: std::fs::File = match std::fs::File::open(file_path) {
            Ok(f) => f,
            Err(e) => {
                // 可能存在权限或其他问题, 需要捕获异常
                eprintln!("{}: {}", file_path.display(), e);
                return Ok(())
            }
        };
        /*
         * BufReader<R> 可以提升对同一文件或网络套接字进行少量且重复读取调用的程序的速度
         * 在一次性读取大量数据、仅读取一次或少数几次时，它并无帮助
         * 从内存中已有的数据源（如 Vec<u8>）读取时，它也不具备任何优势
         */
        let mut reader: BufReader<std::fs::File> = BufReader::new(file);
        // 预读 8KB, 不足8KB为原始长度
        let probe = reader.fill_buf()?;
        if inspect(probe).is_binary() {
            // 跳过二进制文件匹配
            return Ok(())
        }

        // 缓存一行的内容(字节)
        let mut line: Vec<u8> = Vec::new();
        // 行号
        let mut index: usize = 0;
        // 匹配目标长度
        let _length: usize = finder.needle().len();

        loop {
            // 清空上一次的结果
            line.clear();
            // 读至末尾
            if reader.read_until(b'\n', &mut line)? == 0 {
                break;
            }
            // 行号+1
            index = index + 1;

            // 匹配关键词
            let mut it = finder.find_iter(&line);
            
            // 无匹配结果
            if !it.any(|_| true) {
                continue;
            }
            // TODO: 用红色标记关键词, 当前已有 &line, it, length
            println!(
                "{}:{}:{}",
                file_path.display(),
                index,
                String::from_utf8_lossy(&line),
            );
            // 匹配完后进入下一轮循环
        }

        Ok(())
    }

    // [T] 表示编译期长度未知的同类型元素序列， &[T] 是对 slice 的引用
    pub fn grep(&self, files: &[PathBuf]) {
        // memmem 只认 字节， 因此同时可以查找二进制内容
        let b_pattern = self.cli.pattern.as_bytes();
        let finder = memmem::Finder::new(&b_pattern);
        
        for file in files {
            let _ = self.single_file_finder(&finder, file);
        }
    }
}


/* 收集完后把文件路径缓存，再去读取来匹配 */
/* 因为缓存了路径，匹配到后获取行号和当前行的内容 */
/* 一个文件不是匹配到一次就停止，而是一直匹配直到没了 */
/* 不缓存匹配结果了，匹配到直接输出 */
/* 进一步进入可递归的文件夹，重复步骤以获取所有文件和文件夹 */
/* 重复直到无可递归文件夹 */
/* 改成读的时候尝试，读不了就跳过 */