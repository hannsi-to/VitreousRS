use std::fmt;
use crate::logger::ansi_format::ANSIFormat;

#[macro_export]
macro_rules! debug_ln {
    ($($arg:tt)*) => {
        $crate::logger::debug::(format_args!($($arg)*), true)
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logger::debug::(format_args!($($arg)*), false)
    }
}

pub fn _error(args: fmt::Arguments,nl: bool){
    use std::io::Write;
    let reset = ANSIFormat::RESET.code();
    let print_format = ANSIFormat::RED.code();
    if nl {
        let _ = writeln!(std::io::stderr(), "{}{}{}", print_format, args, reset);
    } else {
        let _ = write!(std::io::stderr(), "{}{}{}", print_format, args, reset);
    }
}

pub fn _print(args: fmt::Arguments, nl: bool) {
    use std::io::Write;
    let reset = ANSIFormat::RESET.code();
    let print_format = ANSIFormat::RED.code();
    if nl {
        let _ = writeln!(std::io::stderr(), "{}{}{}", print_format, args, reset);
    } else {
        let _ = write!(std::io::stderr(), "{}{}{}", print_format, args, reset);
    }
}