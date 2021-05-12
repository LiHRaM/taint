//! A trait to constrain the domain operations to taint analysis.

use rustc_index::{bit_set::BitSet, vec::Idx};
use tracing::instrument;

pub(crate) trait TaintDomain<T: Idx> {
    fn propagate(&mut self, old: T, new: T);
    fn is_tainted(&self, elem: T) -> bool;
    fn mark_tainted(&mut self, ix: T);
    fn mark_untainted(&mut self, ix: T);
}

impl<T: Idx> TaintDomain<T> for BitSet<T> {
    #[instrument]
    fn propagate(&mut self, old: T, new: T) {
        if self.is_tainted(old) {
            self.mark_tainted(new);
        } else {
            self.mark_untainted(new);
        }
    }

    #[instrument]
    fn is_tainted(&self, elem: T) -> bool {
        self.contains(elem)
    }

    #[instrument]
    fn mark_tainted(&mut self, ix: T) {
        self.insert(ix);
    }

    #[instrument]
    fn mark_untainted(&mut self, ix: T) {
        self.remove(ix);
    }
}

#[cfg(test)]
mod tests {
    use rustc_middle::mir::Local;

    use super::*;

    const ONE: Local = Local::from_u32(1);
    const TWO: Local = Local::from_u32(2);
    const THREE: Local = Local::from_u32(3);

    #[test]
    fn propagate() {
        let mut set: BitSet<Local> = BitSet::new_empty(4);

        // Taint the first element
        set.mark_tainted(ONE);

        // Propagate the taint through the domain

        // Domain
        // [_1, _2]
        set.propagate(ONE, TWO);

        // Domain
        // [_2]
        set.propagate(THREE, ONE);

        // TWO should be tainted.
        assert!(set.is_tainted(TWO));

        // One should not be tainted.
        assert!(!set.is_tainted(ONE));
    }
}
