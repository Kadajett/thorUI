use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Distribution {
    pub samples: usize,
    pub minimum_ms: f64,
    pub median_ms: f64,
    pub mean_ms: f64,
    pub p95_ms: f64,
    pub maximum_ms: f64,
    pub estimated_hz: f64,
    pub over_budget: usize,
}

pub fn summarize(values: &[f64], budget_ms: f64) -> Distribution {
    let mut sorted: Vec<f64> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite() && *value >= 0.0)
        .collect();
    if sorted.is_empty() {
        return Distribution::default();
    }
    sorted.sort_by(f64::total_cmp);
    let sum: f64 = sorted.iter().sum();
    let sample_count = u32::try_from(sorted.len()).unwrap_or(u32::MAX);
    let mean = sum / f64::from(sample_count);
    Distribution {
        samples: sorted.len(),
        minimum_ms: round(sorted[0]),
        median_ms: round(percentile(&sorted, 50)),
        mean_ms: round(mean),
        p95_ms: round(percentile(&sorted, 95)),
        maximum_ms: round(sorted[sorted.len() - 1]),
        estimated_hz: if mean > 0.0 {
            round(1_000.0 / mean)
        } else {
            0.0
        },
        over_budget: sorted.iter().filter(|value| **value > budget_ms).count(),
    }
}

fn percentile(sorted: &[f64], percentile: usize) -> f64 {
    let index = ((sorted.len() - 1) * percentile + 50) / 100;
    sorted[index]
}

fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::{Distribution, summarize};

    fn close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 0.001);
    }

    #[test]
    fn summarizes_a_frame_distribution() {
        let result = summarize(&[8.0, 8.2, 8.4, 20.0], 9.0);
        assert_eq!(result.samples, 4);
        close(result.minimum_ms, 8.0);
        close(result.maximum_ms, 20.0);
        assert_eq!(result.over_budget, 1);
        close(result.estimated_hz, 89.69);
    }

    #[test]
    fn ignores_invalid_samples() {
        let result = summarize(&[f64::NAN, -1.0, 10.0], 16.7);
        assert_eq!(result.samples, 1);
        close(result.mean_ms, 10.0);
    }

    #[test]
    fn returns_default_for_no_samples() {
        assert_eq!(summarize(&[], 8.34), Distribution::default());
    }
}
