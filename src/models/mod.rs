pub mod contracts;
pub mod gigs;
pub mod messages;
pub mod portfolio;
pub mod users;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

impl PaginationQuery {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn limit(&self) -> u64 {
        self.limit.unwrap_or(20).min(100)
    }
}
