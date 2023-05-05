use super::utils::ToHashSet;
use super::{parse, Navigation};
use clingo::{SolverLiteral, Symbol};
use std::collections::HashSet;

use super::Navigator;

pub fn s_g(
    nav: &mut impl Soe,
    peek_on: impl Iterator<Item = String>,
    t: impl Iterator<Item = String>,
) {
    unimplemented!()
}

pub trait Soe {
    fn s_greedy(
        nav: &mut impl FacetedNavigation,
        peek_on: impl Iterator<Item = String>,
        t: impl Iterator<Item = String>;
    );
}
impl Soe for Navigation {
    fn s_greedy(
        nav: &mut impl FacetedNavigation,
        peek_on: impl Iterator<Item = String>,
        t: impl Iterator<Item = String>;
    ) {
        unimplemented!()
    }
}
