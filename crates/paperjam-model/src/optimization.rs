/// Options controlling PDF optimization.
pub struct OptimizeOptions {
    /// Compress uncompressed streams using FlateDecode.
    pub compress_streams: bool,
    /// Remove objects not referenced by any page or the catalog.
    pub remove_unused_objects: bool,
    /// Remove duplicate stream objects.
    pub remove_duplicates: bool,
    /// Strip document metadata (/Info and /Metadata).
    pub strip_metadata: bool,
}

impl Default for OptimizeOptions {
    fn default() -> Self {
        Self {
            compress_streams: true,
            remove_unused_objects: true,
            remove_duplicates: true,
            strip_metadata: false,
        }
    }
}

/// Result statistics from optimization.
pub struct OptimizeResult {
    pub original_size: usize,
    pub optimized_size: usize,
    pub objects_removed: usize,
    pub streams_compressed: usize,
}
