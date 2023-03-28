pub mod errors;
pub mod faceted_navigation;
mod utils;
pub mod weighted_navigation;

use crate::{lex::*, nav::utils::ToHashSet};

use errors::Result;

use clingo::{ast, Control, SolverLiteral, Symbol};
use std::collections::{HashMap, HashSet};

/// Pretty prints route as `| f_0 ... f_n & f_n+1 ... f_m`.
pub fn show_route(nav: &impl Essential) {
    nav.route_repr()
}

/// Clears current route, setting route to empty route.
pub fn clear_route(nav: &mut impl Essential) -> Result<()> {
    nav.clear()
}

/// Activates all facets in `conjuncts` conjunctively.
pub fn and<S: ToString>(nav: &mut impl Essential, conjuncts: impl Iterator<Item = S>) {
    nav.and_delta(conjuncts)
}

/// Activates all facets in `disjuncts` disjunctively.
pub fn or<S: ToString>(nav: &mut impl Essential, disjuncts: impl Iterator<Item = S>) {
    nav.or_delta(disjuncts)
}

/// Enumerate `n` answer sets under current route conjunctively extended by `peek_on`.
pub fn enumerate_solutions<S: ToString>(
    nav: &mut impl Essential,
    n: usize,
    peek_on: impl Iterator<Item = S>,
) -> Result<()> {
    nav.solutions(n, peek_on)
}

////////
pub struct OnStatementData<'a, 'b> {
    atom: &'b ast::SymbolicAtom<'b>,
    builder: &'a mut ast::ProgramBuilder<'a>,
}

impl<'a, 'b> ast::StatementHandler for OnStatementData<'a, 'b> {
    // adds atom enable to all rule bodies
    fn on_statement(&mut self, stm: &ast::Statement) -> bool {
        let stm_clone = stm.clone();
        // pass through all statements that are not rules
        match stm_clone.is_a().unwrap() {
            ast::StatementIsA::Rule(stm) => {
                let atom_copy = self.atom.clone();
                dbg!(self.atom.to_string());
                //self.builder
                //    .add(&rule.into())
                //    .expect("Failed to add Rule to ProgramBuilder.");
            }
            _ => {
                self.builder
                    .add(stm)
                    .expect("Failed to add Statement to ProgramBuilder.");
            }
        }
        true
    }
}
fn copy_program(source: impl Into<String> + Clone) -> String {
    let mut copy = vec![];
    let re = regex::Regex::new(r"([a-z]+[_0-9_]*)").expect("error: copying program failed.");

    for line in source.clone().into().split("\n") {
        //dbg!(&line);
        let mut new_line = line.to_owned();
        let mut mod_in_line = vec![].to_hashset();
        for x in re.find_iter(line) {
            let s = x.as_str();
            if !mod_in_line.contains(s) {
                let y = &new_line.replace(s, &format!("{}_", s));
                new_line = y.to_owned();
                mod_in_line.insert(s);
            }
        }
        copy.push(new_line);
    }

    copy.join("\n")
}
////////

pub struct Navigator {
    /// Clingo solver.
    ctl: Control,
    /// Active route.
    route: (Vec<String>, Vec<SolverLiteral>),
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

        let lp_original = source.into();
        let lp_copy = copy_program(&lp_original);
        let lp = format!("{lp_original}\n{lp_copy}");
        //dbg!(&lp);

        ctl.add("base", &[], &lp)?;
        ctl.ground(&[clingo::Part::new("base", vec![])?])?;

        let mut literals = HashMap::new();
        for atom in ctl.symbolic_atoms()?.iter()? {
            literals.insert(atom.symbol()?, atom.literal()?);
        }

        Ok(Self {
            ctl,
            route: (vec![], vec![]),
            facets: HashSet::default(),
            literals,
            input: (lp, args),
        })
    }

    fn assume(&mut self, disjunctive: bool) -> Result<()> {
        if disjunctive {
            let lp = match self.route.0.is_empty() {
                true => self.input.0.clone(),
                _ => format!(
                    "{}\n:- {}.",
                    self.input.0,
                    self.route
                        .0
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(",") // TODO: mäh
                ),
            };

            let mut ctl = clingo::control(self.input.1.clone())?;
            ctl.add("base", &[], &lp)?;
            ctl.ground(&[clingo::Part::new("base", vec![])?])?;

            let mut literals = HashMap::new();
            for atom in ctl.symbolic_atoms()?.iter()? {
                literals.insert(atom.symbol()?, atom.literal()?);
            }

            self.ctl = ctl;
        }

        self.ctl
            .backend()
            .and_then(|mut b| b.assume(&self.route.1))
            .map_err(|e| errors::NavigatorError::Clingo(e))
    }

    pub(crate) fn and_delta<S: ToString>(&mut self, delta: impl Iterator<Item = S>) {
        delta
            .map(|f| {
                let s = f.to_string();
                match s.starts_with("~") {
                    true => parse(&s[1..])
                        .map(|symbol| self.literals.get(&symbol).map(|l| l.negate()))
                        .flatten(),
                    _ => parse(&s)
                        .map(|symbol| self.literals.get(&symbol))
                        .flatten()
                        .copied(),
                }
            })
            .for_each(|l| match l {
                Some(f) => self.route.1.push(f),
                _ => println!("invalid input ..."),
            });
    }

    pub(crate) fn or_delta<S: ToString>(&mut self, delta: impl Iterator<Item = S>) {
        // TODO: check if well-formed symbol
        delta.for_each(|f| {
            let s = f.to_string();
            match s.starts_with("~") {
                true => self.route.0.push(s[1..].to_string()),
                _ => self.route.0.push(format!("not {}", s)),
            };
        });
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
    /// Activates all facets in `delta` conjunctively.
    fn and_delta<S: ToString>(&mut self, delta: impl Iterator<Item = S>);
    /// Activates all facets in `delta` disjunctively.
    fn or_delta<S: ToString>(&mut self, delta: impl Iterator<Item = S>);
    /// Enumerate `n` answer sets under current route conjunctively extended by `peek_on`.
    fn solutions<S: ToString>(&mut self, n: usize, peek_on: impl Iterator<Item = S>) -> Result<()>;
}
impl Essential for Navigation {
    fn route_repr(&self) {
        match &self {
            Self::And(nav) => {
                print!("& ");
                nav.route.1.iter().for_each(|f| {
                    match f.get_integer() > 0 {
                        true => nav
                            .literals
                            .iter()
                            .find(|(_, v)| *v == f)
                            .map(|(k, _)| print!("{} ", k.to_string())),
                        _ => nav
                            .literals
                            .iter()
                            .find(|(_, v)| **v == f.negate())
                            .map(|(k, _)| print!("~{} ", k.to_string())),
                    };
                });
            }
            Self::AndOr(nav) => {
                print!("| ");
                nav.route
                    .0
                    .iter()
                    .for_each(|f| match f.starts_with("not ") {
                        true => print!("~{} ", &f[4..]),
                        _ => print!("{} ", f),
                    });
                print!("& ");
                nav.route.1.iter().for_each(|f| {
                    match f.get_integer() > 0 {
                        true => nav
                            .literals
                            .iter()
                            .find(|(_, v)| *v == f)
                            .map(|(k, _)| print!("{} ", k.to_string())),
                        _ => nav
                            .literals
                            .iter()
                            .find(|(_, v)| **v == f.negate())
                            .map(|(k, _)| print!("~{} ", k.to_string())),
                    };
                });
            }
        }
    }
    fn clear(&mut self) -> Result<()> {
        match self {
            Self::And(nav) => {
                nav.route.1.clear();
                Ok(())
            }
            Self::AndOr(nav) => {
                let disjunctive = !nav.route.0.is_empty();
                nav.route.0.clear();
                nav.route.1.clear();
                nav.assume(disjunctive)
            }
        }
    }
    fn and_delta<S: ToString>(&mut self, delta: impl Iterator<Item = S>) {
        match self {
            Self::And(nav) | Self::AndOr(nav) => nav.and_delta(delta),
        }
    }
    fn or_delta<S: ToString>(&mut self, delta: impl Iterator<Item = S>) {
        match self {
            Self::And(nav) | Self::AndOr(nav) => nav.or_delta(delta),
        }
    }
    fn solutions<S: ToString>(&mut self, n: usize, peek_on: impl Iterator<Item = S>) -> Result<()> {
        match self {
            Self::And(nav) => {
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
                route.extend(nav.route.1.clone());

                let mut handle = nav.ctl.fasb_solve(clingo::SolveMode::YIELD, &route)?;
                let mut i = 1;

                match n == 0 {
                    true => {
                        while let Ok(Some(answer_set)) = handle.model() {
                            println!("Solution {:?}: ", i);
                            let atoms = answer_set.symbols(clingo::ShowType::SHOWN)?;
                            atoms.iter().for_each(|atom| {
                                let s = atom.to_string();
                                // FIX:
                                if !s.ends_with("_") {
                                    print!("{} ", s);
                                }
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
                                let s = atom.to_string();
                                // FIX:
                                if !s.ends_with("_") {
                                    print!("{} ", s);
                                }
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
            Self::AndOr(nav) => {
                let route = peek_on
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

                nav.assume(!nav.route.0.is_empty())?;

                let mut handle = nav.ctl.fasb_solve(clingo::SolveMode::YIELD, &route)?;
                let mut i = 1;

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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup() {
        assert!(Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()]).is_ok());
        assert!(Navigator::new("a;;b. c;d :- b. e.", vec!["0".to_string()]).is_err());
        assert!(Navigator::new("a;b. c;d :- b. e.", vec![]).is_ok());

        assert!(Navigator::new(
            "{q(I ,1..8)} == 1 :- I = 1..8.
{q(1..8, J)} == 1 :- J = 1..8.
:- {q(D-J, J)} >= 2, D = 2..2*8.
:- {q(D+J, J)} >= 2, D = 1-8..8-1.",
            vec!["0".to_string()]
        )
        .is_ok());
    }

    /*
    #[test]
    fn and_clear() -> Result<()> {
        let mut nav = Navigation::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;
        let delta = vec!["b".to_owned(), "~c".to_owned()];
        nav.and_delta(delta.iter())?;
        nav.solutions(0, std::iter::empty())?;

        nav.clear()?;

        let delta = vec!["~a".to_owned()];
        nav.and_delta(delta.iter())?;
        nav.solutions(0, std::iter::empty())?;

        nav.clear()?;

        let delta = vec!["a".to_owned(), "b".to_owned()];
        nav.and_delta(delta.iter())?;
        nav.solutions(0, std::iter::empty())?;

        Ok(())
    }

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
