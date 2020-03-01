#![feature(proc_macro_hygiene)]

use bitcoin::blockdata::opcodes::All as Opcode;
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
    quote!(123).into()
}

fn parse(tokens: TokenStream) -> Vec<Syntax> {
    let mut tokens = tokens.into_iter();
    let mut syntax = Vec::with_capacity(64);

    while let Some(token) = tokens.next() {
        let token_str = token.to_string();

        match (&token, token_str.as_ref()) {
            (Punct(_), "<") => {
                let mut escape = vec![];

                while let maybe_token = tokens.next() {
                    let token = maybe_token.unwrap_or_else(
                        || abort!(token.span(), "Unterminated escape")
                    );
                    let token_str = token.to_string();

                    if let (Punct(_), ">") = (&token, token_str.as_ref()) {
                        syntax.push(Syntax::Escape(escape));
                        break;
                    }

                    escape.push(token);
                }
            },

            (Ident(_), _) => {
                let opcode = OPCODES.get(&token_str)
                    .unwrap_or_else(|| abort!(token.span(), "Unknown opcode"));
                syntax.push(Syntax::Opcode(*opcode))
            },

            (Literal(_), _) => {
                // TODO
                syntax.push(Syntax::Data(vec![]))
            },

            _ => emit_error!(token.span(), "Unexpected token")
        }
    }

    syntax
}

#[derive(Debug)]
enum Syntax {
    Opcode(Opcode),
    Escape(Vec<TokenTree>),
    Data(Vec<u8>)
}
