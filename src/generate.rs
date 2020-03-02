use super::parse::Syntax;
use bitcoin::blockdata::opcodes::All as Opcode;
use proc_macro2::{TokenStream, TokenTree, Span, Ident, Literal};
use quote::{quote, quote_spanned};

pub fn generate(syntax: Vec<(Syntax, Span)>) -> TokenStream {
    let mut tokens = quote!(
        ::bitcoin::blockdata::script::Builder::new()
    );

    for (item, span) in syntax {
        let push = match item {
            Syntax::Opcode(opcode) => generate_opcode(opcode, span),
            Syntax::Bytes(bytes) => generate_bytes(bytes, span),
            Syntax::Int(int) => generate_int(int, span),
            Syntax::Escape(expression) => {
                let builder = tokens;
                tokens = TokenStream::new();
                generate_escape(builder, expression, span)
            }
        };
        tokens.extend(push);
    }

    tokens.extend(quote!(.into_script()));
    tokens.into()
}

fn generate_opcode(opcode: Opcode, span: Span) -> TokenStream {
    let ident = Ident::new(opcode.to_string().as_ref(), span);
    quote_spanned!(span=>
        .push_opcode(
            ::bitcoin::blockdata::opcodes::all::#ident
        )
    )
}

fn generate_bytes(bytes: Vec<u8>, span: Span) -> TokenStream {
    let mut slice = TokenStream::new();
    for byte in bytes {
        slice.extend(quote!(#byte,));
    }
    quote_spanned!(span=>.push_slice(&[#slice]))
}

fn generate_int(n: i64, span: Span) -> TokenStream {
    quote_spanned!(span=>.push_int(#n))
}

fn generate_escape(
    builder: TokenStream,
    expression: TokenStream,
    span: Span
) -> TokenStream {
    quote_spanned!(span=>
        (|builder, value| {
            mod __ {
                use ::bitcoin::blockdata::script::Builder;

                pub(super) trait Pushable {
                    fn bitcoin_script_push(&self, builder: Builder) -> Builder;
                }

                impl Pushable for &[u8] {
                    fn bitcoin_script_push(&self, builder: Builder) -> Builder {
                        builder.push_slice(self)
                    }
                }

                impl Pushable for Vec<u8> {
                    fn bitcoin_script_push(&self, builder: Builder) -> Builder {
                        builder.push_slice(self.as_ref())
                    }
                }

                impl Pushable for i64 {
                    fn bitcoin_script_push(&self, builder: Builder) -> Builder {
                        builder.push_int(*self)
                    }
                }

                impl Pushable for ::bitcoin::PublicKey {
                    fn bitcoin_script_push(&self, builder: Builder) -> Builder {
                        builder.push_key(&self)
                    }
                }

                // TODO: support more types
            }

            use ::bitcoin::blockdata::script::Builder;
            fn push(builder: Builder, value: impl __::Pushable) -> Builder {
                value.bitcoin_script_push(builder)
            }

            push(builder, value)
        })(
            #builder,
            #expression
        )
    )
}
