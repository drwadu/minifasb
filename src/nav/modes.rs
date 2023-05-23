use super::{answer_set_count, utils::ToHashSet};
use crate::nav::weighted_navigation::Weight;
use crate::nav::Navigator;
use clingo::{SolverLiteral, Symbol};

use super::faceted_navigation::{consequences, Consequences};

pub enum Mode {
    GoalOriented,
    MinWeighted(Weight),
    MaxWeighted(Weight),
}
pub trait Guide {
    fn step(&mut self, nav: &mut Navigator) -> Option<(String, SolverLiteral)>;
}
impl Guide for Mode {
    fn step(&mut self, nav: &mut Navigator) -> Option<(String, SolverLiteral)> {
        let mut active = nav.conjuncts.0.clone();
        let bc = consequences(Consequences::Brave, nav, &active)?;
        let fs = match !bc.is_empty() {
            true => unsafe {
                consequences(Consequences::Cautious, nav, &active)
                    .as_ref()
                    .map(|ccs| {
                        bc.difference_as_set(&ccs)
                            .iter()
                            .cloned()
                            .collect::<Vec<Symbol>>()
                    })
                    .unwrap_unchecked()
            },
            _ => bc,
        };
        let lits = nav.literals.clone();

        match self {
            Self::GoalOriented => fs
                .into_iter()
                .next()
                .and_then(|f| Some((f.to_string(), *unsafe { lits.get(&f).unwrap_unchecked() }))),
            Self::MaxWeighted(Weight::FacetCounting) => {
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) = (fs.len(), None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                    active.push(*l);
                    let bc = consequences(Consequences::Brave, nav, &active)?;
                    let cc = consequences(Consequences::Cautious, nav, &active)?;
                    let count = bc.to_hashset().difference(&cc.to_hashset()).count();
                    if count == 1 {
                        return Some((sym.to_string(), *l));
                    }
                    if count <= curr {
                        curr = count;
                        f = Some((sym.to_string(), *l));
                    }
                    active.pop();

                    let ln = l.negate();
                    active.push(ln);
                    let bc = consequences(Consequences::Brave, nav, &active)?;
                    let cc = consequences(Consequences::Cautious, nav, &active)?;
                    let count = bc.to_hashset().difference(&cc.to_hashset()).count();
                    if count == 1 {
                        return Some((format!("~{sym}"), ln));
                    }
                    if count <= curr {
                        curr = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                    active.pop();
                }

                f
            }
            Self::MaxWeighted(Weight::AnswerSetCounting) => {
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) = (0, None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                    active.push(*l);
                    let count = answer_set_count(nav, &active, curr).ok()?;
                    if count == 1 {
                        return Some((sym.to_string(), *l));
                    }
                    if count <= curr {
                        curr = count;
                        f = Some((sym.to_string(), *l));
                    }
                    active.pop();

                    let ln = l.negate();
                    active.push(ln);
                    let count = answer_set_count(nav, &active, curr).ok()?;
                    if count == 1 {
                        return Some((format!("~{sym}"), ln));
                    }
                    if count <= curr {
                        curr = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                    active.pop();

                }

                f
            }
            Self::MaxWeighted(Weight::SupportedModelCounting) => {
                eprintln!("ensure --supp-models flag was specified at startup.");
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) = (0, None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                    active.push(*l);
                    let count = answer_set_count(nav, &active, curr).ok()?;
                    if count == 1 {
                        return Some((sym.to_string(), *l));
                    }
                    if count <= curr {
                        curr = count;
                        f = Some((sym.to_string(), *l));
                    }
                    active.pop();

                    let ln = l.negate();
                    active.push(ln);
                    let count = answer_set_count(nav, &active, curr).ok()?;
                    if count == 1 {
                        return Some((format!("~{sym}"), ln));
                    }
                    if count <= curr {
                        curr = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                    active.pop();
                }

                f
            }
            Self::MinWeighted(Weight::FacetCounting) => {
                let ub = fs.len() - 1;
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) = (ub, None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };

                    let ln = l.negate();
                    active.push(ln);
                    let bc = consequences(Consequences::Brave, nav, &active)?;
                    let cc = consequences(Consequences::Cautious, nav, &active)?;
                    let count = bc.to_hashset().difference(&cc.to_hashset()).count();
                    if count == ub {
                        return Some((format!("~{sym}"), ln));
                    }
                    if curr <= count {
                        curr = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                    active.pop();

                    active.push(*l);
                    let bc = consequences(Consequences::Brave, nav, &active)?;
                    let cc = consequences(Consequences::Cautious, nav, &active)?;
                    let count = bc.to_hashset().difference(&cc.to_hashset()).count();
                    if count == ub {
                        return Some((sym.to_string(), *l));
                    }
                    if curr <= count {
                        curr = count;
                        f = Some((sym.to_string(), *l));
                    }
                    active.pop();
                }

                f
            }
            Self::MinWeighted(Weight::AnswerSetCounting) => {
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) = (usize::MAX-1, None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                    let ln = l.negate();
                    println!("bla");
                    
                    active.push(ln);
                    let count = answer_set_count(nav, &active, curr).ok()?;
                    if curr <= count {
                        curr = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                    active.pop();
                    dbg!(&f,curr);

                    active.push(*l);
                    let count = answer_set_count(nav, &active, curr).ok()?;
                    if curr <= count {
                        curr = count;
                        f = Some((sym.to_string(), *l));
                    }
                    active.pop();
                    dbg!(&f,curr);

                }

                f
            }
            Self::MinWeighted(Weight::SupportedModelCounting) => {
                eprintln!("ensure --supp-models flag was specified at startup.");
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) = (usize::MAX-1, None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                    let ln = l.negate();
                    active.push(ln);
                    let count = answer_set_count(nav, &active, curr).ok()?;
                    if curr <= count {
                        curr = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                    active.pop();

                    active.push(*l);
                    let count = answer_set_count(nav, &active, curr).ok()?;
                    if curr <= count {
                        curr = count;
                        f = Some((sym.to_string(), *l));
                    }
                    active.pop();

                }

                f
            }
            _ => todo!(),
        }
    }
}
