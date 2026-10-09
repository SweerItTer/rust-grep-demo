pub struct Args {
    pub pattern: String,
}

impl Args {
    pub fn parse() -> Result<Self, String> {
        // 拿到命令行输入的所有参数 ["main", "p1"...]
        let mut argv = std::env::args();

        // 迭代器取出打一个元素, 并检查是否为空, 为空则构造字符串 "程序"
        let app = argv.next().unwrap_or_else(|| "程序".to_string());

        // 迭代器继续取出下一个元素, 检查是否为空, 为空标记错误, 并格式化字符串, 通过 `?` 传递错误
        let pattern = argv
            .next()
            .ok_or_else(|| format!("usage: {} <pattern>", app))
            ?; // 取出 String，或直接 return Err

        // 构造无异常的 Result
        Ok(Self { pattern })
    }
}
