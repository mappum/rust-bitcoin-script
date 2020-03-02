#![feature(proc_macro_hygiene)]

use bitcoin::blockdata::opcodes::{All as Opcode, all as opcodes};
use lazy_static::lazy_static;
use proc_macro::{TokenTree::{self, *}, TokenStream};
use proc_macro_error::{proc_macro_error, abort, emit_error, set_dummy};
use quote::quote;
use std::collections::HashMap;

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

#[proc_macro]
#[proc_macro_error]
pub fn script(tokens: TokenStream) -> TokenStream {
    set_dummy(quote!((::bitcoin::Script::new())));

    let syntax = parse(tokens);
    println!("{:?}", syntax);

    // let input_as_str = format!("{}", input);
    quote!((123)).into()
}

#[derive(Debug)]
enum Syntax {
    Opcode(Opcode),
    Escape(Vec<TokenTree>),
    Bytes(Vec<u8>),
    Int(i64)
}

fn parse(tokens: TokenStream) -> Vec<Syntax> {
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
                Syntax::Opcode(*opcode)
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

fn parse_escape<T>(token: TokenTree, tokens: &mut T) -> Syntax
    where T: Iterator<Item=TokenTree>
{
    let mut escape = vec![];

    loop {
        let token = tokens.next().unwrap_or_else(|| {
            abort!(token.span(), "unterminated escape")
        });
        let token_str = token.to_string();

        // end of escape
        if let (Punct(_), ">") = (&token, token_str.as_ref()) {
            break;
        }

        escape.push(token);
    }

    Syntax::Escape(escape)
}

fn parse_data(token: TokenTree) -> Syntax {
    if token.to_string().starts_with("0x") {
       parse_bytes(token)
    } else {
       parse_int(token, false)
    }
}

fn parse_bytes(token: TokenTree) -> Syntax {
    let hex_bytes = &token.to_string()[2..];
    let bytes = hex::decode(hex_bytes).unwrap_or_else(|err| {
        emit_error!(token.span(), "invalid hex literal ({})", err);
        vec![]
    });
    Syntax::Bytes(bytes)
}

fn parse_int(token: TokenTree, negative: bool) -> Syntax {
    let token_str = token.to_string();
    let n: i64 = token_str.parse().unwrap_or_else(|err| {
        emit_error!(token.span(), "invalid number literal ({})", err);
        0
    });
    let n = if negative { n * -1 } else { n };
    Syntax::Int(n)
}

fn parse_negative_int<T>(token: TokenTree, tokens: &mut T) -> Syntax
    where T: Iterator<Item=TokenTree>
{
    let fail = || {
        emit_error!(
            token.span(),
            "expected negative sign to be followed by number literal"
        );
        Syntax::Int(0)
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
