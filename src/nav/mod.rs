mod errors;
mod utils;

use crate::lex::*;
use utils::ToHashSet;

use errors::Result;

use clingo::{Control, SolverLiteral, Symbol};
use std::collections::{HashMap, HashSet};

pub struct Navigator {
    /// Clingo solver.
    ctl: Control,
    /// Active route.
    route: (Vec<String>, Vec<SolverLiteral>),
    /// Current facets.
    facets: HashSet<Symbol>,
    /// Literals.
    literals: HashMap<Symbol, SolverLiteral>,
}
impl Navigator {
    /// Constructs `Navigator`.
    pub fn new(source: impl Into<String>, args: Vec<String>) -> Result<Self> {
        let mut ctl = clingo::control(args)?;

        let lp = source.into();
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
        })
    }

    /// Parses facet.
    pub fn parse_facet(&self, exp: &str) -> Option<Symbol> {
        Atom(&exp).parse(&['~'])
    }

    /// Makes assumptions according to specified route.
    fn assume(&mut self, disjunctive: bool) -> Result<()> {
        self.ctl
            .backend()
            .and_then(|mut b| b.assume(&self.route.1))
            .map_err(|e| errors::NavigatorError::Clingo(e))?;
        if disjunctive {
            let or_constraint = format!(
                ":- {}.",
                self.route
                    .0
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(",") // TODO: mäh
            );
            self.ctl
                .add("or", &[], &or_constraint)
                .map_err(|e| errors::NavigatorError::Clingo(e))?;
            return self
                .ctl
                .ground(&[clingo::Part::new("or", vec![])?])
                .map_err(|e| errors::NavigatorError::Clingo(e));
        }

        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.route.0.clear();
        self.route.1.clear();
        self.assume(false)
    }

    /// Activates conjunctive route `delta`.
    pub fn and_delta<S: ToString>(&mut self, delta: impl Iterator<Item = S>) -> Result<()> {
        delta
            .map(|f| {
                let s = f.to_string();
                match s.starts_with("~") {
                    true => Atom(&s[1..])
                        .parse(&['-'])
                        .map(|symbol| self.literals.get(&symbol).map(|l| l.negate()))
                        .flatten(),
                    _ => Atom(&s)
                        .parse(&['-'])
                        .map(|symbol| self.literals.get(&symbol))
                        .flatten()
                        .copied(),
                }
            })
            .for_each(|l| match l {
                Some(f) => self.route.1.push(f),
                _ => println!("invalid input ..."),
            });

        self.assume(true)
    }

    /// Activates disjunctive route `delta`.
    pub fn or_delta<S: ToString>(&mut self, delta: impl Iterator<Item = S>) -> Result<()> {
        // TODO: check if well-formed symbol
        delta.for_each(|f| {
            let s = f.to_string();
            match s.starts_with("~") {
                true => self.route.0.push(s[1..].to_string()),
                _ => self.route.0.push(format!("not {}", s)),
            };
        });

        Ok(())
    }

    /// Enumerates `n` answer sets within sub-space encoded by current route.
    pub fn solutions(&mut self, n: usize, peek_on: Option<&[SolverLiteral]>) -> Result<()> {
        let route = peek_on.unwrap_or(&self.route.1);
        let mut handle = self.ctl.fasb_solve(clingo::SolveMode::YIELD, &route)?;
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

pub trait FacetedNavigation {
    fn brave_consequences(&mut self, peek_on: &[SolverLiteral]) -> Option<Vec<Symbol>>;
    fn cautious_consequences(&mut self, peek_on: &[SolverLiteral]) -> Option<Vec<Symbol>>;
    fn facets(&mut self, peek_on: &[SolverLiteral]) -> Option<HashSet<Symbol>>;
}
impl FacetedNavigation for Navigator {
    fn brave_consequences(&mut self, peek_on: &[SolverLiteral]) -> Option<Vec<Symbol>> {
        self.assume(!self.route.1.is_empty()).ok()?;

        self.ctl
            .configuration_mut()
            .map(|c| {
                c.root()
                    .and_then(|rk| c.map_at(rk, "solve.enum_mode"))
                    .map(|sk| c.value_set(sk, "brave"))
                    .ok()
            })
            .ok()?;

        let mut bcs = vec![];
        let mut handle = self
            .ctl
            .fasb_solve(clingo::SolveMode::YIELD, &peek_on)
            .ok()?;

        while let Ok(Some(xs)) = handle.model() {
            bcs = xs.symbols(clingo::ShowType::SHOWN).ok()?;
            handle.resume().ok()?;
        }

        self.ctl
            .configuration_mut()
            .map(|c| {
                c.root()
                    .and_then(|rk| c.map_at(rk, "solve.enum_mode"))
                    .map(|sk| c.value_set(sk, "auto"))
                    .ok()
            })
            .ok()?;

        Some(bcs)
    }

    fn cautious_consequences(&mut self, peek_on: &[SolverLiteral]) -> Option<Vec<Symbol>> {
        self.assume(!self.route.1.is_empty()).ok()?;

        self.ctl
            .configuration_mut()
            .map(|c| {
                c.root()
                    .and_then(|rk| c.map_at(rk, "solve.enum_mode"))
                    .map(|sk| c.value_set(sk, "cautious"))
                    .ok()
            })
            .ok()?;

        let mut ccs = vec![];
        let mut handle = self
            .ctl
            .fasb_solve(clingo::SolveMode::YIELD, &peek_on)
            .ok()?;

        while let Ok(Some(xs)) = handle.model() {
            ccs = xs.symbols(clingo::ShowType::SHOWN).ok()?;
            handle.resume().ok()?;
        }

        self.ctl
            .configuration_mut()
            .map(|c| {
                c.root()
                    .and_then(|rk| c.map_at(rk, "solve.enum_mode"))
                    .map(|sk| c.value_set(sk, "auto"))
                    .ok()
            })
            .ok()?;

        Some(ccs)
    }

    fn facets(&mut self, peek_on: &[SolverLiteral]) -> Option<HashSet<Symbol>> {
        let bcs = self.brave_consequences(peek_on)?;
        match bcs.is_empty() {
            true => Some(HashSet::new()),
            _ => Some(bcs.difference_as_set(&self.cautious_consequences(peek_on)?)),
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
    }

    #[test]
    fn and_clear() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;
        let delta = vec!["b".to_owned(), "~c".to_owned()];
        nav.and_delta(delta.iter())?;
        nav.solutions(0, None)?;

        nav.clear()?;

        let delta = vec!["~a".to_owned()];
        nav.and_delta(delta.iter())?;
        nav.solutions(0, None)?;

        nav.clear()?;

        let delta = vec!["a".to_owned(), "b".to_owned()];
        nav.and_delta(delta.iter())?;
        nav.solutions(0, None)?;

        Ok(())
    }

    #[test]
    fn or_clear() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;
        let delta = vec!["a".to_owned(), "d".to_owned()];
        nav.or_delta(delta.iter())?;
        nav.solutions(0, None)?;

        nav.clear()?;

        let delta = vec!["a".to_owned(), "~e".to_owned()];
        nav.and_delta(delta.iter())?;
        nav.solutions(0, None)?;

        Ok(())
    }

    #[test]
    fn and_or_clear() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;
        let delta = vec!["a".to_owned(), "b".to_owned()];
        nav.or_delta(delta.iter())?; // delta = a | b
        let delta = vec!["~a".to_owned()];
        nav.and_delta(delta.iter())?; // delta = (a | b) & ~a
        let delta = vec!["a".to_owned()];
        nav.and_delta(delta.iter())?; // delta = (a | b) & ~a & a
        nav.solutions(0, None)?;
        dbg!(nav
            .facets(&[])
            .unwrap()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>());

        nav.clear()?;

        let delta = vec!["a".to_owned(), "b".to_owned()];
        nav.or_delta(delta.iter())?; // delta = a | b
        let delta = vec!["~a".to_owned()];
        nav.and_delta(delta.iter())?; // delta = (a | b) & ~a
        let delta = vec!["a".to_owned()];
        nav.or_delta(delta.iter())?; // delta = (a | b) & ~a | a = (a | a | b) & ~a = (a | b) & ~a
        nav.solutions(0, None)?;
        dbg!(nav
            .facets(&[])
            .unwrap()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>());

        nav.clear()?;

        let delta = vec!["a".to_owned(), "b".to_owned()];
        nav.or_delta(delta.iter())?; // delta = a | b
        let delta = vec!["~a".to_owned(), "d".to_owned()];
        nav.and_delta(delta.iter())?; // delta = (a | b) & (~a & d)
        nav.solutions(0, None)?;
        dbg!(nav
            .facets(&[])
            .unwrap()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>());

        nav.clear()?;

        let delta = vec!["a".to_owned(), "b".to_owned()];
        nav.or_delta(delta.iter())?; // delta = a | b
        nav.solutions(2, None)?;
        dbg!(nav
            .facets(&[])
            .unwrap()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>());

        Ok(())
    }
}
