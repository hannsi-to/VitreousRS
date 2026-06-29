#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ANSIFormat {
    Reset = 0,
    Bold = 1,
    Italic = 3,
    Underline = 4,
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
    BgBlack = 40,
    BgRed = 41,
    BgGreen = 42,
    BgYellow = 43,
    BgBlue = 44,
    BgMagenta = 45,
    BgCyan = 46,
    BgWhite = 47,
    BrightBlack = 90,
    BrightRed = 91,
    BrightGreen = 92,
    BrightYellow = 93,
    BrightBlue = 94,
    BrightMagenta = 95,
    BrightCyan = 96,
    BrightWhite = 97,
}

impl ANSIFormat {
    pub const fn value(self) -> u8 {
        self as u8
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::Reset => "\u{001B}[0m",

            Self::Bold => "\u{001B}[1m",
            Self::Italic => "\u{001B}[3m",
            Self::Underline => "\u{001B}[4m",

            Self::Black => "\u{001B}[30m",
            Self::Red => "\u{001B}[31m",
            Self::Green => "\u{001B}[32m",
            Self::Yellow => "\u{001B}[33m",
            Self::Blue => "\u{001B}[34m",
            Self::Magenta => "\u{001B}[35m",
            Self::Cyan => "\u{001B}[36m",
            Self::White => "\u{001B}[37m",

            Self::BgBlack => "\u{001B}[40m",
            Self::BgRed => "\u{001B}[41m",
            Self::BgGreen => "\u{001B}[42m",
            Self::BgYellow => "\u{001B}[43m",
            Self::BgBlue => "\u{001B}[44m",
            Self::BgMagenta => "\u{001B}[45m",
            Self::BgCyan => "\u{001B}[46m",
            Self::BgWhite => "\u{001B}[47m",

            Self::BrightBlack => "\u{001B}[90m",
            Self::BrightRed => "\u{001B}[91m",
            Self::BrightGreen => "\u{001B}[92m",
            Self::BrightYellow => "\u{001B}[93m",
            Self::BrightBlue => "\u{001B}[94m",
            Self::BrightMagenta => "\u{001B}[95m",
            Self::BrightCyan => "\u{001B}[96m",
            Self::BrightWhite => "\u{001B}[97m",
        }
    }
}