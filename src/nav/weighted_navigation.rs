use super::{
    answer_set_count,
    faceted_navigation::{fs_stats, FacetedNavigation},
    Essential,
};

#[cfg(feature = "verbose")]
use std::time::Instant;

/// TODO
pub fn count<S: ToString, T>(
    sharp: &mut impl WeightedNavigation<T>,
    nav: &mut T,
    peek_on: impl Iterator<Item = S>,
) -> Option<usize> {
    sharp.eval_sharp(nav, peek_on)
}

pub trait WeightedNavigation<T> {
    fn eval_sharp<S: ToString>(
        &mut self,
        nav: &mut T,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<usize>;
    fn eval_sharp_restricted<S: ToString>(
        &mut self,
        nav: &mut T,
        peek_on: impl Iterator<Item = S>,
        target: &[S],
    ) -> Option<usize>;
}

#[derive(Debug, Clone)]
pub enum Weight {
    AnswerSetCounting,
    FacetCounting,
    #[allow(unused)]
    BcCounting,
    #[allow(unused)]
    CcCounting,
}

impl<T: FacetedNavigation + Essential> WeightedNavigation<T> for Weight {
    fn eval_sharp<S: ToString>(
        &mut self,
        nav: &mut T,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<usize> {
        match self {
            Self::FacetCounting => {
                #[cfg(feature = "verbose")]
                eprintln!("facet counting started");
                #[cfg(feature = "verbose")]
                let start = Instant::now();
                let fc = fs_stats(nav, peek_on).and_then(|(_, _, fsc)| Some(fsc));
                #[cfg(feature = "verbose")]
                eprintln!("facet counting elapsed: {:?}", start.elapsed().as_millis());
                fc
            }
            Self::AnswerSetCounting => {
                #[cfg(feature = "verbose")]
                eprintln!("answer set counting started");
                #[cfg(feature = "verbose")]
                let start = Instant::now();
                let route = nav.read_route(peek_on);
                let count = answer_set_count(nav.expose(), &route, 0).ok();
                #[cfg(feature = "verbose")]
                eprintln!(
                    "answer set counting elapsed: {:?}",
                    start.elapsed().as_millis()
                );
                count
            }
            Self::BcCounting => fs_stats(nav, peek_on).and_then(|(bcc, _, _)| Some(bcc)),
            Self::CcCounting => fs_stats(nav, peek_on).and_then(|(_, ccc, _)| Some(ccc)),
        }
    }
    fn eval_sharp_restricted<S: ToString>(
        &mut self,
        _nav: &mut T,
        _peek_on: impl Iterator<Item = S>,
        _target: &[S],
    ) -> Option<usize> {
        todo!()
    }
}
