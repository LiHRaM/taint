//! A trait to constrain the domain operations to taint analysis.

use std::collections::HashSet;

use rustc_index::{bit_set::BitSet, Idx};
use rustc_middle::mir::{Local, Place};
use tracing::instrument;

use crate::taint_analysis::PointsMap;

#[derive(Debug)]
pub(crate) struct PointsAwareTaintDomain<'a, T: Idx> {
    pub(crate) state: &'a mut BitSet<T>,
    pub(crate) map: &'a mut PointsMap,
}

pub(crate) trait TaintDomain<T: Idx> {
    fn propagate(&mut self, old: T, new: T);
    fn get_taint(&self, elem: T) -> bool;
    fn set_taint(&mut self, ix: T, value: bool);
}

impl<T: Idx> TaintDomain<T> for BitSet<T> {
    #[instrument]
    fn propagate(&mut self, old: T, new: T) {
        self.set_taint(new, self.get_taint(old));
    }

    #[instrument]
    fn get_taint(&self, elem: T) -> bool {
        self.contains(elem)
    }

    #[instrument]
    fn set_taint(&mut self, ix: T, taint: bool) {
        if taint {
            self.insert(ix);
        } else {
            self.remove(ix);
        }
    }
}

impl TaintDomain<Local> for PointsAwareTaintDomain<'_, Local> {
    fn propagate(&mut self, old: Local, new: Local) {
        self.set_taint(new, self.get_taint(old));
    }

    fn get_taint(&self, ix: Local) -> bool {
        self.state.get_taint(ix)
    }

    fn set_taint(&mut self, ix: Local, value: bool) {
        let children = self.get_aliases(ix);

        for child in children {
            self.state.set_taint(child, value);
        }
    }
}

impl PointsAwareTaintDomain<'_, Local> {
    pub(crate) fn add_ref(&mut self, from: &Place, to: &Place) {
        let set = self.map.entry(from.local).or_default();
        set.insert(to.local);
    }

    fn get_aliases(&mut self, ix: Local) -> HashSet<Local> {
        let mut result = HashSet::new();
        result.insert(ix);
        let mut previous_size = result.len();

        loop {
            for (key, set) in self.map.iter() {
                if result.contains(key) {
                    for l in set.iter() {
                        result.insert(*l);
                    }
                }
            }

            let current_size = result.len();
            if previous_size != current_size {
                previous_size = current_size;
            } else {
                break;
            }
        }

        result
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
        set.set_taint(ONE, true);

        // Propagate the taint through the domain

        // Domain
        // [_1, _2]
        set.propagate(ONE, TWO);

        // Domain
        // [_2]
        set.propagate(THREE, ONE);

        // TWO should be tainted.
        assert!(set.get_taint(TWO));

        // One should not be tainted.
        assert!(!set.get_taint(ONE));
    }
}
