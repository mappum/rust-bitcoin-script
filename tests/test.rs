#![feature(proc_macro_hygiene)]

use bitcoin_script::script;

#[test]
fn it_works() {
    let sig = vec![88; 32];
    let script = script! {
        OP_CHECKSIGVERIFY <sig>
        OP_HASH160
        0x1234
        -1
    };

    println!("script: {:?}\nbytes: {:?}", script, script.to_bytes());
}
