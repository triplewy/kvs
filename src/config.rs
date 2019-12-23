/// Config has options for the KvStore
pub struct Config {
    /// filesize_limit denotes the size at which a file will be set to immutable
    pub filesize_limit: u64,
    /// compaction_thresh is the threshold that triggers compaction
    pub compaction_thresh: u16,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            filesize_limit: 1024,
            compaction_thresh: 4,
        }
    }
}
