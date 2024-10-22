use bnum::{cast::As, BInt, BUint};
use ckb_std::syscalls::current_cycles;

pub fn mul_mod_expand<const S: usize, const S2: usize>(
    a: BUint<S>,
    b: BUint<S>,
    p: BUint<S>,
) -> BUint<S> {
    let c = (a.as_::<BUint<S2>>()) * (b.as_::<BUint<S2>>());
    let result = c % p.as_::<BUint<S2>>();
    result.as_()
}

pub fn add_mod_expand<const S: usize, const S2: usize>(
    a: BUint<S>,
    b: BUint<S>,
    p: BUint<S>,
) -> BUint<S> {
    let c = (a.as_::<BUint<S2>>()) + (b.as_::<BUint<S2>>());
    let result = c % p.as_::<BUint<S2>>();
    result.as_()
}

pub unsafe fn mul_mod<const S: usize>(a: BUint<S>, b: BUint<S>, p: BUint<S>) -> BUint<S> {
    let c: BUint<S> = (a).unchecked_mul(b);
    let result = c % p;
    result
}

pub fn power_mod<const S: usize, const S2: usize>(
    base: BUint<S>,
    mut index: u64,
    modular: BUint<S>,
) -> BUint<S> {
    let mut base: BUint<S2> = base.as_();
    let mut result = BUint::<S2>::ONE;
    let modular: BUint<S2> = modular.as_();

    while index != 0 {
        if (index & 1) == 1 {
            result = unsafe { mul_mod::<S2>(result, base, modular) };
        }
        base = unsafe { mul_mod::<S2>(base, base, modular) };
        index >>= 1;
    }
    result.as_()
}
#[allow(unused)]
pub fn power_mod_buint<const S: usize, const S2: usize>(
    base: BUint<S>,
    mut index: BUint<S>,
    modular: BUint<S>,
) -> BUint<S> {
    // let _ = CycleMeasure::new("power mod buint");
    // ckb_std::debug!("Enter powet buint {}", current_cycles());
    let mut result: BUint<S2> = BUint::<S2>::ONE;
    let mut base = base.as_::<BUint<S2>>();
    let modular = modular.as_::<BUint<S2>>();

    while !index.is_zero() {
        // ckb_std::debug!("enter power mod buint loop");
        if (index.bitand(BUint::<S>::ONE)).is_one() {
            result = unsafe { mul_mod::<S2>(result, base, modular) };
        }
        base = unsafe { mul_mod::<S2>(base, base, modular) };
        index = unsafe { index.unchecked_shr(1) };
    }

    // ckb_std::debug!("Exit powet buint {}", current_cycles());
    result.as_()
}

fn extended_gcd<const S: usize>(a: BInt<S>, b: BInt<S>) -> (BInt<S>, BInt<S>, BInt<S>) {
    if b.is_zero() {
        (a, BInt::<S>::ONE, BInt::<S>::ZERO)
    } else {
        let (gcd, x1, y1) = extended_gcd(b, a % b);
        (gcd, y1, x1 - (a / b) * y1)
    }
}

pub fn inverse_mod<const S: usize, const S2: usize>(a: BUint<S>, p: BUint<S>) -> BUint<S> {
    let (_, x, _) = extended_gcd(a.as_(), p.as_());
    if x.is_negative() {
        let signed_p = p.as_::<BInt<S>>();
        ((x + signed_p) % signed_p).as_()
    } else {
        x.as_()
    }
}
pub struct CycleMeasure(u64, &'static str);
impl Drop for CycleMeasure {
    fn drop(&mut self) {
        let c = current_cycles();
        ckb_std::debug!(
            "{}: used cycles={}, begin {}, end {}",
            self.1,
            c - self.0,
            self.0,
            c
        );
    }
}
impl CycleMeasure {
    pub fn new(name: &'static str) -> Self {
        Self(current_cycles(), name)
    }
}
#[allow(unused)]
pub fn multi_step_power<const S: usize, const S2: usize>(
    a: BUint<S>,
    b: BUint<S>,
    p: BUint<S>,
) -> BUint<S> {
    let mut result = BUint::<S>::ONE;
    // c = 2^{64}
    let power_of_c = |x: BUint<S>| -> BUint<S> {
        let y = power_mod::<S, S2>(x.clone(), ((1u128 << 64) - 1) as u64, p.clone());
        mul_mod_expand::<S, S2>(y, x, p)
    };
    let mut a_power_c_power_k = a.clone();

    for digit in b.digits() {
        result =
            mul_mod_expand::<S, S2>(result, power_mod::<S, S2>(a_power_c_power_k, *digit, p), p);

        a_power_c_power_k = power_of_c(a_power_c_power_k);
    }
    result
}
