use super::nav::weighted_navigation::{count, Weight};
use super::nav::{Essential, Navigation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lnn(pub HashMap<String, Neuron>);
impl Lnn {
    #[allow(unused)]
    pub fn add(&mut self, neuron: Neuron) {
        self.0.insert(neuron.repr.clone(), neuron);
    }

    #[allow(unused)]
    pub fn get(&self, repr: impl ToString) -> Neuron {
        unsafe { self.0.get(&repr.to_string()).unwrap_unchecked() }.clone()
    }

    #[allow(unused)]
    pub fn update_state(&mut self, repr: impl ToString, state: (f64, f64)) {
        if let Some(m) = self.0.get_mut(&repr.to_string()) {
            m.state = state;
        }
    }

    #[allow(unused)]
    pub fn update_bounds(&mut self, wrt: &mut Navigation, soft: bool) {
        wrt.update(); // TODO:
        let mut w = match soft {
            true => Weight::FacetCounting,
            _ => Weight::AnswerSetCounting,
        };
        let ovr =
            unsafe { count(&mut w, wrt, std::iter::empty::<String>()).unwrap_unchecked() }.0 as f64;
        self.0
            .clone()
            .values_mut()
            .filter(|n| n.f == Activation::Proposition)
            .for_each(|n| {
                let c = unsafe { count(&mut w, wrt, [&n.repr].iter()).unwrap_unchecked() }.0 as f64
                    / ovr;
                self.update_state(n.repr.clone(), (c, c));
                self.update_state(format!("~{}", n.repr), (1.0 - c, 1.0 - c))
            });
    }

    #[allow(unused)]
    pub fn update_bound(&mut self, n: &Neuron, wrt: &mut Navigation, w: &mut Weight) {
        wrt.update(); // TODO:
        let bound = if n.f == Activation::Implies {
            1.0
        } else {
            let ovr =
                unsafe { count(w, wrt, std::iter::empty::<String>()).unwrap_unchecked() }.0 as f64;
            unsafe { count(w, wrt, [&n.repr].iter()).unwrap_unchecked() }.0 as f64 / ovr
        };

        self.update_state(n.repr.clone(), (bound, bound))
    }

    #[allow(unused)]
    pub fn update_bound_to_feed(
        &mut self,
        neuron: String,
        wrt: &mut Navigation,
        w: &mut Weight,
    ) -> f64 {
        wrt.update(); // TODO:
        let n = self.get(neuron);
        let bound = if n.f == Activation::Implies {
            1.0
        } else {
            let ovr =
                unsafe { count(w, wrt, std::iter::empty::<String>()).unwrap_unchecked() }.0 as f64;
            unsafe { count(w, wrt, [&n.repr].iter()).unwrap_unchecked() }.0 as f64 / ovr
        };

        self.update_state(n.repr.clone(), (bound, bound));
        bound
    }

    #[allow(unused)]
    pub fn show_bounds(&self) {
        for (k, v) in self.0.iter() {
            if v.f == Activation::Proposition || v.f == Activation::Not {
                println!("{k} {:?}", v.state)
            }
        }
    }
}

#[allow(unused)]
fn trim_repr(s: impl AsRef<str>) -> String {
    s.as_ref()
        .replace("¬", "~")
        .replace(" ∧ ", "&")
        .replace(" ∨ ", "|")
        .replace(" → ", ">>")
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Neuron {
    pub repr: String,
    pub alpha: f64,
    bias: f64,
    pub state: (f64, f64),
    weights: Vec<f64>,
    pub f: Activation,
}
impl Neuron {
    #[allow(unused)]
    pub fn evaluate(&mut self, inputs: &[f64]) -> f64 {
        eval(self.f.clone(), self.bias, inputs, &self.weights)
    }

    #[allow(unused)]
    pub fn tune_knob(&mut self, tune_to: f64) {
        self.alpha = match tune_to > 0.5 {
            true => tune_to,
            _ => self.alpha,
        }
    }

    #[allow(unused)]
    /// Recursively feeds formula.
    pub fn upward(&self, wrt: &mut Lnn, nav: &mut Navigation, mut w: &mut Weight) -> f64 {
        println!("{:?}", self);
        match self.f {
            Activation::And => {
                let s = &mut self.repr[1..].to_string();
                s.pop();
                let inputs = s.clone().replace(")", "");
                let bound = (self.bias
                    - inputs
                        .split("&")
                        .map(|n| wrt.get(&n).upward(wrt, nav, &mut w))
                        .zip(self.weights.iter())
                        .map(|(x, w)| w * (1.0 - x))
                        .sum::<f64>())
                .min(1.0)
                .max(0.0);
                wrt.update_state(&self.repr, (bound, bound));
                bound
            }
            Activation::Or => {
                let s = &mut self.repr[1..].to_string();
                s.pop();
                let inputs = s.clone().replace(")", "");
                let bound = (1.0 - self.bias
                    + inputs
                        .split("|")
                        .map(|n| wrt.get(&n).upward(wrt, nav, &mut w))
                        .zip(self.weights.iter())
                        .map(|(x, w)| w * x)
                        .sum::<f64>())
                .min(1.0)
                .max(0.0);
                wrt.update_state(&self.repr, (bound, bound));
                bound
            }
            Activation::Implies => {
                let s = &mut self.repr[1..].to_string();
                s.pop();
                let ac = s.clone().replace(")", "");
                let mut inputs = ac.split(">>").map(|n| wrt.get(&n).upward(wrt, nav, &mut w));
                let mut weights = self.weights.iter();
                let bound = unsafe {
                    (1.0 - self.bias
                        + (((1.0 - inputs.next().unwrap_unchecked())
                            * weights.next().unwrap_unchecked())
                            + (inputs.next().unwrap_unchecked()
                                * weights.next().unwrap_unchecked())))
                    .min(1.0)
                    .max(0.0)
                };
                wrt.update_state(&self.repr, (bound, bound));
                bound
            }
            Activation::Not => {
                let bound = 1.0 - wrt.get(&self.repr[1..]).upward(wrt, nav, &mut w);
                wrt.update_state(&self.repr, (bound, bound));
                bound
            }
            Activation::Proposition => wrt.update_bound_to_feed(self.repr.clone(), nav, w),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Activation {
    Proposition,
    Not,
    And,
    Or,
    Implies,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum State {
    Unknown,
    True,
    False,
    Contradicting,
}

#[allow(unused)]
pub fn parse_neuron(neuron_print: impl AsRef<str>) -> Neuron {
    serde_yaml::from_str(&neuron_print.as_ref()).expect("")
}

#[allow(unused)]
fn eval(f: Activation, bias: f64, inputs: &[f64], weights: &[f64]) -> f64 {
    match f {
        Activation::Or => {
            relu(1.0 - bias + inputs.iter().zip(weights).map(|(x, w)| x * w).sum::<f64>())
        }
        Activation::And => relu(
            bias - inputs
                .iter()
                .zip(weights)
                .map(|(x, w)| (1.0 - x) * w)
                .sum::<f64>(),
        ),
        Activation::Implies => relu(
            1.0 - bias
                + ((1.0 - inputs.get(0).unwrap_or(&f64::NAN)) + inputs.get(1).unwrap_or(&f64::NAN)),
        ),
        Activation::Not => 1.0 - *inputs.get(0).unwrap_or(&f64::NAN),
        Activation::Proposition => *inputs.get(0).unwrap_or(&f64::NAN),
    }
}

#[allow(unused)]
fn relu(x: f64) -> f64 {
    x.min(1.0).max(0.0)
}


#[allow(unused)]
pub fn state_neuron(neuron: &Neuron) -> State {
    let (l, u) = neuron.state;
    match l > u {
        true => State::Contradicting,
        _ => match (l <= neuron.alpha && l <= neuron.alpha)
            && (u <= neuron.alpha && u <= neuron.alpha)
        {
            true => State::True,
            _ => match (l <= (1.0 - neuron.alpha)) && (u <= (1.0 - neuron.alpha)) {
                true => State::False,
                _ => State::Unknown,
            },
        },
    }
}

#[allow(unused)]
pub fn translate_info(info: impl AsRef<str>) -> Option<String> {
    let mut yaml = vec![];
    let mut s = info
        .as_ref()
        .split("\n")
        .skip(3)
        .take_while(|l| !l.starts_with("*"));
    while let Some(l) = s.next() {
        let mut ts = l.split(":");
        let f = ts.next()?.split(" ").nth(1)?;
        let delimiter = match ["APPROX_UNKNOWN", "APPROX_TRUE", "APPROX_FALSE"]
            .iter()
            .find(|s| l.contains(*s))
        {
            Some(d) => d,
            _ => ["TRUE", "FALSE", "UNKNOWN"]
                .iter()
                .find(|s| l.contains(*s))?,
        };
        let mut rest = ts.next()?[1..].split(delimiter);
        let repr = trim_repr(rest.next()?.trim());
        let state = &rest.next()?.replace("(", "").replace(")", "")[1..];
        let mut state_iter = state.split(", ");
        let (l, u) = (state_iter.next()?, state_iter.next()?);
        let params = &s.next()?[8..];
        let mut params_iter = params.split(", ");
        if f == "Proposition" || f == "Not" {
            let alpha = params_iter.next()?.trim().split(":").nth(1)?;
            yaml.push(format!("repr: {repr}\nalpha: {alpha}\nbias: {:?}\nstate:\n- {l}\n- {u}\nweights:\n- {:?}\nf: {f}", 
            0.0, 0.0));
            //f64::NAN, f64::NAN));
        } else {
            let (alpha, bias, weights) = (
                params_iter.next()?.trim().split(":").nth(1)?,
                params_iter.next()?.trim().split(":").nth(1)?,
                &params_iter
                    .next()?
                    .trim()
                    .split(":")
                    .nth(1)?
                    .replace("[", "")
                    .replace("]", "")[1..],
            );
            yaml.push(format!("repr: {repr}\nalpha: {alpha}\nbias: {bias}\nstate:\n- {l}\n- {u}\nweights:\n- {}\nf: {f}", 
            weights.split(" ").collect::<Vec<&str>>().join("\n- ")).lines().filter(|s| *s!="- ").collect::<Vec<&str>>().join("\n"));
        }
    }

    Some(yaml.join("\n*\n"))
}
