use std::collections::HashSet;

use super::nav::faceted_navigation::{fs, FacetedNavigation};
use clingo::Symbol;

/// TODO
pub fn show<T>(structure: &mut impl Incidences<T>, nav: &mut T) {
    structure.show(nav)
}

#[derive(Debug, Clone)]
pub enum Structure {
    F(Vec<Symbol>),
    B(Vec<Symbol>),
    C(Vec<Symbol>),
}

pub trait Incidences<T> {
    fn show(&mut self, nav: &mut T);
    fn ret<S: ToString>(
        &mut self,
        nav: &mut T,
        peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
        target: &[S],
    ) -> Option<(usize, Option<usize>)>;
}

impl<T: FacetedNavigation> Incidences<T> for Structure {
    fn show(&mut self, nav: &mut T) {
        match self {
            Self::F(ord) => {
                let xs = match ord.is_empty() {
                    true => fs(nav, (std::iter::empty::<String>(), std::iter::empty()))
                        .unwrap_or(HashSet::new())
                        .into_iter()
                        .collect::<Vec<_>>(),
                    _ => ord.to_vec(),
                };
                xs.iter()
                    .map(|f| fs(nav, ([f].iter(), std::iter::empty())))
                    .flatten()
                    .for_each(|fs| {
                        xs.iter().for_each(|f| match fs.contains(f) {
                            true => print!("x"),
                            _ => print!(" "),
                        });
                        println!()
                    });
            }
            _ => todo!(),
        }
    }
    fn ret<S: ToString>(
        &mut self,
        _nav: &mut T,
        _peek_on: (impl Iterator<Item = S>, impl Iterator<Item = S>),
        _target: &[S],
    ) -> Option<(usize, Option<usize>)> {
        todo!()
    }
}
