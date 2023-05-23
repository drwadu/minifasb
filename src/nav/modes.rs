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
        nav.ctl
            .backend()
            .and_then(|mut b| b.assume(&nav.conjuncts.0))
            .ok()?;
        let bc = consequences(Consequences::Brave, nav, &[])?;
        let fs = match !bc.is_empty() {
            true => unsafe {
                consequences(Consequences::Cautious, nav, &[])
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
            Self::MinWeighted(Weight::FacetCounting) => {
                let (mut min, mut f): (usize, Option<(String, SolverLiteral)>) = (1, None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                    let bc = consequences(Consequences::Brave, nav, &[*l])?;
                    let cc = consequences(Consequences::Cautious, nav, &[*l])?;
                    let count = bc.to_hashset().difference(&cc.to_hashset()).count();
                    if count == 1 {
                        return Some((sym.to_string(), *l));
                    }
                    if count <= min {
                        min = count;
                        f = Some((sym.to_string(), *l));
                    }

                    let ln = l.negate();
                    let bc = consequences(Consequences::Brave, nav, &[ln])?;
                    let cc = consequences(Consequences::Cautious, nav, &[ln])?;
                    let count = bc.to_hashset().difference(&cc.to_hashset()).count();
                    if count == 1 {
                        return Some((format!("~{sym}"), ln));
                    }
                    if count <= min {
                        min = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                }

                f
            }
            Self::MinWeighted(Weight::AnswerSetCounting) => {
                let (mut min, mut f): (usize, Option<(String, SolverLiteral)>) = (0, None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                    let count = answer_set_count(nav, &[*l], min).ok()?;
                    if count == 1 {
                        return Some((sym.to_string(), *l));
                    }
                    if count <= min {
                        min = count;
                        f = Some((sym.to_string(), *l));
                    }

                    let ln = l.negate();
                    let count = answer_set_count(nav, &[ln], min).ok()?;
                    if count == 1 {
                        return Some((format!("~{sym}"), ln));
                    }
                    if count <= min {
                        min = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                }

                f
            }
            Self::MinWeighted(Weight::SupportedModelCounting) => {
                eprintln!("ensure --supp-models flag was specified at startup.");
                let (mut min, mut f): (usize, Option<(String, SolverLiteral)>) = (0, None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                    let count = answer_set_count(nav, &[*l], min).ok()?;
                    if count == 1 {
                        return Some((sym.to_string(), *l));
                    }
                    if count <= min {
                        min = count;
                        f = Some((sym.to_string(), *l));
                    }

                    let ln = l.negate();
                    let count = answer_set_count(nav, &[ln], min).ok()?;
                    if count == 1 {
                        return Some((format!("~{sym}"), ln));
                    }
                    if count <= min {
                        min = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                }

                f
            }
            Self::MaxWeighted(Weight::FacetCounting) => {
                let ub = fs.len();
                let (mut max, mut f): (usize, Option<(String, SolverLiteral)>) = (ub, None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                    let bc = consequences(Consequences::Brave, nav, &[*l])?;
                    let cc = consequences(Consequences::Cautious, nav, &[*l])?;
                    let count = bc.to_hashset().difference(&cc.to_hashset()).count();
                    if count == ub {
                        return Some((sym.to_string(), *l));
                    }
                    if max <= count {
                        max = count;
                        f = Some((sym.to_string(), *l));
                    }

                    let ln = l.negate();
                    let bc = consequences(Consequences::Brave, nav, &[ln])?;
                    let cc = consequences(Consequences::Cautious, nav, &[ln])?;
                    let count = bc.to_hashset().difference(&cc.to_hashset()).count();
                    if count == ub {
                        return Some((format!("~{sym}"), ln));
                    }
                    if max <= count {
                        max = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                }

                f
            }
            Self::MaxWeighted(Weight::AnswerSetCounting) => {
                let (mut max, mut f): (usize, Option<(String, SolverLiteral)>) = (0, None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                    let count = answer_set_count(nav, &[*l], max).ok()?;
                    if max <= count {
                        max = count;
                        f = Some((sym.to_string(), *l));
                    }

                    let ln = l.negate();
                    let count = answer_set_count(nav, &[ln], max).ok()?;
                    if max <= count {
                        max = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                }

                f
            }
            Self::MaxWeighted(Weight::SupportedModelCounting) => {
                eprintln!("ensure --supp-models flag was specified at startup.");
                let (mut max, mut f): (usize, Option<(String, SolverLiteral)>) = (0, None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                    let count = answer_set_count(nav, &[*l], max).ok()?;
                    if max <= count {
                        max = count;
                        f = Some((sym.to_string(), *l));
                    }

                    let ln = l.negate();
                    let count = answer_set_count(nav, &[ln], max).ok()?;
                    if max <= count {
                        max = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                }

                f
            }
            _ => todo!(),
        }
    }
}
