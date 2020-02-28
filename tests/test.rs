#![feature(proc_macro_hygiene)]

use bitcoin_script::script;

#[test]
fn it_works() {
    let script = script! {
        <prefix x>
        OP_CHECKSIGVERIFY <sig>
        OP_HASH160
        123
        0x10000000000000000000000000000000010000000000000000000000000000000
    };

    println!("{:?}", script);
}
