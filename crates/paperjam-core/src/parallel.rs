use crate::error::Result;

/// Minimum page count to use parallel processing (below this, overhead isn't worth it).
#[cfg(feature = "parallel")]
const PAR_THRESHOLD: u32 = 4;

/// Maps a function over page numbers `1..=count`, returning results in order.
///
/// Uses rayon's `par_iter` when the `parallel` feature is enabled and `count` exceeds
/// the threshold. Otherwise falls back to sequential iteration.
#[cfg(feature = "parallel")]
pub fn par_map_pages<T, F>(count: u32, f: F) -> Vec<Result<T>>
where
    T: Send,
    F: Fn(u32) -> Result<T> + Send + Sync,
{
    if count > PAR_THRESHOLD {
        use rayon::prelude::*;
        (1..=count).into_par_iter().map(&f).collect()
    } else {
        (1..=count).map(f).collect()
    }
}

/// Sequential fallback when the `parallel` feature is disabled.
#[cfg(not(feature = "parallel"))]
pub fn par_map_pages<T, F>(count: u32, f: F) -> Vec<Result<T>>
where
    T: Send,
    F: Fn(u32) -> Result<T> + Send + Sync,
{
    (1..=count).map(f).collect()
}

/// Collects results from `par_map_pages`, short-circuiting on the first error.
pub fn collect_par_results<T>(results: Vec<Result<T>>) -> Result<Vec<T>> {
    results.into_iter().collect()
}
