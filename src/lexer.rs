#[derive(Debug)]
pub enum Token {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    DoubleEq,
    Equal,
    LessThan,
    DoubleLt,
    LtEq,
    GreaterThan,
    DoubleGt,
    TripleGt,
    GtEq,
    Ampersand,
    Pipe,
    Circumflex,
    ExclEq,
    Excl,
    Tilde,
    Semicolon,
    Dot,
    Comma,
    Colon,
    OpeningBracket,
    ClosingBracket,
    OpeningBrace,
    ClosingBrace,
    OpeningParens,
    ClosingParens,
    FloatKeyword,
    IntKeyword,
    IfKeyword,
    GetKeyword,
    SetKeyword,
    ParamKeyword,
    WhileKeyword,
    FtoiKeyword,
    ItofKeyword,
    AllocKeyword,
    ElseKeyword,
    PutcKeyword,
    GetcKeyword,
    ParamsKeyword,
    FuncKeyword,
    ReturnKeyword,
    Int(i64),
    Float(f64),
    Ident(String),
}

macro_rules! next_if_matches {
    ($iter:expr, [$($p:pat),*]) => {
        if $iter.as_slice().len() >= [$(stringify!($p)),*].len() && matches!($iter.as_slice()[..[$(stringify!($p)),*].len()], [$($p),*]) {
            $iter.next()
        } else {
            None
        }
    };
}

pub fn parse(s: &str) -> Vec<Token> {
    let mut iter = s.as_bytes().iter();
    let mut tokens = Vec::new();

    loop {
        let Some(next_char) = iter.next().copied() else {
            return tokens;
        };
        match next_char {
            b'+' => tokens.push(Token::Plus),
            b'-' => tokens.push(Token::Minus),
            b'*' => tokens.push(Token::Asterisk),
            b'/' => {
                if next_if_matches!(iter, [b'/']).is_some() {
                    while iter.as_slice().first().is_some_and(|c| *c != b'\n') {
                        iter.next();
                    }
                } else {
                    tokens.push(Token::Slash)
                }
            }
            b'%' => tokens.push(Token::Percent),
            b'&' => tokens.push(Token::Ampersand),
            b'|' => tokens.push(Token::Pipe),
            b'^' => tokens.push(Token::Circumflex),
            b'~' => tokens.push(Token::Tilde),
            b';' => tokens.push(Token::Semicolon),
            b':' => tokens.push(Token::Colon),
            b',' => tokens.push(Token::Comma),
            b'.' => tokens.push(Token::Dot),
            b'[' => tokens.push(Token::OpeningBracket),
            b']' => tokens.push(Token::ClosingBracket),
            b'(' => tokens.push(Token::OpeningParens),
            b')' => tokens.push(Token::ClosingParens),
            b'{' => tokens.push(Token::OpeningBrace),
            b'}' => tokens.push(Token::ClosingBrace),
            b'=' => tokens.push(if next_if_matches!(iter, [b'=']).is_some() {
                Token::DoubleEq
            } else {
                Token::Equal
            }),
            b'!' => tokens.push(if next_if_matches!(iter, [b'=']).is_some() {
                Token::ExclEq
            } else {
                Token::Excl
            }),
            b'>' => tokens.push(if next_if_matches!(iter, [b'=']).is_some() {
                Token::GtEq
            } else if next_if_matches!(iter, [b'>']).is_some() {
                if next_if_matches!(iter, [b'>']).is_some() {
                    Token::TripleGt
                } else {
                    Token::DoubleGt
                }
            } else {
                Token::GreaterThan
            }),
            b'<' => tokens.push(if next_if_matches!(iter, [b'=']).is_some() {
                Token::LtEq
            } else if next_if_matches!(iter, [b'<']).is_some() {
                Token::DoubleLt
            } else {
                Token::LessThan
            }),
            b'A'..=b'Z' | b'_' | b'a'..=b'z' => {
                let begin_cursor = iter.as_slice().as_ptr() as usize - 1;
                while next_if_matches!(iter, [b'0'..=b'9' | b'A'..=b'Z' | b'_' | b'a'..=b'z'])
                    .is_some()
                { /* empty */ }
                let end_cursor = iter.as_slice().as_ptr() as usize;
                let ident = unsafe {
                    core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                        begin_cursor as *const u8,
                        end_cursor - begin_cursor,
                    ))
                };
                tokens.push(match ident.as_bytes() {
                    b"while" => Token::WhileKeyword,
                    b"if" => Token::IfKeyword,
                    b"else" => Token::ElseKeyword,
                    b"set" => Token::SetKeyword,
                    b"get" => Token::GetKeyword,
                    b"int" => Token::IntKeyword,
                    b"float" => Token::FloatKeyword,
                    b"i_to_f" => Token::ItofKeyword,
                    b"f_to_i" => Token::FtoiKeyword,
                    b"alloc" => Token::AllocKeyword,
                    b"getc" => Token::GetcKeyword,
                    // b"printfloat" => Token::PrintFloatKeyword,
                    // b"printint" => Token::PrintIntKeyword,
                    b"putc" => Token::PutcKeyword,
                    b"func" => Token::FuncKeyword,
                    b"params" => Token::ParamsKeyword,
                    b"param" => Token::ParamKeyword,
                    b"return" => Token::ReturnKeyword,
                    _ => Token::Ident(ident.to_owned()),
                })
            }
            b'0'..=b'9' => {
                let begin_cursor = iter.as_slice().as_ptr() as usize - 1;
                while next_if_matches!(iter, [b'0'..=b'9']).is_some() { /* empty */ }
                if next_if_matches!(iter, [b'.', b'0'..=b'9']).is_some() {
                    iter.next();
                    while next_if_matches!(iter, [b'0'..=b'9']).is_some() { /* empty */ }
                    let end_cursor = iter.as_slice().as_ptr() as usize;
                    let num_content = unsafe {
                        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                            begin_cursor as *const u8,
                            end_cursor - begin_cursor,
                        ))
                    };
                    tokens.push(Token::Float(num_content.parse().unwrap()))
                } else {
                    let end_cursor = iter.as_slice().as_ptr() as usize;
                    let num_content = unsafe {
                        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                            begin_cursor as *const u8,
                            end_cursor - begin_cursor,
                        ))
                    };
                    tokens.push(Token::Int(num_content.parse().unwrap()))
                }
            }
            b'\n' | b'\t' | b'\r' | b' ' => {}
            _ => panic!(
                "Unexpected char {:?}",
                char::from_u32(next_char.into()).unwrap()
            ),
        }
    }
}
