use bitcoin::blockdata::opcodes::{All as Opcode, all as opcodes};
use lazy_static::lazy_static;
use proc_macro2::{TokenTree::{self, *}, TokenStream, Span};
use proc_macro_error::{abort, emit_error};
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
    Int(i64)
}

pub fn parse(tokens: TokenStream) -> Vec<(Syntax, Span)> {
    let mut tokens = tokens.into_iter();
    let mut syntax = Vec::with_capacity(64);

    while let Some(token) = tokens.next() {
        let token_str = token.to_string();

        syntax.push(match (&token, token_str.as_ref()) {
            // identifier, look up opcode
            (Ident(_), _) => {
                let opcode = OPCODES.get(&token_str)
                    .unwrap_or_else(|| {
                        emit_error!(token.span(), "unknown opcode");
                        &opcodes::OP_NOP
                    });
                (Syntax::Opcode(*opcode), token.span())
            },

            // '<', start of escape (parse until first '>')
            (Punct(_), "<") => parse_escape(token, &mut tokens),

            // literal, push data (int or bytes)
            (Literal(_), _) => parse_data(token),

            // negative sign, parse negative int
            (Punct(_), "-") => parse_negative_int(token, &mut tokens),

            // anything else is invalid 
            _ => abort!(token.span(), "unexpected token")
        });
    }

    syntax
}

fn parse_escape<T>(token: TokenTree, tokens: &mut T) -> (Syntax, Span)
    where T: Iterator<Item=TokenTree>
{
    let mut escape = TokenStream::new();
    let mut span = token.span();

    loop {
        let token = tokens.next().unwrap_or_else(|| {
            abort!(token.span(), "unterminated escape")
        });
        let token_str = token.to_string();

        span = span.join(token.span())
            .unwrap_or(token.span());

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
        vec![]
    });
    println!("BYTES! {:?}", bytes);
    (Syntax::Bytes(bytes), token.span())
}

fn parse_int(token: TokenTree, negative: bool) -> (Syntax, Span) {
    let token_str = token.to_string();
    let n: i64 = token_str.parse().unwrap_or_else(|err| {
        emit_error!(token.span(), "invalid number literal ({})", err);
        0
    });
    let n = if negative { n * -1 } else { n };
    (Syntax::Int(n), token.span())
}

fn parse_negative_int<T>(token: TokenTree, tokens: &mut T) -> (Syntax, Span)
    where T: Iterator<Item=TokenTree>
{
    let fail = || {
        emit_error!(
            token.span(),
            "expected negative sign to be followed by number literal"
        );
        (Syntax::Int(0), token.span())
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
