/// TODO
pub fn weighted_l_t_norm(
    bias: Option<f64>,
    inputs: impl Iterator<Item = f64>,
    weights: Option<impl Iterator<Item = f64>>,
) -> f64 {
    let a = match weights {
        Some(ws) => inputs.zip(ws).map(|(x, w)| w * (1.0 - x)).sum::<f64>(),
        _ => inputs.sum::<f64>(),
    };
    let b = bias.unwrap_or(1.0) - a;

    match b > 1.0 {
        true => 1.0,
        _ => match b < 0.0 {
            true => 0.0,
            _ => b,
        },
    }
}

/// TODO
pub fn weighted_l_t_conorm(
    bias: Option<f64>,
    inputs: impl Iterator<Item = f64>,
    weights: Option<impl Iterator<Item = f64>>,
) -> f64 {
    let a = match weights {
        Some(ws) => inputs.zip(ws).map(|(x, w)| w * x).sum::<f64>(),
        _ => inputs.sum::<f64>(),
    };
    let b = 1.0 - bias.unwrap_or(1.0) + a;

    match b > 1.0 {
        true => 1.0,
        _ => match b < 0.0 {
            true => 0.0,
            _ => b,
        },
    }
}

/// TODO
pub fn weighted_l_residuum(
    bias: Option<f64>,
    x: f64,
    y: f64,
    weights: Option<impl Iterator<Item = f64>>,
) -> f64 {
    let a = match weights {
        Some(mut ws) => ws.next().unwrap_or(1.0) * (1.0 - x) + ws.next().unwrap_or(1.0)*y,
        _ => (1.0 - x) + y,
    };
    let b = 1.0 - bias.unwrap_or(1.0) + a;

    match b > 1.0 {
        true => 1.0,
        _ => match b < 0.0 {
            true => 0.0,
            _ => b,
        },
    }
}
