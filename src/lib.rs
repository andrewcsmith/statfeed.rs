extern crate rand;

use rand::random;
use std::cmp::Ordering;

pub struct Statfeed<T> {
    pub randoms: Vec<Vec<f64>>,
    pub weights: Vec<Vec<f64>>,
    pub choices: Vec<T>,
    pub heterogeneities: Vec<f64>,
    accents: Vec<f64>,
    statistics: Vec<f64>,
    decisions: Vec<usize>,
    options: Vec<T>
}

impl<T: Clone> Statfeed<T> {
    pub fn new(options: Vec<T>, size: usize) -> Self {
        // Default to weights of 1.0 for each decision/option
        let weights: Vec<Vec<f64>> = (0..size).map(|_| {
            (0..options.len()).map(|_| 1.0 / options.len() as f64).collect()
        }).collect();
        // Generate random numbers
        let randoms: Vec<Vec<f64>> = (0..size).map(|_| {
            (0..options.len()).map(|_| random::<f64>()).collect()
        }).collect();
        let heterogeneities = vec![0.1f64; size];
        let accents = vec![1f64; size];
        let statistics  = vec![0.0; options.len()];
        let choices: Vec<T> = Vec::with_capacity(size);
        let decisions: Vec<usize> = (0..size).collect();

        Statfeed {
            weights: weights,
            randoms: randoms,
            heterogeneities: heterogeneities,
            accents: accents,
            statistics: statistics,
            choices: choices,
            decisions: decisions,
            options: options
        }
    }

    pub fn populate_choices(&mut self) {
        self.choices.clear();
        for dec in 0..self.decisions.len() {
            let (choice, index) = {
                let options = self.sort_options(&self.scheduling_values(dec)[..]);
                let index = options.iter().position(|el| self.is_acceptable(el, dec)).unwrap();
                (options[index].clone(), index)
            };
            self.choices.push(choice);
            self.increment_statistics(dec, index);
            self.normalize_statistics(dec);
        }
    }

    fn is_acceptable(&self, el: &T, idx: usize) -> bool {
        true
    }

    fn increment_statistics(&mut self, dec: usize, idx: usize) {
        self.statistics[idx] += self.true_increment(dec, idx);
    }

    fn normalize_statistics(&mut self, dec: usize) {
        for idx in 0..self.statistics.len() {
            if self.weights[dec][idx] > 0.0 {
                self.statistics[idx] -= self.normalization_value(dec);
            } 
        }
    }

    fn normalization_value(&self, idx: usize) -> f64 {
        self.accents[idx] / self.weights[idx].iter().fold(0., |a, v| a + v)
    }

    fn true_increment(&self, decision: usize, option: usize) -> f64 {
        self.accents[decision] / self.weights[decision][option]
    }

    fn expected_increment(&self, decision: usize, option: usize) -> f64 {
        (self.accents[decision] + 
         (self.heterogeneities[decision] * self.randoms[decision][option])) 
        / self.weights[decision][option]
    }

    fn scheduling_values(&self, decision: usize) -> Vec<f64> {
        (0..self.options.len()).map(|m| {
            self.statistics[m] + self.expected_increment(decision, m)
        }).collect()
    }

    fn sort_options(&self, vals: &[f64]) -> Vec<&T> {
        let mut ov: Vec<(&T, &f64)> = self.options.iter().zip(vals.iter()).collect();
        ov.sort_by(|&(_, v), &(_, v2)| v.partial_cmp(v2).unwrap_or(Ordering::Equal));
        ov.iter().map(|&(o, _)| o).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Into;
    use std::f64;
    use super::*;

    fn setup() -> Statfeed<char> {
        let mut sf = Statfeed::new(vec!['a', 'b', 'c'], 3);
        sf.randoms = vec![vec![0.1, 0.2, 0.3], vec![0.4, 0.5, 0.6], vec![0.7, 0.8, 0.9]];
        sf
    }

    fn assert_in_delta(left: f64, right: f64, delta: f64) {
        assert!((left - right).abs() < delta);
    }

    #[test]
    fn test_true_increment() {
        let sf = setup();
        let res = sf.true_increment(0, 0);
        println!("true_increment: {}", &res);
        assert_in_delta(3.0, res, 1.0e-10);
    }

    #[test]
    fn test_expected_increment() {
        let sf = setup();
        let res = sf.expected_increment(0, 0);
        println!("expected_increment: {}", &res);
        assert_in_delta(3.03, res, 1.0e-10);
    }

    #[test]
    fn test_scheduling_values() {
        let actual = setup().scheduling_values(0);
        for (exp, res) in [3.03, 3.06, 3.09].iter().zip(actual.iter()) {
            println!("scheduling_values: {}", res);
            assert_in_delta(*exp, *res, 1.0e-10);
        }
    }

    #[test]
    fn test_sort_options() {
        let sf = setup();
        let actual = sf.sort_options(&[0.3, 0.5, 0.4]);
        println!("sort_options: {:?}", &actual);
        assert_eq!(3, actual.len());
        for (exp, res) in ['a', 'c', 'b'].iter().zip(actual.iter()) {
            assert_eq!(exp, *res);
        }
    }

    #[test]
    fn test_populate_choices() {
        let mut sf = setup();
        sf.populate_choices();
        println!("choices: {:?}", &sf.choices);
        assert_eq!(3, sf.choices.len());
        for (exp, res) in ['a', 'b', 'b'].iter().zip(sf.choices.iter()) {
            assert_eq!(exp, res);
        }
    }
}
