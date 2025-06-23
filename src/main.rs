use std::fs;

use charts_rs::{LineChart, Series, svg_to_png};
use rand_distr::{Distribution, Normal};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

struct SimulationArgs<'a> {
    sims: usize,
    steps: usize,
    signal_distribution: Normal<f64>,
    signal_threshold: f64,
    normal_volatility: Normal<f64>,
    high_volatility: Normal<f64>,
    buckets: &'a Vec<(f64, f64)>,
    play_on_high_volatility: bool,
}

fn main() {
    let step = 0.1;
    let bucket_num = (5.0 / step) as usize;
    let buckets = (0..bucket_num)
        .map(|i| (step * (i as f64), step * (i as f64) + step))
        .collect::<Vec<(f64, f64)>>();

    let signal_distribution = Normal::new(0.0, 0.01).unwrap();

    let outcomes_risk = simulate(SimulationArgs {
        sims: 1000000,
        steps: 300,
        signal_distribution,
        signal_threshold: 0.005,
        normal_volatility: Normal::new(0.0, 0.01).unwrap(),
        high_volatility: Normal::new(0.0, 0.05).unwrap(),
        play_on_high_volatility: true,
        buckets: &buckets,
    });
    let outcomes_safe = simulate(SimulationArgs {
        sims: 1000000,
        steps: 300,
        signal_distribution,
        signal_threshold: 0.005,
        normal_volatility: Normal::new(0.0, 0.01).unwrap(),
        high_volatility: Normal::new(0.0, 0.05).unwrap(),
        play_on_high_volatility: false,
        buckets: &buckets,
    });

    let chart = LineChart::new(
        vec![
            Series::new("risk".to_string(), outcomes_risk),
            Series::new("safe".to_string(), outcomes_safe),
        ],
        buckets
            .iter()
            .map(|x| format!("{:.2}", x.0 + step / 2.0))
            .collect(),
    );
    fs::write("outcomes.png", svg_to_png(&chart.svg().unwrap()).unwrap()).unwrap();
}

fn simulate(args: SimulationArgs) -> Vec<f32> {
    let mut distribution = vec![0usize; args.buckets.len()];

    let outcomes = (0..args.sims)
        .into_par_iter()
        .map_init(
            || rand::rng(),
            |mut rng, _| {
                let mut cash = 1.0;
                for i in 0..args.steps {
                    let signal_level = args.signal_distribution.sample(&mut rng);
                    let high_volatility_day = i % 30 == 20;
                    let signal_error = if high_volatility_day {
                        if args.play_on_high_volatility {
                            args.high_volatility.sample(&mut rng)
                        } else {
                            continue;
                        }
                    } else {
                        args.normal_volatility.sample(&mut rng)
                    };
                    if signal_level > args.signal_threshold {
                        let day_return = 1.0 + signal_level + signal_error;
                        cash *= day_return;
                    }
                }
                cash
            },
        )
        .collect::<Vec<_>>();

    for out in outcomes {
        for (i, bucket) in args.buckets.iter().enumerate() {
            if out >= bucket.0 && out < bucket.1 {
                distribution[i] += 1;
                break;
            }
        }
    }
    dbg!(&distribution);
    distribution.iter().map(|x| *x as f32).collect()
}
