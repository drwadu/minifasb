use super::faceted_navigation::{fs_stats, FacetedNavigation};

/// TODO
pub fn count<S: ToString, T>(
    sharp: &mut impl WeightedNavigation<T>,
    nav: &mut T,
    peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
) -> Option<(usize, Option<usize>)> {
    sharp.eval_sharp(nav, peek_on)
}

pub trait WeightedNavigation<T> {
    fn eval_sharp<S: ToString>(
        &mut self,
        nav: &mut T,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
    ) -> Option<(usize, Option<usize>)>;
    fn eval_sharp_restricted<S: ToString>(
        &mut self,
        nav: &mut T,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
        target: &[S],
    ) -> Option<(usize, Option<usize>)>;
}

#[derive(Debug, Clone)]
pub enum Sharp {
    AnswerSetCounting,
    FacetCounting,
    BcCounting,
    CcCounting,
}

impl<T: FacetedNavigation> WeightedNavigation<T> for Sharp {
    fn eval_sharp<S: ToString>(
        &mut self,
        nav: &mut T,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
    ) -> Option<(usize, Option<usize>)> {
        match self {
            Self::FacetCounting => fs_stats(nav, peek_on).and_then(|(_, _, fsc)| Some((fsc, None))),
            Self::AnswerSetCounting => {
                todo!()
            }
            Self::BcCounting => fs_stats(nav, peek_on).and_then(|(bcc, _, _)| Some((bcc, None))),
            Self::CcCounting => fs_stats(nav, peek_on).and_then(|(_, ccc, _)| Some((ccc, None))),
        }
    }
    fn eval_sharp_restricted<S: ToString>(
        &mut self,
        _nav: &mut T,
        _peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
        _target: &[S],
    ) -> Option<(usize, Option<usize>)> {
        todo!()
    }
}
