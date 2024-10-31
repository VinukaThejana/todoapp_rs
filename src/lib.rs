pub struct Paginator {
    pub skip: u64,
    pub take: u64,
}

impl Default for Paginator {
    fn default() -> Self {
        Self { skip: 0, take: 10 }
    }
}
