pub struct Args {
    pub pattern: String,
}

impl Args {
    pub fn parse() -> Args {
        let mut argv: Vec<String> = std::env::args().collect();
        return Args {
            pattern: argv.swap_remove(1)
        };
    }
}
