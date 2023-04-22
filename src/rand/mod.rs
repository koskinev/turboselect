mod pcg;
mod rng;

pub(crate) use pcg::PCGRng;
pub use rng::Rng;

#[cfg(test)]
mod tests;
