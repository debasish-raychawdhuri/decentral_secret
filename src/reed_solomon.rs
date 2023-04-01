use linearalgebra::{binary::BinaryField, Field, Ring};
use proptest::prelude::*;
use urandom::csprng;
pub struct Polynomial {
    coefficients: Vec<u64>,
}

impl Polynomial {
    pub fn random(degree: usize, constant_term: u64) -> Self {
        let mut coefficients = Vec::with_capacity(degree);
        coefficients.push(constant_term);
        let mut prng = csprng();
        for _ in 1..degree {
            let c = prng.next();
            coefficients.push(c);
        }
        Polynomial { coefficients }
    }
    pub fn evaluate_at_points(&self, points: &[u64]) -> Vec<(u64, u64)> {
        points
            .iter()
            .map(|x| {
                let mut v = 1u64;
                let mut sum = 0u64;
                let mut field = BinaryField::new();
                for i in 0..self.coefficients.len() {
                    sum = field.add(&sum, &field.mul(&v, &self.coefficients[i]));
                    v = field.mul(&v, x);
                }
                (*x, sum)
            })
            .collect()
    }
    pub fn interpolate_constant_term(values: &[(u64, u64)]) -> u64 {
        let ring = BinaryField::new();
        values
            .iter()
            .map(|(x, y)| {
                let denom = values
                    .iter()
                    .filter(|(x_, y_)| *x_ != *x)
                    .map(|(x_, y_)| ring.add(x, x_))
                    .fold(1u64, |a, b| ring.mul(&a, &b));
                let numerator = values
                    .iter()
                    .filter(|(x_, y_)| *x_ != *x)
                    .map(|(x_, y_)| *x_)
                    .fold(1u64, |a, b| ring.mul(&a, &b));
                let n = ring.mul(&numerator, y);
                ring.mul(&n, &ring.inv(&denom).ok().unwrap())
            })
            .fold(0u64, |a, b| ring.add(&a, &b))
    }
}
proptest! {
    #[test]
    fn test_interpolation(c:u64, points:Vec<u64>){
        if points.len()>0{
            let random = Polynomial::random(points.len(),c);
            let evaluations = random.evaluate_at_points(&points);
            let interpolated_c = Polynomial::interpolate_constant_term(&evaluations);
            assert_eq!(c, interpolated_c);

        }
   }

}
