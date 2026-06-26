#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ANSIFormat {
    RESET = 0,
    BOLD = 1,
    ITALIC = 3,
    UNDERLINE = 4,
    BLACK = 30,
    RED = 31,
    GREEN = 32,
    YELLOW = 33,
    BLUE = 34,
    MAGENTA = 35,
    CYAN = 36,
    WHITE = 37,
    BG_BLACK = 40,
    BG_RED = 41,
    BG_GREEN = 42,
    BG_YELLOW = 43,
    BG_BLUE = 44,
    BG_MAGENTA = 45,
    BG_CYAN = 46,
    BG_WHITE = 47,
    BRIGHT_BLACK = 90,
    BRIGHT_RED = 91,
    BRIGHT_GREEN = 92,
    BRIGHT_YELLOW = 93,
    BRIGHT_BLUE = 94,
    BRIGHT_MAGENTA = 95,
    BRIGHT_CYAN = 96,
    BRIGHT_WHITE = 97,
}

impl ANSIFormat {
    pub const fn value(self) -> u8 {
        self as u8
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::RESET => "\u{001B}[0m",

            Self::BOLD => "\u{001B}[1m",
            Self::ITALIC => "\u{001B}[3m",
            Self::UNDERLINE => "\u{001B}[4m",

            Self::BLACK => "\u{001B}[30m",
            Self::RED => "\u{001B}[31m",
            Self::GREEN => "\u{001B}[32m",
            Self::YELLOW => "\u{001B}[33m",
            Self::BLUE => "\u{001B}[34m",
            Self::MAGENTA => "\u{001B}[35m",
            Self::CYAN => "\u{001B}[36m",
            Self::WHITE => "\u{001B}[37m",

            Self::BG_BLACK => "\u{001B}[40m",
            Self::BG_RED => "\u{001B}[41m",
            Self::BG_GREEN => "\u{001B}[42m",
            Self::BG_YELLOW => "\u{001B}[43m",
            Self::BG_BLUE => "\u{001B}[44m",
            Self::BG_MAGENTA => "\u{001B}[45m",
            Self::BG_CYAN => "\u{001B}[46m",
            Self::BG_WHITE => "\u{001B}[47m",

            Self::BRIGHT_BLACK => "\u{001B}[90m",
            Self::BRIGHT_RED => "\u{001B}[91m",
            Self::BRIGHT_GREEN => "\u{001B}[92m",
            Self::BRIGHT_YELLOW => "\u{001B}[93m",
            Self::BRIGHT_BLUE => "\u{001B}[94m",
            Self::BRIGHT_MAGENTA => "\u{001B}[95m",
            Self::BRIGHT_CYAN => "\u{001B}[96m",
            Self::BRIGHT_WHITE => "\u{001B}[97m",
        }
    }
}