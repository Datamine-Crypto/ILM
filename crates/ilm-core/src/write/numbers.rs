use patterns::{because, source};

pub struct FixedArithmetic;
source!(FixedArithmetic, "the arithmetic of a learner that decides by counts: every bound is tested with numbers of the common double width in one fixed order of operations, so that the same lessons give the same model on every run and with every number of threads; the logarithm, the power of the base of the logarithm and the power of a number come from the C library of the machine");

pub struct PoissonTail;
source!(PoissonTail, "the law of small numbers of Poisson: the chance that a count is at most a given number when a mean is expected; a value is taken as surely rare in a place by this chance");

pub struct ChernoffDistance;
source!(ChernoffDistance, "the bound of Chernoff on the tail of a binomial count, written with the distance of Kullback and Leibler between two shares: a count of so many at one share has at most the chance of the base of the logarithm to the power of minus that many distances when the true share is the other one");

pub fn natural_log(number: f64) -> f64 {
    number.ln()
}
because!(natural_log, FixedArithmetic, "the natural logarithm as one call in one place, so that every count that is compared by logarithms uses the same function");

pub fn e_to(number: f64) -> f64 {
    number.exp()
}
because!(e_to, FixedArithmetic, "the base of the natural logarithm to a power as one call in one place, for the tail of the law of small numbers and for a mean of logarithms");

pub fn to_power(base: f64, power: f64) -> f64 {
    base.powf(power)
}
because!(to_power, FixedArithmetic, "one number to the power of another by the call of the C library, also for a whole power: a product in a loop rounds in another way, and the order of equal texts could turn on that");

pub fn poisson_at_most(count: i64, mean: f64) -> f64 {
    let mut term = e_to(-mean);
    let mut total = term;
    let mut k = f64::default();
    for _ in 0..count.max(0) {
        k += f64::from(true);
        term *= mean / k;
        total += term;
    }
    total
}
because!(poisson_at_most, PoissonTail, "the chance of a count of at most this many when the mean is expected, summed term by term from nought upward, so that the same count and mean give the same chance to the last bit");

pub fn share_distance(got: f64, share: f64) -> f64 {
    let mut out = f64::default();
    let one = f64::from(true);
    if got > f64::default() {
        out += got * natural_log(got / share);
    }
    if got < one {
        out += (one - got) * natural_log((one - got) / (one - share));
    }
    out
}
because!(share_distance, ChernoffDistance, "the distance between the share that was counted and the share that chance would give; the count of texts times this distance bounds the chance of the count from above");
