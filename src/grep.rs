use std::{io::BufReader, io::BufRead, path::PathBuf};
use std::sync::LazyLock;
use memchr::memmem;
use crate::cli::Cli;
use content_inspector::inspect;
use crate::colorful::{PrinterAttr, Color, colorful_contact};
use crate::IS_TERMINAL;

/*
 * stdout 指向的文件身份 (st_dev, st_ino)
 * 路径不参与判断: 硬链接/软链接/`./` 等等价写法都会落到同一个 inode
 * 只在 stdout 是普通文件时才有意义 (管道、tty、/dev/null 无需比较)
 */
static STDOUT_ID: LazyLock<Option<(u64, u64)>> = LazyLock::new(|| {
    use std::os::unix::fs::MetadataExt;
    // Linux 用 /proc/self/fd, BSD/macOS 用 /dev/fd
    let probe = if cfg!(target_os = "linux") { "/proc/self/fd/1" } else { "/dev/fd/1" };
    std::fs::metadata(probe)
        .ok()
        .filter(std::fs::Metadata::is_file)
        .map(|m| (m.dev(), m.ino()))
});

/* 该文件是否就是 stdout 本身 */
fn is_stdout(file: &std::fs::File) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (*STDOUT_ID, file.metadata()) {
        (Some((dev, ino)), Ok(m)) => m.is_file() && m.dev() == dev && m.ino() == ino,
        _ => false,
    }
}

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
         * 输入就是 stdout 时先跳过
         * 否则会边读边把匹配行追加到自己后面, 永远读不到 EOF
         * 用打开后的 fd 判断, 避免 stat 与 open 之间的竞态
         */
        if is_stdout(&file) {
            eprintln!("{}: input file is also the output", file_path.display());
            return Ok(())
        }
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
        let length: usize = finder.needle().len();

        loop {
            // 清空上一次的结果
            line.clear();
            // 读至末尾
            if reader.read_until(b'\n', &mut line)? == 0 {
                break;
            }
            // 行号+1
            index = index + 1;

            // 去掉末尾的换行符
            if line.last() == Some(&b'\n') { line.pop(); }

            // 匹配关键词
            let it = finder.find_iter(&line);
            
            // 无匹配结果
            if !it.clone().any(|_| true) {
                continue;
            }
            
            let mut raw_context = String::from_utf8_lossy(&line).to_string();

            if *IS_TERMINAL {
                // 终端输出时, 高亮显示匹配结果
                let mut attrs: Vec<PrinterAttr> = Vec::new();
                for start in it {
                    attrs.push(
                        PrinterAttr {
                            color: Color::Red,
                            position: start..start + length,
                        }
                    );
                }
                colorful_contact(&mut raw_context, &attrs);
            } 
            println!(
                "{}",
                format!(
                    "{}:{}:{}",
                    file_path.display(),
                    index,
                    &raw_context
                )
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


// TODO: 文件过多是, 需要双线程处理, 避免长时间堵塞