#![feature(proc_macro_hygiene)]

use bitcoin_script::script;

#[test]
fn it_works() {
    let script = script! {
        OP_CHECKSIGVERIFY <sig + lol>
        OP_HASH160
        1234
        -1
    };

    println!("{:?}", script);
}
