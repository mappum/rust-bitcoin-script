#![feature(proc_macro_hygiene)]

use bitcoin_script::bitcoin_script;

#[test]
fn fixture() {
    let foo = vec![1, 2, 3, 4];
    let script = bitcoin_script! {
        OP_HASH160
        1234
        255
        -1
        -255
        0xabcd
        <1 + 1>
        <foo>
    };

    assert_eq!(
        script.to_bytes(),
        vec![169, 2, 210, 4, 2, 255, 0, 79, 2, 255, 128, 2, 171, 205, 82, 4, 1, 2, 3, 4]
    );
}
