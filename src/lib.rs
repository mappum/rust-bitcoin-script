//! [![Rust](https://github.com/mappum/rust-bitcoin-script/workflows/Rust/badge.svg)](https://github.com/mappum/rust-bitcoin-script/actions?query=workflow%3ARust)
//! [![crates.io](https://img.shields.io/crates/v/bitcoin-script.svg)](https://crates.io/crates/bitcoin-script)
//! [![docs.rs](https://docs.rs/bitcoin-script/badge.svg)](https://docs.rs/bitcoin-script)
//! 
//! **Bitcoin scripts inline in Rust.**
//! 
//! ---
//! 
//! ## Usage
//! 
//! This crate exports a `bitcoin_script!` macro which can be used to build Bitcoin scripts. The macro returns the [`Script`](https://docs.rs/bitcoin/0.23.0/bitcoin/blockdata/script/struct.Script.html) type from the [`bitcoin`](https://github.com/rust-bitcoin/rust-bitcoin) crate.
//! 
//! **Example:**
//! 
//! ```rust
//! use bitcoin_script::bitcoin_script;
//! 
//! let htlc_script = bitcoin_script! {
//!     OP_IF
//!         OP_SHA256 <digest> OP_EQUALVERIFY OP_DUP OP_SHA256 <seller_pubkey_hash>
//!     OP_ELSE
//!         100 OP_CSV OP_DROP OP_DUP OP_HASH160 <buyer_pubkey_hash>
//!     OP_ENDIF
//!     OP_EQUALVERIFY
//!     OP_CHECKSIG
//! };
//! ```
//! 
//! ### Syntax
//! 
//! Scripts are based on the standard syntax made up of opcodes, base-10 integers, or hex string literals. Additionally, Rust expressions can be interpolated in order to support dynamically capturing Rust variables or computing values (delimited by `<angle brackets>`).
//! 
//! Whitespace is ignored - scripts can be formatted in the author's preferred style.
//! 
//! #### Opcodes
//! 
//! All normal opcodes are available, in the form `OP_X`.
//! 
//! ```rust
//! let script = bitcoin_script!(OP_CHECKSIG OP_VERIFY);
//! ```
//! 
//! #### Integer Literals
//! 
//! Positive and negative 64-bit integer literals can be used, and will resolve to their most efficient encoding.
//! 
//! For example:
//!     - `2` will resolve to `OP_PUSHNUM_2` (`0x52`)
//!     - `255` will resolve to a length-delimited varint: `0x02ff00` (note the extra zero byte, due to the way Bitcoin scripts use the most-significant bit to represent the sign)`
//! 
//! ```rust
//! let script = bitcoin_script!(123 -456 999999);
//! ```
//! 
//! #### Hex Literals
//! 
//! Hex strings can be specified, prefixed with `0x`.
//! 
//! ```rust
//! let script = bitcoin_script!(
//!     0x0102030405060708090a0b0c0d0e0f OP_HASH160
//! );
//! ```
//! 
//! #### Escape Sequences
//! 
//! Dynamic Rust expressions are supported inside the script, surrounded by angle brackets. In many cases, this will just be a variable identifier, but this can also be a function call or arithmetic.
//! 
//! Rust expressions of the following types are supported:
//! 
//! - `i64`
//! - `Vec<u8>`
//! - [`bitcoin::PublicKey`](https://docs.rs/bitcoin/0.23.0/bitcoin/util/key/struct.PublicKey.html)
//! 
//! ```rust
//! let bytes = vec![1, 2, 3];
//! 
//! let script = bitcoin_script! {
//!     <bytes> OP_CHECKSIGVERIFY
//! 
//!     <2016 * 5> OP_CSV
//! };
//! ```

#![feature(proc_macro_hygiene)]

mod generate;
mod parse;

use generate::generate;
use parse::parse;
use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, set_dummy};
use quote::quote;

#[proc_macro]
#[proc_macro_error]
pub fn bitcoin_script(tokens: TokenStream) -> TokenStream {
    set_dummy(quote!((::bitcoin::Script::new())));
    generate(parse(tokens.into())).into()
}
