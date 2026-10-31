use crate::models::Game;

#[derive(Debug, Clone, Default)]
pub struct FilterCriteria {
    pub query: String,
    pub platform: Option<String>,
    pub category: Option<String>,
    pub sort_by: String,
}

pub struct LibraryFilter;

impl LibraryFilter {
    pub fn filter_indices(games: &[Game], criteria: &FilterCriteria) -> Vec<usize> {
        // Real-time case-insensitive filtering in memory
        // Returns original indices to prevent selection desynchronization
    }
}
