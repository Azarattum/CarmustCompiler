#[macro_export]
macro_rules! first {
    ($first:expr$(, $($rest:expr),*)?) => {
        $first
    };
}

#[macro_export]
macro_rules! nested {
    ($stream:ident, $expected:ident, $($case:pat $(if $cond:expr)? $(, $($cases:pat $(if $conds:expr)?),*)? => $result:expr);*) => {
        match $stream.peek().map(|&x| x) {
            $(Some($case) $(if $cond)? => {
                $stream.next();
                first!($(nested!($stream, $expected, $($cases $(if $conds)?),* => $result),)? Ok($result))},
            )*
            token => Err(SyntaxError {expected: $expected, found: token})
        }
    };
}

#[macro_export]
macro_rules! syntax {
    ($name:ident($($arg_name:ident: $arg_type:ty),*) -> $type:ty: $($($cases:pat $(if $conds:expr)?),+ => $result:expr;)+) => {
        pub fn $name<'a>(stream: &mut Peekable<impl TokenStream<'a>>, $($arg_name: $arg_type),*) -> Result<$type, SyntaxError<'a>> {
            let expected = {
                let parts: Vec<String> = vec![$(format!("{:?}", $arg_name)),*];
                if parts.len() > 0 {
                    format!("{}({})", stringify!($name), parts.join(", "))
                } else {
                    stringify!($name).to_owned()
                }
            };

            nested!(stream, expected, $($($cases $(if $conds)?),+ => $result);+)
        }
    };
}
