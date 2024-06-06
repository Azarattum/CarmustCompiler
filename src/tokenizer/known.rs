use lazy_static::lazy_static;
use regex::Regex;

pub const KEYWORDS: [&str; 8] = [
    "typedef", "int", "float", "short", "long", "char", "for", "return",
];

pub const SYMBOLS: [&str; 13] = [
    "[", "]", "{", "}", "(", ")", ";", "=", "+", "-", "*", "/", ",",
];

lazy_static! {
    pub static ref TOKEN_EXPRESSION: Regex =
        Regex::new(r"'[^']'|\/\/.*|(?s)\/\*.*?\*\/|\d+\.\d*|\w+|\S").unwrap();
}
