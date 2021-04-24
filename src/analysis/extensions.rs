//! The Rust compiler library does not support reading the domain during a GenKill analysis.
//! We work around that here by implementing a trait which converts `GenKill<Local>` into `BitSet<Local>`,
//! which exposes the methods we need to be able to propagate the taint.


use rustc_index::bit_set::BitSet;
use rustc_middle::mir::Local;
use rustc_mir::dataflow::GenKill;

pub(crate) trait GenKillBitSetExt {
    fn get_set(&mut self) -> &BitSet<Local>;
    fn propagate(&mut self, old: Local, new: Local);
    fn is_tainted(&mut self, elem: Local) -> bool;
}

impl<T> GenKillBitSetExt for T
where
    T: GenKill<Local>,
{
    fn get_set(&mut self) -> &BitSet<Local> {
        unsafe { &*(self as *mut T as *const BitSet<Local>) }
    }

    fn propagate(&mut self, old: Local, new: Local) {
        if self.is_tainted(old) {
            self.gen(new);
        } else {
            self.kill(new);
        }
    }

    fn is_tainted(&mut self, elem: Local) -> bool {
        let set = self.get_set();
        set.contains(elem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: Local = Local::from_u32(1);
    const TWO: Local = Local::from_u32(2);
    const THREE: Local = Local::from_u32(3);

    #[test]
    fn propagate() {
        /// We're testing the use of propagate in a setting similar to
        /// the one where we implement the transfer functions.
        fn propagate_inner<T: GenKill<Local>>(domain: &mut T) {
            // Domain
            // [_1, _2]
            domain.propagate(ONE, TWO);

            // Domain
            // [_2]
            domain.propagate(THREE, ONE);
        }

        let mut set: BitSet<Local> = BitSet::new_empty(4);

        // Taint the first element
        set.insert(ONE);

        // Propagate the taint through the domain
        propagate_inner(&mut set);

        // TWO should be tainted.
        assert!(set.contains(TWO));

        // One should not be tainted.
        assert!(!set.contains(ONE));
    }
}
