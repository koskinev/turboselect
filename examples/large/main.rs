use std::time::Instant;

use select::{adaptive_quickselect, floyd_rivest_select_nth};

mod pcg_rng;
use pcg_rng::PCGRng;

/// Generates the test data of `count` random `u32` values.
fn random_u32s(count: usize) -> Vec<u32> {
    let mut data = Vec::with_capacity(u32::MAX as usize);
    let mut rng = PCGRng::new(data.as_ptr() as u64);
    let iter = std::iter::from_fn(move || Some(rng.u32())).take(count);
    data.extend(iter);
    data
}

fn main() {
    eprint!("Generating data...");
    let master = random_u32s(u32::MAX as usize);
    eprintln!("{} elements", master.len());

    eprintln!("Selecting 42nd element...");

    {
        let mut data = master.clone();
        let now = Instant::now();
        floyd_rivest_select_nth(data.as_mut_slice(), 42);
        let elapsed = now.elapsed().as_secs_f32();
        let found = data[42];
        eprintln!("  Floyd & Rivest select_nth: {found} in {elapsed} seconds");
    }

    {
        let mut data = master.clone();
        let now = Instant::now();
        adaptive_quickselect(data.as_mut_slice(), 42);
        let elapsed = now.elapsed().as_secs_f32();
        let found = data[42];
        eprintln!("  Adaptive quickselect: {found} in {elapsed} seconds");
    }

    {
        let mut data = master.clone();
        let now = Instant::now();
        data.select_nth_unstable(42);
        let elapsed = now.elapsed().as_secs_f32();
        let found = data[42];
        eprintln!("  std::slice::select_nth_unstable: {found} in {elapsed} seconds");
    }
}
