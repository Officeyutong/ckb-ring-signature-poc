use alloc::vec::Vec;
use bnum::{cast::As, BUint};
use ckb_std::syscalls::current_cycles;

use crate::utils::{inverse_mod, CycleMeasure};

type Uint256 = BUint<4>;
type Uint512 = BUint<8>;
const P: Uint256 = Uint256::parse_str_radix(
    "115792089237316195423570985008687907853269984665640564039457584007908834671663",
    10,
);

const P512: Uint512 = Uint512::parse_str_radix(
    "115792089237316195423570985008687907853269984665640564039457584007908834671663",
    10,
);
#[allow(unused)]
const N: Uint256 = Uint256::parse_str_radix(
    "115792089237316195423570985008687907852837564279074904382605163141518161494337",
    10,
);
const G: Point = Point(
    Uint256::parse_str_radix(
        "55066263022277343669578718895168534326250603453777594175500187360389116729240",
        10,
    ),
    Uint256::parse_str_radix(
        "32670510020758816978083085130507043184471273380659243275938904335757337482424",
        10,
    ),
);
#[derive(PartialEq, Clone)]
struct Point(Uint256, Uint256);

unsafe fn point_add(p1: Point, p2: Point) -> Point {
    // let _ = CycleMeasure::new("point add");
    ckb_std::debug!("Enter point add {}", current_cycles());
    let Point(x1, y1) = p1;
    let Point(x2, y2) = p2;

    let x1_512 = x1.as_::<Uint512>();
    let x2_512 = x2.as_::<Uint512>();

    let y1_512 = y1.as_::<Uint512>();
    let y2_512 = y2.as_::<Uint512>();

    if x1.is_zero() && y1.is_zero() {
        ckb_std::debug!("Exit point add {}", current_cycles());
        return Point(x2, y2);
    }
    if x2.is_zero() && y2.is_zero() {
        ckb_std::debug!("Exit point add {}", current_cycles());
        return Point(x1, y1);
    }
    if x1 == x2 && ((y1_512 + y2_512) % P512).is_zero() {
        ckb_std::debug!("Exit point add {}", current_cycles());
        return Point(Uint256::ZERO, Uint256::ZERO);
    }

    ckb_std::debug!("initialized point add {}", current_cycles());
    let k = if x1 != x2 || y1 != y2 {
        ((y1_512 + P512 - y2_512) % P512)
            * inverse_mod::<4, 8>(((x1_512 + P512 - x2_512) % P512).as_(), P).as_::<Uint512>()
            % P512
    } else {
        Uint512::THREE * x1_512 % P512 * x1_512 % P512
            * (inverse_mod::<4, 8>((y1_512 * Uint512::TWO % P512).as_(), P)).as_::<Uint512>()
            % P512
    };

    ckb_std::debug!("getK point add {}", current_cycles());
    let x3 = (k.unchecked_mul(k) % P512 + P512 - x1_512 + P512 - x2_512) % P512;
    let y3 = (k.unchecked_mul((x1_512 + P512 - x3) % P512) % P512 + P512 - y1_512) % P512;

    ckb_std::debug!("Exit point add {}", current_cycles());
    Point(x3.as_(), y3.as_())
}

fn uint_to_bytes<const S: usize>(num: &BUint<S>, out: &mut [u8]) {
    let digits = num.digits();
    for i in 0..S {
        let curr = digits[i];
        out[i * 8..(i + 1) * 8].copy_from_slice(&curr.to_le_bytes());
    }
}

fn point_to_bytes(pnt: &Point, out: &mut [u8]) {
    uint_to_bytes(&pnt.0, &mut out[0..32]);
    uint_to_bytes(&pnt.1, &mut out[32..64]);
}

fn compound_hash(message: &[u8], pub_keys: &[Point], extra_point: &Point) -> [u8; 32] {
    let _ = CycleMeasure::new("compound hash");
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(message);
    let mut local_buf = [0u8; 64];
    for pnt in pub_keys {
        point_to_bytes(pnt, &mut local_buf);
        buf.extend_from_slice(&local_buf);
    }
    point_to_bytes(extra_point, &mut local_buf);
    buf.extend_from_slice(&local_buf);
    return lhash::sha256(&buf);
}

unsafe fn kmul(mut k: Uint256, mut point: Point) -> Point {
    let mut result = Point(BUint::ZERO, BUint::ZERO);
    ckb_std::debug!("enter kmul cycle={}", current_cycles());
    while !k.is_zero() {
        ckb_std::debug!("enter kmul one loop cycle={}", current_cycles());
        // let _ = CycleMeasure::new("kmul one loop");
        if (k.bitand(Uint256::ONE)).is_one() {
            result = point_add(result, point.clone());
        }
        ckb_std::debug!("kmul one loop after bit check cycle={}", current_cycles());
        point = point_add(point.clone(), point);
        ckb_std::debug!("kmul one loop after point add cycle={}", current_cycles());
        k = k.shr(1);
        ckb_std::debug!("exit kmul one loop cycle={}", current_cycles());
    }
    ckb_std::debug!("exit kmul cycle={}", current_cycles());
    result
}
#[allow(unused)]
pub fn check_hash_ecc(message: &[u8], signature: &[u8]) -> Result<(), &'static str> {
    let buf = signature;

    let n = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    let c0_bytes = &buf[4..4 + 32];
    let c0 = Uint256::from_le_slice(c0_bytes).unwrap();
    let mut r = Vec::<Uint256>::with_capacity(n);
    let mut c = Vec::<Uint256>::with_capacity(n);
    let mut pub_keys = Vec::<Point>::with_capacity(n);
    let start_offset = 4 + 32;
    for i in 0..n {
        let curr_sec_off = start_offset + i * (32 + 32 + 32);
        r.push(Uint256::from_le_slice(&buf[curr_sec_off..curr_sec_off + 32]).unwrap());
        let p1 = Uint256::from_le_slice(&buf[curr_sec_off + 32..curr_sec_off + 32 * 2]).unwrap();
        let p2 =
            Uint256::from_le_slice(&buf[curr_sec_off + 32 * 2..curr_sec_off + 32 * 3]).unwrap();
        pub_keys.push(Point(p1, p2));
    }
    c.resize(n, Uint256::ZERO);
    // c.push(c0.clone());
    let cyc0 = current_cycles();
    ckb_std::debug!("current cycle {}", cyc0);
    c[0] = c0;
    for i in 0..n {
        ckb_std::debug!("curr i={}", i);
        let pub_key = &pub_keys[i];
        let rgck = unsafe { point_add(kmul(r[i], G), kmul(c[i], pub_key.clone())) };
        c[(i + 1) % n] = Uint256::from_le_slice(&compound_hash(message, &pub_keys, &rgck)).unwrap();
    }
    ckb_std::debug!("received c0={}, calculated c0={}", c0, c[0]);
    if c0 != c[0] {
        return Err("Bad signature");
    }
    Ok(())
}
