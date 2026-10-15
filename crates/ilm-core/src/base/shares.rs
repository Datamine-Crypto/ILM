use patterns::{because, source};

pub struct Hundredths;
source!(Hundredths, "a share of right answers is given in hundredths of a whole, as people read it");

pub const PERCENT: f64 = 100.0;
because!(PERCENT, Hundredths, "a whole share in the parts that people read shares in, so that the bar of a learned kind and a printed share are the same sort of number");

pub fn share(count: usize, of: usize) -> f64 {
    PERCENT * (count as f64) / (of as f64)
}
because!(share, Hundredths, "a count as hundredths of a whole; the product comes before the division, so that a whole share is the round number");
