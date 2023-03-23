use super::utils::ToHashSet;
use super::{parse, Navigation};
use clingo::{SolverLiteral, Symbol};
use std::collections::HashSet;

use super::faceted_navigation::{fs_stats, FacetedNavigation};
use super::Navigator;

pub trait WeightedNavigation {
    fn eval_sharp<S: ToString>(
        &mut self,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
    ) -> Option<(usize, Option<usize>)>;
    fn eval_sharp_restricted<S: ToString>(
        &mut self,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
        target: &[S],
    ) -> Option<(usize, Option<usize>)>;
}

#[derive(Debug, Clone)]
pub enum Sharp<T> {
    AnswerSetCounting(T),
    FacetCounting(T),
    BcCounting(T),
    CcCounting(T),
}

impl<T: FacetedNavigation> WeightedNavigation for Sharp<T> {
    fn eval_sharp<S: ToString>(
        &mut self,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
    ) -> Option<(usize, Option<usize>)> {
        match self {
            Self::FacetCounting(nav) => {
                fs_stats(nav, peek_on).and_then(|(_, _, fsc)| Some((fsc, None)))
            }
            Self::AnswerSetCounting(_) => {
                todo!()
            }
            Self::BcCounting(nav) => {
                fs_stats(nav, peek_on).and_then(|(bcc, _, _)| Some((bcc, None)))
            }
            Self::CcCounting(nav) => {
                fs_stats(nav, peek_on).and_then(|(_, ccc, _)| Some((ccc, None)))
            }
        }
    }
    fn eval_sharp_restricted<S: ToString>(
        &mut self,
        _peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
        _target: &[S],
    ) -> Option<(usize, Option<usize>)> {
        todo!()
    }
}
