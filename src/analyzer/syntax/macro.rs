#[macro_export]
macro_rules! first_expr {
    ($first:expr$(, $($rest:expr),*)?) => {
        $first
    };
}

#[macro_export]
macro_rules! first_ident {
    ($first:ident$(, $($rest:ident),*)?) => {
        $first
    };
}

#[macro_export]
macro_rules! nested {
    ($stream:expr, $expected:ident, $($case:pat $(if $cond:expr)? $(, $($cases:pat $(if $conds:expr)?),*)? => $result:expr);*) => {
        match $stream.peek().map(|&x| x) {
            $(Some($case) $(if $cond)? => {
                $stream.next();
                first_expr!($(nested!($stream, $expected, $($cases $(if $conds)?),* => $result),)? Ok($result))},
            )*
            token => Err(SyntaxError {expected: $expected, found: token})
        }
    };
}

#[macro_export]
macro_rules! syntax {
    ($name:ident($($arg_name:ident: $arg_type:ty),*)$( with $stream_name:ident)? -> $type:ty: $($($cases:pat $(if $conds:expr)?),+ => $result:expr;)+) => {
        pub fn $name<'a>(stream: &mut Peekable<impl TokenStream<'a>>, $($arg_name: $arg_type),*) -> Result<$type, SyntaxError<'a>> {
            let expected = {
                let parts: Vec<String> = vec![$(format!("{:?}", $arg_name)),*];
                if parts.len() > 0 {
                    format!("{}({})", stringify!($name), parts.join(", "))
                } else {
                    stringify!($name).to_owned()
                }
            };

            let first_ident!($($stream_name,)? stream) = stream;
            nested!(first_ident!($($stream_name,)? stream), expected, $($($cases $(if $conds)?),+ => {
                $result
            });+)
        }
    };
}
