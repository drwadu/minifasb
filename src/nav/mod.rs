pub mod errors;
pub mod faceted_navigation;
mod utils;
pub mod weighted_navigation;

use crate::lex::*;

use errors::Result;

use clingo::{Control, SolverLiteral, Symbol};
use std::collections::{HashMap, HashSet};

/// Pretty prints route.
pub fn show_route(nav: &impl Essential) {
    nav.route_repr()
}

/// Clears current route, setting route to empty route.
pub fn clear_route(nav: &mut impl Essential) -> Result<()> {
    nav.clear()
}

/// Activates a facets according to specified `route`.
pub fn delta<S: ToString>(nav: &mut impl Essential, route: impl Iterator<Item = S>) {
    nav.delta(route)
}

/// Enumerate `n` answer sets under current route conjunctively extended by `peek_on`.
pub fn enumerate_solutions<S: ToString>(
    nav: &mut impl Essential,
    n: usize,
    peek_on: impl Iterator<Item = S>,
) -> Result<()> {
    nav.solutions(n, peek_on)
}

#[derive(Debug, Clone)]
struct FacetRepr(String);

pub struct Navigator {
    /// Clingo solver.
    ctl: Control,
    /// Conjuctively activated facets
    conjuncts: (Vec<SolverLiteral>, Vec<FacetRepr>),
    /// Disjunctively activated facets
    disjuncts: Vec<FacetRepr>,
    /// Active route.
    route: String,
    /// Current facets.
    facets: HashSet<Symbol>,
    /// Literals.
    literals: HashMap<Symbol, SolverLiteral>,
    /// Input program and args.
    input: (String, Vec<String>),
}
impl Navigator {
    pub fn new(source: impl Into<String>, args: Vec<String>) -> Result<Self> {
        let mut ctl = clingo::control(args.clone())?;

        let lp = source.into();
        ctl.add("base", &[], &lp)?;
        ctl.ground(&[clingo::Part::new("base", vec![])?])?;

        let mut literals = HashMap::new();
        for atom in ctl.symbolic_atoms()?.iter()? {
            literals.insert(atom.symbol()?, atom.literal()?);
        }

        Ok(Self {
            ctl,
            conjuncts: (vec![], vec![]),
            disjuncts: vec![],
            //route: (vec![], vec![], vec![]),
            route: "".to_owned(),
            facets: HashSet::default(),
            literals,
            input: (lp, args),
        })
    }

    fn assume(&mut self) -> Result<()> {
        match !self.disjuncts.is_empty() {
            true => {
                let disjunction = self
                    .disjuncts
                    .iter()
                    .map(|f| {
                        let s = &f.0;
                        match s.starts_with("~") {
                            true => s[1..].to_owned(),
                            _ => format!("not {s}"),
                        }
                    })
                    .collect::<Vec<_>>() // TODO: mäh
                    .join(",");
                let lp = format!(
                    "{}\n{}",
                    self.input.0,
                    self.conjuncts
                        .1
                        .iter()
                        .map(|f| {
                            let s = &f.0;
                            let repr = match s.starts_with("~") {
                                true => s[1..].to_owned(),
                                _ => format!("not {s}"),
                            };
                            format!(":- {repr}, {}.", disjunction)
                        })
                        .collect::<Vec<_>>() // TODO: mäh
                        .join("\n")
                );
                dbg!("{:?}", &lp);
                dbg!("{:?}", self.input.1.clone());

                let mut ctl = clingo::control(self.input.1.clone())?;
                ctl.add("base", &[], &lp)?;
                ctl.ground(&[clingo::Part::new("base", vec![])?])?;

                let mut literals = HashMap::new();
                for atom in ctl.symbolic_atoms()?.iter()? {
                    literals.insert(atom.symbol()?, atom.literal()?);
                }

                self.ctl = ctl;

                Ok(())
            }
            _ => self
                .ctl
                .backend()
                .and_then(|mut b| b.assume(&self.conjuncts.0))
                .map_err(|e| errors::NavigatorError::Clingo(e)),
        }
    }

    pub(crate) fn delta<S: ToString>(&mut self, mut delta: impl Iterator<Item = S>) {
        let mut con = true;
        while let Some(token) = delta.next().map(|s| s.to_string()) {
            match token == "&" {
                true => {
                    con = true;
                    self.route = format!("{}{token}", self.route);
                    continue;
                }
                _ => match token == "|" {
                    true => {
                        con = false;
                        self.route = format!("{}{token}", self.route);
                        continue;
                    }
                    _ => {
                        let (symbol, exc) = match token.starts_with('~') {
                            true => (token[1..].to_owned(), true),
                            _ => (token.clone(), false),
                        };
                        match parse(&symbol).as_ref() {
                            Some(sym) => match self.literals.get(sym) {
                                Some(lit) => {
                                    self.route = format!("{}{}", self.route, token.clone());
                                    if con {
                                        match exc {
                                            true => self.conjuncts.0.push(lit.negate()),
                                            _ => self.conjuncts.0.push(*lit),
                                        }
                                        self.conjuncts.1.push(FacetRepr(token))
                                    } else {
                                        self.disjuncts.push(FacetRepr(token))
                                    }
                                }
                                _ => {
                                    eprintln!("ignoring unknown symbol: {symbol}");
        iet delta = "a & c | d".split(" ");
        aonav.delta(delta);
        println!();
        aonav.route_repr();
        println!();
        aonav.solutions(0, std::iter::empty::<String>())?;
        println!();
        aonav.clear()?;
        let delta = "a & c | d".split(" ");
        aonav.delta(delta);
        println!();
        aonav.route_repr();
        println!();
        aonav.solutions(0, std::iter::empty::<String>())?;
        println!();
        aonav.clear()?;
        let delta = "a & c | d".split(" ");
        aonav.delta(delta);
        println!();
        aonav.route_repr();
        println!();
        aonav.solutions(0, std::iter::empty::<String>())?;
        println!();
        aonav.clear()?;
        let delta = "a & c | d".split(" ");
        aonav.delta(delta);
        println!();
        aonav.route_repr();
        println!();
        aonav.solutions(0, std::iter::empty::<String>())?;
        println!();
        aonav.clear()?;
        let delta = "a & c | d".split(" ");
        aonav.delta(delta);
        println!();
        aonav.route_repr();
        println!();
        aonav.solutions(0, std::iter::empty::<String>())?;
        println!();
        aonav.clear()?;
        let delta = "a & c | d".split(" ");
        aonav.delta(delta);
        println!();
        aonav.route_repr();
        println!();
        aonav.solutions(0, std::iter::empty::<String>())?;
        println!();
        aonav.clear()?;
        let delta = "a & c | d".split(" ");
        aonav.delta(delta);
        println!();
        aonav.route_repr();
        println!();
        aonav.solutions(0, std::iter::empty::<String>())?;
        println!();
        aonav.clear()?;
                                    continue;
                                }
                            },
                            _ => {
                                eprintln!("ignoring invalid input: {token}");
                                continue;
                            }
                        }
                    }
                },
            }
        }
        //println!("{:?}", &self.route);
    }
}

pub enum Navigation {
    And(Navigator),
    AndOr(Navigator),
}

pub trait Essential {
    /// Pretty prints route as `| f_0 ... f_n & f_n+1 ... f_m`.
    fn route_repr(&self);
    /// Clears current route, setting route to empty route.
    fn clear(&mut self) -> Result<()>;
    /// Activates all facets in `delta`.
    fn delta<S: ToString>(&mut self, delta: impl Iterator<Item = S>);
    /// Enumerate `n` answer sets under current route conjunctively extended by `peek_on`.
    fn solutions<S: ToString>(&mut self, n: usize, peek_on: impl Iterator<Item = S>) -> Result<()>;
}
impl Essential for Navigation {
    fn route_repr(&self) {
        match &self {
            Self::And(nav) | Self::AndOr(nav) => print!("{}", nav.route),
        }
    }

    fn clear(&mut self) -> Result<()> {
        match self {
            Self::And(nav) => {
                nav.conjuncts.0.clear();
                nav.conjuncts.1.clear();
                nav.route.clear();
                Ok(())
            }
            Self::AndOr(nav) => {
                nav.conjuncts.0.clear();
                nav.conjuncts.1.clear();
                nav.disjuncts.clear();
                nav.route.clear();
                nav.assume()
            }
        }
    }

    fn delta<S: ToString>(&mut self, delta: impl Iterator<Item = S>) {
        match self {
            Self::And(nav) | Self::AndOr(nav) => nav.delta(delta),
        }
    }

    fn solutions<S: ToString>(&mut self, n: usize, peek_on: impl Iterator<Item = S>) -> Result<()> {
        match self {
            Self::And(nav) => {
                let mut route = read_peek_on(peek_on, nav);
                route.extend(nav.conjuncts.0.clone());

                output_answer_sets(nav, &route, n)
            }
            Self::AndOr(nav) => {
                let route = read_peek_on(peek_on, nav);

                nav.assume()?;

                output_answer_sets(nav, &route, n)
            }
        }
    }
}

fn output_answer_sets(nav: &mut Navigator, route: &[SolverLiteral], n: usize) -> Result<()> {
    let mut handle = nav.ctl.fasb_solve(clingo::SolveMode::YIELD, &route)?;
    let mut i = 1;

    dbg!(&nav.conjuncts);
    dbg!(&nav.disjuncts);
    dbg!(route);

    match n == 0 {
        true => {
            while let Ok(Some(answer_set)) = handle.model() {
                println!("Solution {:?}: ", i);
                let atoms = answer_set.symbols(clingo::ShowType::SHOWN)?;
                atoms.iter().for_each(|atom| {
                    print!("{} ", atom.to_string());
                });
                println!();

                i += 1;
                handle.resume()?;
            }
        }
        _ => {
            while let Ok(Some(answer_set)) = handle.model() {
                println!("Solution {:?}: ", i);
                let atoms = answer_set.symbols(clingo::ShowType::SHOWN)?;
                atoms.iter().for_each(|atom| {
                    print!("{} ", atom.to_string());
                });
                println!();

                i += 1;
                if i > n {
                    break;
                }
                handle.resume()?;
            }
        }
    }

    println!("found {:?}", i - 1);

    return handle
        .close()
        .map_err(|e| errors::NavigatorError::Clingo(e));
}

fn read_peek_on<S: ToString>(
    peek_on: impl Iterator<Item = S>,
    nav: &Navigator,
) -> Vec<SolverLiteral> {
    peek_on
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
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup() {
        let nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()]);
        assert!(nav.is_ok());
    }

    #[test]
    fn and_health() -> Result<()> {
        let nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;
        let mut anav = Navigation::And(nav);

        let delta = "b & ~c".split(" ");
        anav.delta(delta);
        println!();
        anav.route_repr();
        println!();
        anav.solutions(0, std::iter::empty::<String>())?;
        println!();
        anav.clear()?;

        let delta = "~a".split(" ");
        anav.delta(delta);
        println!();
        anav.route_repr();
        println!();
        anav.solutions(0, std::iter::empty::<String>())?;
        println!();
        anav.clear()?;

        let delta = "a & b".split(" ");
        anav.delta(delta);
        println!();
        anav.route_repr();
        println!();
        anav.solutions(0, std::iter::empty::<String>())?;
        println!();
        anav.clear()?;

        Ok(())
    }

    #[test]
    fn andor_health() -> Result<()> {
        let nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;
        let mut aonav = Navigation::AndOr(nav);

        let delta = "b | ~c".split(" ");
        //let delta = "a | d | c".split(" ");
        //let delta = "a | ~b".split(" ");
        aonav.delta(delta);
        println!();
        aonav.route_repr();
        println!();
        aonav.solutions(0, std::iter::empty::<String>())?;
        println!();
        aonav.clear()?;

        let delta = "~b | c".split(" ");
        aonav.delta(delta);
        println!();
        aonav.route_repr();
        println!();
        aonav.solutions(0, std::iter::empty::<String>())?;
        println!();
        aonav.clear()?;
        
        let delta = "a | d".split(" ");
        aonav.delta(delta);
        println!();
        aonav.route_repr();
        println!();
        aonav.solutions(0, std::iter::empty::<String>())?;
        println!();
        aonav.clear()?;

        let delta = "a & c | d".split(" ");
        aonav.delta(delta);
        println!();
        aonav.route_repr();
        println!();
        aonav.solutions(0, std::iter::empty::<String>())?;
        println!();
        aonav.clear()?;

        let delta = "a & c | d | a".split(" ");
        aonav.delta(delta);
        println!();
        aonav.route_repr();
        println!();
        aonav.solutions(0, std::iter::empty::<String>())?;
        println!();
        aonav.clear()?;



        //aonav.route_repr();
        //println!();
        //aonav.solutions(0, std::iter::empty::<String>())?;
        //println!();
        //aonav.clear()?;

        /*
        let delta = "~a".split(" ");
        aonav.delta(delta);
        aonav.route_repr();
        aonav.solutions(0, std::iter::empty::<String>())?;
        aonav.clear()?;

        let delta = "a | b".split(" ");
        aonav.delta(delta);
        aonav.route_repr();
        aonav.solutions(0, std::iter::empty::<String>())?;
        aonav.clear()?;

        let delta = "~c & a | b | ~a | ~b & d".split(" ");
        aonav.delta(delta);
        aonav.route_repr();
        aonav.solutions(0, std::iter::empty::<String>())?;
        aonav.clear()?;
        */

        Ok(())
    }

    /*
    #[test]
    fn or_clear() -> Result<()> {
        let mut nav = Navigation::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;
        let delta = vec!["a".to_owned(), "d".to_owned()];
        nav.or_delta(delta.iter())?;
        nav.solutions(0, std::iter::empty())?;

        nav.clear()?;

        let delta = vec!["a".to_owned(), "~e".to_owned()];
        nav.and_delta(delta.iter())?;
        nav.solutions(0, std::iter::empty())?;

        Ok(())
    }

    #[test]
    fn and_or_clear() -> Result<()> {
        let mut nav = Navigation::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;
        let delta = vec!["a".to_owned(), "b".to_owned()];
        nav.or_delta(delta.iter())?; // delta = a | b
        let delta = vec!["~a".to_owned()];
        nav.and_delta(delta.iter())?; // delta = (a | b) & ~a
        let delta = vec!["a".to_owned()];
        nav.and_delta(delta.iter())?; // delta = (a | b) & ~a & a
        println!("(a | b) & ~a & a");
        nav.solutions(0, std::iter::empty())?;
        println!(
            "{:?}",
            nav.facets((std::iter::empty(), std::iter::empty()), None)
                .unwrap()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );

        nav.clear()?;

        let delta = vec!["a".to_owned(), "b".to_owned()];
        nav.or_delta(delta.iter())?; // delta = a | b
        let delta = vec!["~a".to_owned()];
        nav.and_delta(delta.iter())?; // delta = (a | b) & ~a
        let delta = vec!["a".to_owned()];
        nav.or_delta(delta.iter())?; // delta = (a | b) & ~a | a = (a | a | b) & ~a = (a | b) & ~a
        println!("(a | b) & ~a | a = (a | a | b) & ~a = (a | b) & ~a");
        nav.solutions(0, std::iter::empty())?;
        println!(
            "{:?}",
            nav.facets((std::iter::empty(), std::iter::empty()), None)
                .unwrap()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );

        nav.clear()?;

        let delta = vec!["a".to_owned(), "b".to_owned()];
        nav.or_delta(delta.iter())?; // delta = a | b
        let delta = vec!["~a".to_owned(), "d".to_owned()];
        nav.and_delta(delta.iter())?; // delta = (a | b) & (~a & d)
        println!("(a | b) & (~a & d)");
        nav.solutions(0, std::iter::empty())?;
        println!(
            "{:?}",
            nav.facets((std::iter::empty(), std::iter::empty()), None)
                .unwrap()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );

        nav.clear()?;

        let delta = vec!["a".to_owned(), "b".to_owned()];
        nav.or_delta(delta.iter())?; // delta = a | b
        println!("a | b");
        nav.solutions(2, std::iter::empty())?;
        println!(
            "{:?}",
            nav.facets((std::iter::empty(), std::iter::empty()), None)
                .unwrap()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );

        Ok(())
    }
    */
}
