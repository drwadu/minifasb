use super::utils::ToHashSet;
use super::{parse, Navigation};
use clingo::{SolverLiteral, Symbol};
use std::collections::HashSet;

use super::Navigator;

/// TODO
pub fn bc(
    nav: &mut impl FacetedNavigation,
    peek_on: (impl Iterator<Item = String>, impl Iterator<Item = String>),
) -> Option<Vec<Symbol>> {
    nav.brave_consequences(peek_on)
}

/// TODO
pub fn cc(
    nav: &mut impl FacetedNavigation,
    peek_on: (impl Iterator<Item = String>, impl Iterator<Item = String>),
) -> Option<Vec<Symbol>> {
    nav.cautious_consequences(peek_on)
}

/// TODO
pub fn fs<S: ToString>(
    nav: &mut impl FacetedNavigation,
    peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
) -> Option<HashSet<Symbol>> {
    nav.facets(peek_on)
}

/// TODO
pub fn fs_stats<S: ToString>(
    nav: &mut impl FacetedNavigation,
    peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
) -> Option<(usize, usize, usize)> {
    nav.stats(peek_on)
}

pub trait FacetedNavigation {
    fn brave_consequences<S: ToString>(
        &mut self,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
    ) -> Option<Vec<Symbol>>;
    fn cautious_consequences<S: ToString>(
        &mut self,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
    ) -> Option<Vec<Symbol>>;
    fn facets<S: ToString>(
        &mut self,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
    ) -> Option<HashSet<Symbol>>;
    fn stats<S: ToString>(
        &mut self,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
    ) -> Option<(usize, usize, usize)>;
}
impl FacetedNavigation for Navigation {
    fn brave_consequences<S: ToString>(
        &mut self,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
    ) -> Option<Vec<Symbol>> {
        let (nav, route) = match self {
            Self::And(nav) => {
                let mut route = peek_on
                    .0
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
                route.extend(nav.route.1.clone());

                (nav, route)
            }
            Self::AndOr(nav) => {
                let route = peek_on
                    .1
                    .map(|f| {
                        let s = f.to_string();
                        match s.starts_with("~") {
                            true => parse(&s[1..]),
                            _ => parse(&s),
                        }
                    })
                    .flatten()
                    .collect::<Vec<_>>();
                if !route.is_empty() {
                    nav.or_delta(route.iter());
                }
                let mut route = peek_on
                    .0
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
                route.extend(nav.route.1.clone());

                (nav, route)
            }
        };

        consequences(Consequences::Brave, nav, &route)
    }

    fn cautious_consequences<S: ToString>(
        &mut self,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
    ) -> Option<Vec<Symbol>> {
        let (nav, route) = match self {
            Self::And(nav) => {
                let mut route = peek_on
                    .0
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
                route.extend(nav.route.1.clone());

                (nav, route)
            }
            Self::AndOr(nav) => {
                let route = peek_on
                    .1
                    .map(|f| {
                        let s = f.to_string();
                        match s.starts_with("~") {
                            true => parse(&s[1..]),
                            _ => parse(&s),
                        }
                    })
                    .flatten()
                    .collect::<Vec<_>>();
                if !route.is_empty() {
                    nav.or_delta(route.iter());
                }
                let mut route = peek_on
                    .0
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
                route.extend(nav.route.1.clone());

                (nav, route)
            }
        };

        consequences(Consequences::Cautious, nav, &route)
    }

    fn facets<S: ToString>(
        &mut self,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
    ) -> Option<HashSet<Symbol>> {
        let (nav, route) = match self {
            Self::And(nav) => {
                let mut route = peek_on
                    .0
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
                route.extend(nav.route.1.clone());
                (nav, route)
            }
            Self::AndOr(nav) => {
                let route = peek_on
                    .1
                    .map(|f| {
                        let s = f.to_string();
                        match s.starts_with("~") {
                            true => parse(&s[1..]),
                            _ => parse(&s),
                        }
                    })
                    .flatten()
                    .collect::<Vec<_>>();
                if !route.is_empty() {
                    nav.or_delta(route.iter());
                }
                let mut route = peek_on
                    .0
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
                route.extend(nav.route.1.clone());

                (nav, route)
            }
        };

        let bcs = consequences(Consequences::Brave, nav, &route)?;
        match !bcs.is_empty() {
            true => consequences(Consequences::Cautious, nav, &route)
                .as_ref()
                .and_then(|ccs| Some(bcs.difference_as_set(ccs))),
            _ => Some(bcs.to_hashset()),
        }
    }

    fn stats<S: ToString>(
        &mut self,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
    ) -> Option<(usize, usize, usize)> {
        let (nav, route) = match self {
            Self::And(nav) => {
                let mut route = peek_on
                    .0
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
                route.extend(nav.route.1.clone());
                (nav, route)
            }
            Self::AndOr(nav) => {
                let route = peek_on
                    .1
                    .map(|f| {
                        let s = f.to_string();
                        match s.starts_with("~") {
                            true => parse(&s[1..]),
                            _ => parse(&s),
                        }
                    })
                    .flatten()
                    .collect::<Vec<_>>();
                if !route.is_empty() {
                    nav.or_delta(route.iter());
                }
                let mut route = peek_on
                    .0
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
                route.extend(nav.route.1.clone());

                (nav, route)
            }
        };

        let bcs = consequences(Consequences::Brave, nav, &route)?;
        match !bcs.is_empty() {
            true => {
                let ccs = consequences(Consequences::Cautious, nav, &route)?;
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
