#[derive(Clone, Debug)]
pub struct ArchiveEntry {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub encrypted: bool,
}
