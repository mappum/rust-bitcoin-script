#![feature(proc_macro_hygiene)]

use bitcoin::blockdata::opcodes::All as Opcode;
use byteorder::{ByteOrder, LittleEndian};
use lazy_static::lazy_static;
use proc_macro::{TokenTree::{self, *}, TokenStream};
use proc_macro_error::{proc_macro_error, abort, set_dummy};
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
    Data(Vec<u8>)
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
                    .unwrap_or_else(
                        || abort!(token.span(), "unknown opcode")
                    );
                Syntax::Opcode(*opcode)
            },

            // '<', start of escape (parse until first '>')
            (Punct(_), "<") => parse_escape(token, &mut tokens),

            // literal, push data
            (Literal(_), _) => parse_data(token),

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
    let token_str = token.to_string();

    // TODO: support negative literals

    // hex strings (0x...)
    if token_str.starts_with("0x") {
        let hex_bytes = &token_str[2..];
        let bytes = hex::decode(hex_bytes).unwrap_or_else(|err| {
            abort!(token.span(), "invalid hex literal ({})", err)
        });
       return Syntax::Data(bytes);
    }

    let n: u32 = token_str.parse().unwrap_or_else(|err| {
        abort!(token.span(), "invalid decimal literal ({})", err)
    });
    let mut bytes = vec![0; 4];
    LittleEndian::write_u32(&mut bytes, n);
    Syntax::Data(bytes)
}
