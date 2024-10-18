#![no_std]
#![cfg_attr(not(test), no_main)]

#[cfg(test)]
extern crate alloc;

#[cfg(not(test))]
ckb_std::entry!(program_entry);
#[cfg(not(test))]
ckb_std::default_alloc!(4 * 1024, 1024 * 1024, 64);
const MESSAGE: &'static [u8] = b"hello, world!";

mod ecc;
mod rsa;
mod utils;
mod rsa2;
mod rsa2_linkable;

pub fn program_entry() -> i8 {
    let mut buf = [0u8; 1032260];
    let got_len =
        ckb_std::syscalls::load_cell_data(&mut buf, 0, 0, ckb_std::ckb_constants::Source::Input)
            .unwrap();
    ckb_std::debug!("Read data {}", got_len);
    // check_hash_rsa(MESSAGE).unwrap();
    // ecc::check_hash_ecc(&MESSAGE, &buf).unwrap();
    rsa2_linkable::check_hash_rsa2(&MESSAGE, &buf).unwrap();

    0
}

#[test]
fn test_ecc(){
    let ecc_data = include_bytes!("../sign-ecc-10.bin");
    ecc::check_hash_ecc(&MESSAGE, &ecc_data[..]).unwrap();

}

// #[test]
// fn test_rsa() {
//     let rsa_data = include_bytes!("../sign13.bin");
//     rsa::check_hash_rsa(&MESSAGE, &rsa_data[..]).unwrap();
// }
