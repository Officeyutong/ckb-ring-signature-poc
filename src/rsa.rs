use aes::cipher::{BlockEncryptMut, KeyInit};
use bnum::BUint;
use ckb_std::syscalls::current_cycles;

use crate::utils::power_mod;

type Uint2048 = BUint<32>;
// type Uint4096 = BUint<64>;

#[allow(unused)]
pub fn check_hash_rsa(message: &[u8], buf: &[u8]) -> Result<(), &'static str> {
    let aes_key = lhash::sha256(message);

    let v = { Uint2048::from_le_slice(&buf[0..256]).unwrap() };

    let n = {
        let buf = &buf[256..];

        u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize
    };
    ckb_std::debug!("n={}", n);
    let mut aes_inst = ecb::Encryptor::<aes::Aes256>::new((&aes_key).into());
    let mut equation = v;

    for i in 0..n {
        ckb_std::debug!("i={}", i);
        let start_offset = 256 + 4 + i * (4 + 256 + 256);
        let e = {
            let buf = &buf[start_offset..];

            u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]])
        };

        let n = {
            let buf = &buf[start_offset + 4..];
            Uint2048::from_le_slice(&buf[..256]).unwrap()
        };
        let x = {
            let buf = &buf[start_offset + 4 + 256..];

            Uint2048::from_le_slice(&buf[..256]).unwrap()
        };
        let c1 = current_cycles();
        let y = unsafe { power_mod::<32, 64>(x, e, n) };
        equation = equation ^ y;
        let c2 = current_cycles();
        let input_block = equation.digits_mut();
        for i in (0..input_block.len()).step_by(2) {
            let ptr = &mut input_block[i] as *mut u64 as *mut u8;

            aes_inst.encrypt_block_mut(unsafe { core::slice::from_raw_parts_mut(ptr, 16) }.into());
        }
        let c3 = current_cycles();
        ckb_std::debug!("rsa={}, aes={}", c2 - c1, c3 - c2);
    }
    if equation != v {
        return Err("Bad signature");
    }
    Ok(())
}
