use super::utils::ToHashSet;
use super::{parse, Navigation};
use clingo::{SolverLiteral, Symbol};
use std::collections::HashSet;

use super::Navigator;

/// TODO
pub fn bc(
    nav: &mut impl FacetedNavigation,
    peek_on: impl Iterator<Item = String>,
) -> Option<Vec<Symbol>> {
    nav.brave_consequences(peek_on)
}

/// TODO
pub fn cc(
    nav: &mut impl FacetedNavigation,
    peek_on: impl Iterator<Item = String>,
) -> Option<Vec<Symbol>> {
    nav.cautious_consequences(peek_on)
}

/// TODO
pub fn fs<S: ToString>(
    nav: &mut impl FacetedNavigation,
    peek_on: impl Iterator<Item = S>,
) -> Option<HashSet<Symbol>> {
    nav.facets(peek_on)
}

/// TODO
pub fn fs_stats<S: ToString>(
    nav: &mut impl FacetedNavigation,
    peek_on: impl Iterator<Item = S>,
) -> Option<(usize, usize, usize)> {
    nav.stats(peek_on)
}

fn nav_route<S: ToString>(
    state: &mut Navigation,
    peek_on: impl Iterator<Item = S>,
) -> (&mut Navigator, Vec<SolverLiteral>) {
    match state {
        Navigation::And(nav) | Navigation::AndOr(nav) => {
            let mut route = peek_on
                .map(|f| {
                    let s = f.to_string();
                    match s.starts_with("~") {
                        true => parse(&s[1..])
                            .map(|symbol| nav.literals.get(&symbol).map(|l| l.negate()))
                            .flatten(),
                        _ => parse(&s)
                            .map(|symbol| nav.literals.get(&symbol))
                            .flatten()
                            .copied(),
                    }
                })
                .flatten()
                .collect::<Vec<_>>();
            route.extend(nav.conjuncts.0.clone());

            (nav, route)
        }
    }
}

pub trait FacetedNavigation {
    fn brave_consequences<S: ToString>(
        &mut self,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>>;
    fn cautious_consequences<S: ToString>(
        &mut self,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>>;
    fn facets<S: ToString>(&mut self, peek_on: impl Iterator<Item = S>) -> Option<HashSet<Symbol>>;
    fn stats<S: ToString>(
        &mut self,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<(usize, usize, usize)>;
}
impl FacetedNavigation for Navigation {
    fn brave_consequences<S: ToString>(
        &mut self,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>> {
        let (mut nav, route) = nav_route(self, peek_on);

        consequences(Consequences::Brave, &mut nav, &route)
    }

    fn cautious_consequences<S: ToString>(
        &mut self,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>> {
        //let (nav, route) = match self {
        //    Self::And(nav) | Self::AndOr(nav) => {
        //        let mut route = read_peek_on(peek_on, &nav);
        //        route.extend(nav.conjuncts.0.clone());

        //        (nav, route)
        //    }
        //};
        let (mut nav, route) = nav_route(self, peek_on);

        consequences(Consequences::Cautious, &mut nav, &route)
    }

    fn facets<S: ToString>(&mut self, peek_on: impl Iterator<Item = S>) -> Option<HashSet<Symbol>> {
        let (mut nav, route) = nav_route(self, peek_on);

        let bcs = consequences(Consequences::Brave, &mut nav, &route)?;

        match !bcs.is_empty() {
            true => consequences(Consequences::Cautious, &mut nav, &route)
                .as_ref()
                .and_then(|ccs| Some(bcs.difference_as_set(ccs))),
            _ => Some(bcs.to_hashset()),
        }
    }

    fn stats<S: ToString>(
        &mut self,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<(usize, usize, usize)> {
        let (mut nav, route) = nav_route(self, peek_on);

        let bcs = consequences(Consequences::Brave, &mut nav, &route)?;
        match !bcs.is_empty() {
            true => {
                let ccs = consequences(Consequences::Cautious, &mut nav, &route)?;
                Some((
                    bcs.len(),
                    ccs.len(),
                    bcs.to_hashset().difference(&ccs.to_hashset()).count(),
                ))
            }
            _ => Some((0, 0, 0)),
        }
    }
}

fn consequences(
    kind: impl BCCC,
    nav: &mut Navigator,
    route: &[SolverLiteral],
) -> Option<Vec<Symbol>> {
    kind.consequences(nav, route)
}

enum Consequences {
    Brave,
    Cautious,
}
trait BCCC {
    fn consequences(&self, nav: &mut Navigator, route: &[SolverLiteral]) -> Option<Vec<Symbol>>;
}
impl BCCC for Consequences {
    fn consequences(&self, nav: &mut Navigator, route: &[SolverLiteral]) -> Option<Vec<Symbol>> {
        let s = match self {
            Self::Brave => "brave",
            Self::Cautious => "cautious",
        };
        nav.ctl
            .configuration_mut()
            .map(|c| {
                c.root()
                    .and_then(|rk| c.map_at(rk, "solve.enum_mode"))
                    .map(|sk| c.value_set(sk, s))
                    .ok()
            })
            .ok()?;

        let mut xs = vec![];
        let mut handle = nav.ctl.fasb_solve(clingo::SolveMode::YIELD, &route).ok()?;

        while let Ok(Some(ys)) = handle.model() {
            xs = ys.symbols(clingo::ShowType::SHOWN).ok()?;
            handle.resume().ok()?;
        }

        nav.ctl
            .configuration_mut()
            .map(|c| {
                c.root()
                    .and_then(|rk| c.map_at(rk, "solve.enum_mode"))
                    .map(|sk| c.value_set(sk, "auto"))
                    .ok()
            })
            .ok()?;

        Some(xs)
    }
}
