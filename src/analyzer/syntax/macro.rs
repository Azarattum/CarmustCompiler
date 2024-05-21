#[macro_export]
macro_rules! first {
    ($first:expr$(, $($rest:expr),*)?) => {
        $first
    };
}

#[macro_export]
macro_rules! nested {
    ($stream:ident, $name:ident, $($case:pat$(, $($cases:pat),*)? => $result:expr);*) => {
        match $stream.peek().map(|&x| x) {
            $(Some($case) => {
                $stream.next();
                first!($(nested!($stream, $name, $($cases),* => $result),)? Ok($result))},
            )*
            token => Err(SyntaxError {expected: stringify!($name), found: token})
        }
    };
}

#[macro_export]
macro_rules! syntax {
    ($name:ident -> $type:ty: $($($cases:pat),+ => $result:expr;)+) => {
        pub fn $name<'a>(stream: &mut Peekable<impl TokenStream<'a>>) -> Result<$type, SyntaxError<'a>> {
            nested!(stream, $name, $($($cases),+ => $result);+)
        }
    };
}
