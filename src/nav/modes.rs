use super::{answer_set_count, utils::ToHashSet};
use crate::lex;
use crate::nav::weighted_navigation::Weight;
use crate::nav::Navigator;
use clingo::{SolverLiteral, Symbol};

use super::faceted_navigation::{consequences, Consequences};

#[cfg(feature = "verbose")]
use std::time::Instant;

#[allow(unused)]
pub enum Mode {
    GoalOriented,
    MinWeighted(Weight),
    MaxWeighted(Weight),
}
pub trait Guide {
    fn step(
        &mut self,
        nav: &mut Navigator,
        split_on: &mut Option<usize>,
    ) -> Option<(String, SolverLiteral)>;
    fn step_wrt(
        &mut self,
        nav: &mut Navigator,
        curr: &[String],
        split_on: &mut Option<usize>,
    ) -> Option<(String, SolverLiteral)>;
}
impl Guide for Mode {
    fn step(
        &mut self,
        nav: &mut Navigator,
        split_on: &mut Option<usize>,
    ) -> Option<(String, SolverLiteral)> {
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
        if fs.is_empty() {
            return None;
        }

        match self {
            Self::GoalOriented => fs
                .into_iter()
                .next()
                .and_then(|f| Some((f.to_string(), *unsafe { lits.get(&f).unwrap_unchecked() }))),
            Self::MaxWeighted(Weight::FacetCounting) => {
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) =
                    (fs.len() - 1, None);
                for sym in fs {
                    let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                    active.push(*l);
                    let bc = consequences(Consequences::Brave, nav, &active)?;
                    let cc = consequences(Consequences::Cautious, nav, &active)?;
                    let count = bc.to_hashset().difference(&cc.to_hashset()).count();
                    if count == 0 {
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
                    if count == 0 {
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
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) =
                    (usize::MAX - 1, None);

                if let Some(c) = split_on {
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
                        let count_ = *c - count;
                        if count_ == 1 {
                            return Some((format!("~{sym}"), ln));
                        }
                        if count_ <= curr {
                            curr = count_;
                            f = Some((format!("~{sym}"), ln));
                        }
                    }
                } else {
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
                }

                *split_on = Some(curr);

                f
            }
            Self::MinWeighted(Weight::FacetCounting) => {
                let ub = fs.len() - 1;
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) = (0, None);
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
                    if count >= curr {
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

                    if count >= curr {
                        curr = count;
                        f = Some((sym.to_string(), *l));
                    }
                    active.pop();
                }

                f
            }
            Self::MinWeighted(Weight::AnswerSetCounting) => {
                let ub = usize::MAX - 1;
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) = (1, None);

                if let Some(c) = split_on {
                    for sym in fs {
                        let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                        let ln = l.negate();

                        active.push(ln);
                        let count = answer_set_count(nav, &active, curr).ok()?;
                        if count == ub {
                            return Some((sym.to_string(), *l));
                        }
                        if count >= curr {
                            curr = count;
                            f = Some((format!("~{sym}"), ln));
                        }
                        active.pop();

                        let count_ = *c - count;
                        if count_ == ub {
                            return Some((sym.to_string(), *l));
                        }
                        if count_ >= curr {
                            curr = count_;
                            f = Some((sym.to_string(), *l));
                        }
                    }
                } else {
                    for sym in fs {
                        let l = unsafe { lits.get(&sym).unwrap_unchecked() };
                        let ln = l.negate();

                        active.push(ln);
                        let count = answer_set_count(nav, &active, curr).ok()?;
                        if count == ub {
                            return Some((sym.to_string(), *l));
                        }
                        if count >= curr {
                            curr = count;
                            f = Some((format!("~{sym}"), ln));
                        }
                        active.pop();

                        active.push(*l);
                        let count = answer_set_count(nav, &active, curr).ok()?;
                        if count == ub {
                            return Some((sym.to_string(), *l));
                        }
                        if count >= curr {
                            curr = count;
                            f = Some((sym.to_string(), *l));
                        }
                        active.pop();
                    }
                }

                *split_on = Some(curr);

                f
            }
            _ => todo!(),
        }
    }

    fn step_wrt(
        &mut self,
        nav: &mut Navigator,
        curr: &[String],
        split_on: &mut Option<usize>,
    ) -> Option<(String, SolverLiteral)> {
        let mut active = nav.conjuncts.0.clone();
        let fs = curr;
        let lits = nav.literals.clone();
        if fs.is_empty() {
            return None;
        }

        #[cfg(feature = "verbose")]
        eprintln!("step started");
        #[cfg(feature = "verbose")]
        let start = Instant::now();
        let ret = match self {
            Self::GoalOriented => fs.into_iter().next().and_then(|f| {
                Some((f.clone(), *unsafe {
                    lits.get(&lex::parse(f).unwrap_unchecked())
                        .unwrap_unchecked()
                }))
            }),
            Self::MaxWeighted(Weight::FacetCounting) => {
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) =
                    (fs.len() - 1, None);
                for sym in fs {
                    let l = unsafe {
                        lits.get(&lex::parse(sym).unwrap_unchecked())
                            .unwrap_unchecked()
                    };
                    active.push(*l);
                    let bc = consequences(Consequences::Brave, nav, &active)?;
                    let cc = consequences(Consequences::Cautious, nav, &active)?;
                    let count = bc.to_hashset().difference(&cc.to_hashset()).count();
                    if count == 0 {
                        #[cfg(feature = "verbose")]
                        println!("early stoppage +");
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
                    if count == 0 {
                        #[cfg(feature = "verbose")]
                        println!("early stoppage -");
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
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) =
                    (usize::MAX - 1, None);

                if let Some(c) = split_on {
                    for sym in fs {
                        let l = unsafe {
                            lits.get(&lex::parse(sym).unwrap_unchecked())
                                .unwrap_unchecked()
                        };
                        active.push(*l);
                        let count = answer_set_count(nav, &active, curr).ok()?;
                        if count == 1 {
                            #[cfg(feature = "verbose")]
                            println!("early stoppage +");
                            return Some((sym.to_string(), *l));
                        }
                        if count <= curr {
                            curr = count;
                            f = Some((sym.to_string(), *l));
                        }
                        active.pop();

                        let ln = l.negate();
                        let count_ = *c - count;
                        if count_ == 1 {
                            #[cfg(feature = "verbose")]
                            println!("early stoppage -");
                            return Some((format!("~{sym}"), ln));
                        }
                        if count_ <= curr {
                            curr = count_;
                            f = Some((format!("~{sym}"), ln));
                        }
                    }
                } else {
                    for sym in fs {
                        let l = unsafe {
                            lits.get(&lex::parse(sym).unwrap_unchecked())
                                .unwrap_unchecked()
                        };
                        active.push(*l);
                        let count = answer_set_count(nav, &active, curr).ok()?;
                        if count == 1 {
                            #[cfg(feature = "verbose")]
                            println!("early stoppage +");
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
                            #[cfg(feature = "verbose")]
                            println!("early stoppage -");
                            return Some((format!("~{sym}"), ln));
                        }
                        if count <= curr {
                            curr = count;
                            f = Some((format!("~{sym}"), ln));
                        }
                        active.pop();
                    }
                }

                *split_on = Some(curr);

                f
            }
            Self::MinWeighted(Weight::FacetCounting) => {
                let ub = fs.len() - 1;
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) = (0, None);
                for sym in fs {
                    let l = unsafe {
                        lits.get(&lex::parse(sym).unwrap_unchecked())
                            .unwrap_unchecked()
                    };

                    let ln = l.negate();
                    active.push(ln);
                    let bc = consequences(Consequences::Brave, nav, &active)?;
                    let cc = consequences(Consequences::Cautious, nav, &active)?;
                    let count = bc.to_hashset().difference(&cc.to_hashset()).count();
                    if count == ub {
                        #[cfg(feature = "verbose")]
                        println!("early stoppage -");
                        return Some((format!("~{sym}"), ln));
                    }
                    if count >= curr {
                        curr = count;
                        f = Some((format!("~{sym}"), ln));
                    }
                    active.pop();

                    active.push(*l);
                    let bc = consequences(Consequences::Brave, nav, &active)?;
                    let cc = consequences(Consequences::Cautious, nav, &active)?;
                    let count = bc.to_hashset().difference(&cc.to_hashset()).count();
                    if count == ub {
                        #[cfg(feature = "verbose")]
                        println!("early stoppage +");
                        return Some((sym.to_string(), *l));
                    }
                    if count >= curr {
                        curr = count;
                        f = Some((sym.to_string(), *l));
                    }
                    active.pop();
                }

                f
            }
            Self::MinWeighted(Weight::AnswerSetCounting) => {
                let ub = usize::MAX - 1;
                let (mut curr, mut f): (usize, Option<(String, SolverLiteral)>) = (0, None);

                if let Some(c) = split_on {
                    let ub = *c - 1;
                    for sym in fs {
                        let l = unsafe {
                            lits.get(&lex::parse(sym).unwrap_unchecked())
                                .unwrap_unchecked()
                        };
                        let ln = l.negate();

                        active.push(ln);
                        let count = answer_set_count(nav, &active, curr).ok()?;
                        if count == ub {
                            #[cfg(feature = "verbose")]
                            println!("early stoppage -");
                            return Some((sym.to_string(), ln));
                        }
                        if count >= curr {
                            curr = count;
                            f = Some((format!("~{sym}"), ln));
                        }
                        active.pop();

                        let count_ = *c - count;
                        if count_ == ub {
                            #[cfg(feature = "verbose")]
                            println!("early stoppage +");
                            return Some((sym.to_string(), *l));
                        }
                        if count_ >= curr {
                            curr = count_;
                            f = Some((sym.to_string(), *l));
                        }
                    }
                } else {
                    for sym in fs {
                        let l = unsafe {
                            lits.get(&lex::parse(sym).unwrap_unchecked())
                                .unwrap_unchecked()
                        };
                        let ln = l.negate();

                        active.push(ln);
                        let count = answer_set_count(nav, &active, curr).ok()?;
                        if count == ub {
                            #[cfg(feature = "verbose")]
                            println!("early stoppage -");
                            return Some((sym.to_string(), ln));
                        }
                        if count >= curr {
                            curr = count;
                            f = Some((format!("~{sym}"), ln));
                        }
                        active.pop();

                        active.push(*l);
                        let count = answer_set_count(nav, &active, curr).ok()?;
                        if count == ub {
                            #[cfg(feature = "verbose")]
                            println!("early stoppage +");
                            return Some((sym.to_string(), *l));
                        }
                        if count >= curr {
                            curr = count;
                            f = Some((sym.to_string(), *l));
                        }
                        active.pop();
                    }
                }

                *split_on = Some(curr);

                f
            }
            _ => todo!(),
        };

        #[cfg(feature = "verbose")]
        eprintln!("step elapsed: {:?}", start.elapsed());
        ret
    }
}
