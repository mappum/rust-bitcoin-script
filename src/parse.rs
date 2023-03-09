use bitcoin::blockdata::opcodes::All as Opcode;
use lazy_static::lazy_static;
use proc_macro2::{
    Span, TokenStream,
    TokenTree::{self, *},
};
use std::collections::HashMap;

// index opcodes by identifier string
lazy_static! {
    static ref OPCODES: HashMap<String, Opcode> = {
        let mut map = HashMap::with_capacity(256);
        for i in 0..=255 {
            let opcode = Opcode::from(i);
            let name = format!("{:?}", opcode);
            map.insert(name, opcode);
        }
        map
    };
}

#[derive(Debug)]
pub enum Syntax {
    Opcode(Opcode),
    Escape(TokenStream),
    Bytes(Vec<u8>),
    Int(i64),
}

macro_rules! emit_error {
    ($span:expr, $($message:expr),*) => {{
        #[cfg(not(test))]
        proc_macro_error::emit_error!($span, $($message),*);

        #[cfg(test)]
        panic!($($message),*);

        #[allow(unreachable_code)]
        {
            panic!();
        }
    }}
}

macro_rules! abort {
    ($span:expr, $($message:expr),*) => {{
        #[cfg(not(test))]
        proc_macro_error::abort!($span, $($message),*);

        #[cfg(test)]
        panic!($($message),*);
    }}
}

pub fn parse(tokens: TokenStream) -> Vec<(Syntax, Span)> {
    let mut tokens = tokens.into_iter();
    let mut syntax = Vec::with_capacity(64);

    while let Some(token) = tokens.next() {
        let token_str = token.to_string();

        syntax.push(match (&token, token_str.as_ref()) {
            // identifier, look up opcode
            (Ident(_), _) => {
                let opcode = OPCODES.get(&token_str).unwrap_or_else(|| {
                    emit_error!(token.span(), "unknown opcode \"{}\"", token_str);
                });
                (Syntax::Opcode(*opcode), token.span())
            }

            // '<', start of escape (parse until first '>')
            (Punct(_), "<") => parse_escape(token, &mut tokens),

            // literal, push data (int or bytes)
            (Literal(_), _) => parse_data(token),

            // negative sign, parse negative int
            (Punct(_), "-") => parse_negative_int(token, &mut tokens),

            // anything else is invalid
            _ => abort!(token.span(), "unexpected token"),
        });
    }

    syntax
}

fn parse_escape<T>(token: TokenTree, tokens: &mut T) -> (Syntax, Span)
where
    T: Iterator<Item = TokenTree>,
{
    let mut escape = TokenStream::new();
    let mut span = token.span();

    loop {
        let token = tokens
            .next()
            .unwrap_or_else(|| abort!(token.span(), "unterminated escape"));
        let token_str = token.to_string();

        span = span.join(token.span()).unwrap_or(token.span());

        // end of escape
        if let (Punct(_), ">") = (&token, token_str.as_ref()) {
            break;
        }

        escape.extend(TokenStream::from(token));
    }

    (Syntax::Escape(escape), span)
}

fn parse_data(token: TokenTree) -> (Syntax, Span) {
    if token.to_string().starts_with("0x") {
        parse_bytes(token)
    } else {
        parse_int(token, false)
    }
}

fn parse_bytes(token: TokenTree) -> (Syntax, Span) {
    let hex_bytes = &token.to_string()[2..];
    let bytes = hex::decode(hex_bytes).unwrap_or_else(|err| {
        emit_error!(token.span(), "invalid hex literal ({})", err);
    });
    (Syntax::Bytes(bytes), token.span())
}

fn parse_int(token: TokenTree, negative: bool) -> (Syntax, Span) {
    let token_str = token.to_string();
    let n: i64 = token_str.parse().unwrap_or_else(|err| {
        emit_error!(token.span(), "invalid number literal ({})", err);
    });
    let n = if negative { n * -1 } else { n };
    (Syntax::Int(n), token.span())
}

fn parse_negative_int<T>(token: TokenTree, tokens: &mut T) -> (Syntax, Span)
where
    T: Iterator<Item = TokenTree>,
{
    let fail = || {
        #[allow(unused_variables)]
        let span = token.span();
        emit_error!(
            span,
            "expected negative sign to be followed by number literal"
        );
    };

    let maybe_token = tokens.next();

    if let Some(token) = maybe_token {
        if let Literal(_) = token {
            parse_int(token, true)
        } else {
            fail()
        }
    } else {
        fail()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::blockdata::opcodes::all as opcodes;
    use proc_macro2::TokenTree;
    use quote::quote;

    #[test]
    fn parse_empty() {
        assert!(parse(quote!()).is_empty());
    }

    #[test]
    #[should_panic(expected = "unexpected token")]
    fn parse_unexpected_token() {
        parse(quote!(OP_CHECKSIG &));
    }

    #[test]
    #[should_panic(expected = "unknown opcode \"A\"")]
    fn parse_invalid_opcode() {
        parse(quote!(OP_CHECKSIG A B));
    }

    #[test]
    fn parse_opcodes() {
        let syntax = parse(quote!(OP_CHECKSIG OP_HASH160));

        if let Syntax::Opcode(opcode) = syntax[0].0 {
            assert_eq!(opcode, opcodes::OP_CHECKSIG);
        } else {
            panic!();
        }

        if let Syntax::Opcode(opcode) = syntax[1].0 {
            assert_eq!(opcode, opcodes::OP_HASH160);
        } else {
            panic!();
        }
    }

    #[test]
    #[should_panic(expected = "unterminated escape")]
    fn parse_unterminated_escape() {
        parse(quote!(OP_CHECKSIG < abc));
    }

    #[test]
    fn parse_escape() {
        let syntax = parse(quote!(OP_CHECKSIG<abc>));

        if let Syntax::Escape(tokens) = &syntax[1].0 {
            let tokens = tokens.clone().into_iter().collect::<Vec<TokenTree>>();

            assert_eq!(tokens.len(), 1);
            if let TokenTree::Ident(_) = tokens[0] {
                assert_eq!(tokens[0].to_string(), "abc");
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    #[test]
    #[should_panic(expected = "invalid number literal (invalid digit found in string)")]
    fn parse_invalid_int() {
        parse(quote!(OP_CHECKSIG 12g34));
    }

    #[test]
    fn parse_int() {
        let syntax = parse(quote!(OP_CHECKSIG 1234));

        if let Syntax::Int(n) = syntax[1].0 {
            assert_eq!(n, 1234i64);
        } else {
            panic!()
        }
    }

    #[test]
    #[should_panic(expected = "expected negative sign to be followed by number literal")]
    fn parse_invalid_negative_sign() {
        parse(quote!(OP_CHECKSIG - OP_HASH160));
    }

    #[test]
    fn parse_negative_int() {
        let syntax = parse(quote!(OP_CHECKSIG - 1234));

        if let Syntax::Int(n) = syntax[1].0 {
            assert_eq!(n, -1234i64);
        } else {
            panic!()
        }
    }

    #[test]
    #[should_panic(expected = "invalid hex literal (Odd number of digits)")]
    fn parse_invalid_hex() {
        parse(quote!(OP_CHECKSIG 0x123));
    }

    #[test]
    fn parse_hex() {
        let syntax = parse(quote!(OP_CHECKSIG 0x1234));

        if let Syntax::Bytes(bytes) = &syntax[1].0 {
            assert_eq!(bytes, &vec![0x12, 0x34]);
        } else {
            panic!()
        }
    }
}
