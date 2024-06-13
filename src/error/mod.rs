pub mod assembly;
pub mod compile;
pub mod semantic;
pub mod syntax;

use colored::Colorize;
use std::{
    cmp::{max, min},
    process::exit,
};

pub trait ErrorLike {
    fn slice(&self) -> Option<&str>;
    fn message(&self) -> String;
    fn kind() -> &'static str;

    fn lookup(text: &str, slice: Option<&str>) -> (usize, usize) {
        let offset = match slice {
            Some(slice) => slice.as_ptr() as usize - text.as_ptr() as usize,
            None => text.len(),
        };

        let line = text[..min(offset + 1, text.len())].lines().count();
        let start = text[..offset]
            .char_indices()
            .rev()
            .find(|x| x.1 == '\n')
            .and_then(|x| Some(x.0 as i64))
            .unwrap_or(-1);

        return (line - 1, (offset as i64 - start - 1) as usize);
    }

    fn report(&self, code: &str, filename: &str) -> String {
        let token = self.slice();
        let (line, char) = Self::lookup(code, token);
        let pad = (line + 2).to_string().len() + 2;
        let length = token.unwrap_or("").len();
        let message = self.message();

        let snippet = code
            .lines()
            .enumerate()
            .skip(max(line as i64 - 1, 0 as i64).try_into().unwrap())
            .take(3)
            .map(|(n, text)| {
                format!(
                    "{: >pad$} {} {}",
                    (n + 1).to_string().blue().bold(),
                    "|".blue().bold(),
                    if n == line {
                        let before = &text[..char].underline();
                        let highlighted = &text[char..char + length].red().bold().underline();
                        let after = &text[char + length..].underline();
                        format!("{}{}{}", before, highlighted, after)
                    } else {
                        text.to_owned()
                    },
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let spacer_pad = pad + 3;
        format!(
            "{}{} {}\n   {: >pad$} {}:{}:{}\n{: >spacer_pad$}\n{}",
            Self::kind().red().bold(),
            ":".red().bold(),
            message.red(),
            "-->".blue().bold(),
            filename.cyan(),
            (line + 1).to_string().cyan(),
            (char + 1).to_string().cyan(),
            "| ".blue().bold(),
            snippet
        )
    }

    fn crash(&self, code: &str, filename: &str) -> ! {
        println!("{}", self.report(code, filename));
        exit(1);
    }
}
