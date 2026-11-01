pub struct PrinterAttr {
    pub color: Color,
    pub position: std::ops::Range<usize>,
}

pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    None,
}

impl Color {
    fn code(&self) -> &'static str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::None => "\x1b[0m",
        }
    }
}

pub fn colorful_contact(raw_context: &mut String, attrs: &[PrinterAttr]) {
    // 先按位置从大到小排好，保证处理顺序 = 字符串里从后往前
    let mut sorted: Vec<&PrinterAttr> = attrs.iter().collect();
    sorted.sort_by(|a, b| b.position.start.cmp(&a.position.start));

    for attr in sorted {
        // 获取对应位置的内容
        let origin = raw_context[attr.position.clone()].to_string();
        // 为 origin 上色
        let colored = format!("{}{}{}", attr.color.code(), origin, Color::None.code());
        // 反向替换原始字符串中的内容
        raw_context.replace_range(attr.position.clone(), &colored);
    }
}

#[test]
fn test_print_colorful() {
    // 每行末尾的 \ 会吞掉换行和下一行的缩进，只保留内容
    let mut raw_context = String::from(
        "Hello!\n\
         You might have wondered why we often prefer to write code late at night\n\
         in the quiet, with headphones on and music playing.\n\
         To be honest, I don't know the answer myself\n\
         but once you've tried it, you'll realize that\n\
         it doesn't matter at all\n\
         if you don't know why.\n"
    );

    let attrs = vec![
        PrinterAttr { color: Color::Yellow, position:  60..64  }, // code
        PrinterAttr { color: Color::Red,    position:  65..78  }, // late at night
        PrinterAttr { color: Color::Yellow, position:  98..108 }, // headphones
        PrinterAttr { color: Color::Blue,   position: 209..216 }, // realize
        PrinterAttr { color: Color::Green,  position: 225..246 }, // doesn't matter at all
    ];

    colorful_contact(&mut raw_context, &attrs);
    println!("{}", raw_context);
}