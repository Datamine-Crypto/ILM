use patterns::{because, source};

pub struct CompensatedSum;
source!(CompensatedSum, "the compensated sum of Neumaier, an improved form of the sum of Kahan and Babuska: beside the running sum it keeps the part that each addition loses to rounding and adds it at the end; a sum of many logarithms then has the error of one rounding and not of one for each term, so scores that are equal in exact numbers are compared as equal, and the first of equals is taken as the rule says");

pub fn compensated(values: impl IntoIterator<Item = f64>) -> f64 {
    let mut sum = f64::default();
    let mut lost = f64::default();
    for value in values {
        let next = sum + value;
        if sum.abs() >= value.abs() {
            lost += (sum - next) + value;
        } else {
            lost += (value - next) + sum;
        }
        sum = next;
    }
    if lost != f64::default() && lost.is_finite() {
        sum += lost;
    }
    sum
}
because!(compensated, CompensatedSum, "the sum of some numbers with the lost parts added once at the end; a lost part that is not finite is left out, so that a sum that overflowed stays what it is");
