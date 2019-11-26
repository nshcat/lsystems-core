use rand::*;
use rand::distributions::*;
use rand::distributions::Distribution;

#[derive(Copy, Clone, Debug)]
pub struct Weighted<T> {
    pub weight: f64,
    pub item: T,
}

#[derive(Debug)]
pub struct WeightedChoice<'a, T:'a> {
    items: &'a mut [Weighted<T>],
    weight_range: Uniform<f64>,
}

impl<'a, T: Clone> WeightedChoice<'a, T> {
    pub fn new(items: &'a mut [Weighted<T>]) -> WeightedChoice<'a, T> {
        assert!(!items.is_empty(), "WeightedChoice::new called with no items");

        let mut running_total: f64 = 0.0;

        for item in items.iter_mut() {
            running_total += item.weight;
            item.weight = running_total;
        }
        assert!(running_total != 0.0, "WeightedChoice::new called with a total weight of 0");

        WeightedChoice {
            items,
            // we're likely to be generating numbers in this range
            // relatively often, so might as well cache it
            weight_range: Uniform::new(0.0, running_total)
        }
    }
}

impl<'a, T: Clone> Distribution<T> for WeightedChoice<'a, T> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        let sample_weight = self.weight_range.sample(rng);

        if sample_weight < self.items[0].weight {
            return self.items[0].item.clone();
        }

        let mut idx = 0;
        let mut modifier = self.items.len();

        while modifier > 1 {
            let i = idx + modifier / 2;
            if self.items[i].weight <= sample_weight {
                idx = i;
                modifier += 1;
            } else {
            }
            modifier /= 2;
        }
        self.items[idx + 1].item.clone()
    }
}
