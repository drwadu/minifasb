#![deny(clippy::all)]

pub mod lex;
pub mod nav;
#[cfg(feature = "lnn")]
pub mod lofo;
//pub mod incidences;

/// Parses facet.
pub fn parse_facet(exp: &str) -> Option<clingo::Symbol> {
    lex::parse(exp)
}
