use bnum::BUint;
use ckb_std::syscalls::current_cycles;
use sha2::{Digest, Sha256};

use crate::utils::{add_mod_expand, mul_mod_expand, power_mod};

type Uint2048 = BUint<32>;

fn sha256_for_integer(num: &Uint2048) -> Uint2048 {
    let mut hasher = Sha256::new();
    for digit in num.digits() {
        hasher.update(unsafe { core::slice::from_raw_parts(digit as *const u64 as *const u8, 8) });
    }
    Uint2048::from_le_slice(&hasher.finalize()).unwrap()
}

pub fn check_hash_rsa2<T: Fn(usize) -> ()>(
    message: &[u8],
    buf: &[u8],
    progress_callback: Option<T>,
) -> Result<(), &'static str> {
    let n_size = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    ckb_std::debug!("n={}", n_size);
    let c0 = Uint2048::from_le_slice(&buf[4..4 + 256]).unwrap();
    let image = Uint2048::from_le_slice(&buf[4 + 256..4 + 256 + 256]).unwrap();

    let mut hasher = Sha256::new();
    hasher.update(message);
    for i in 0..n_size {
        ckb_std::debug!("read pub key of {}", i);
        let start_offset = 4 + 256 + 256 + i * (256 + 4 + 256);
        hasher.update(&buf[start_offset + 256 + 4..start_offset + 256 + 4 + 256]);
    }
    let compund_hash = |integer1: &Uint2048, integer2: &Uint2048| -> Uint2048 {
        let mut local_hasher = hasher.clone();
        for digit in integer1.digits().iter().chain(integer2.digits().iter()) {
            local_hasher.update(unsafe {
                core::slice::from_raw_parts(digit as *const u64 as *const u8, 8)
            });
        }
        Uint2048::from_le_slice(&local_hasher.finalize()).unwrap()
    };
    let mut last_c = c0;
    for i in 0..n_size {
        if let Some(ref f) = progress_callback {
            f(i);
        }
        let start_offset = 4 + 256 + 256 + i * (256 + 4 + 256);
        let r = Uint2048::from_le_slice(&buf[start_offset..start_offset + 256]).unwrap();

        let e_bytes = &buf[start_offset + 256..start_offset + 256 + 4];
        let e = u32::from_le_bytes([e_bytes[0], e_bytes[1], e_bytes[2], e_bytes[3]]);
        let n = Uint2048::from_le_slice(&buf[start_offset + 256 + 4..start_offset + 256 + 4 + 256])
            .unwrap();
        ckb_std::debug!("e={}, cycle={}", e, current_cycles());
        let rcpe = power_mod::<32, 64>(mul_mod_expand::<32, 64>(r, last_c, n.clone()), e as u64, n);
        ckb_std::debug!("rsa, cycle={}", current_cycles());
        // c = CH(rcpe, (rcpe) ^ {e} ^ {I} * H)
        last_c = compund_hash(
            &rcpe,
            &mul_mod_expand::<32, 64>(
                // power_mod_buint::<32, 64>(
                //     power_mod::<32, 64>(rcpe, e as u64, n),
                //     image.clone(),
                //     n.clone(),
                // ),
                add_mod_expand::<32, 33>(rcpe, image, n),
                sha256_for_integer(&n),
                n,
            ),
        );
        ckb_std::debug!("hash, cycle={}", current_cycles());
    }

    if last_c != c0 {
        return Err("Bad signature");
    }
    Ok(())
}
