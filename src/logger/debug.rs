use std::fmt;
use crate::logger::ansi_format::ANSIFormat;

#[macro_export]
macro_rules! debug_ln {
    ($($arg:tt)*) => {
        $crate::logger::debug::_debug(format_args!($($arg)*), true)
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logger::debug::_debug(format_args!($($arg)*), false)
    }
}

#[macro_export]
macro_rules! info_ln {
    ($($arg:tt)*) => {
        $crate::logger::debug::_info(format_args!($($arg)*), true)
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::logger::debug::_info(format_args!($($arg)*), false)
    }
}

#[macro_export]
macro_rules! warning_ln {
    ($($arg:tt)*) => {
        $crate::logger::debug::_warning(format_args!($($arg)*), true)
    }
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {
        $crate::logger::debug::_warning(format_args!($($arg)*), false)
    }
}

#[macro_export]
macro_rules! error_ln {
    ($($arg:tt)*) => {
        $crate::logger::debug::_error(format_args!($($arg)*), true)
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::logger::debug::_error(format_args!($($arg)*), false)
    }
}

pub fn _error(args: fmt::Arguments,nl: bool){
    _console_write(args, nl, ANSIFormat::Red.code());
}

pub fn _warning(args: fmt::Arguments,nl: bool) {
    _console_write(args, nl, ANSIFormat::Yellow.code());
}

pub fn _info(args: fmt::Arguments,nl: bool) {
    _console_write(args, nl, ANSIFormat::Blue.code());
}

pub fn _debug(args: fmt::Arguments, nl: bool) {
    _console_write(args, nl, ANSIFormat::Reset.code());
}

fn _console_write(args: fmt::Arguments, nl: bool, print_format: &str) {
    use std::io::Write;
    let reset = ANSIFormat::Reset.code();
    if nl {
        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        let _ = writeln!(handle, "{}{}{}", print_format, args, reset);
        let _ = handle.flush();
    } else {
        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        let _ = write!(handle, "{}{}{}", print_format, args, reset);
        let _ = handle.flush();
    }
}