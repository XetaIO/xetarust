//! DTOs shared by several contexts (the "published language" of the API).
//!
//! TypeScript bindings are generated in `frontend/src/types/api/shared/`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::pagination::{Page, PageRequest};

/// Pagination query string (`?page=2&per_page=10`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, TS)]
#[ts(export, export_to = "shared/", optional_fields = nullable)]
pub struct PageQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl PageQuery {
    /// Converts the query into a normalized page request.
    pub fn to_page_request(&self) -> PageRequest {
        PageRequest::new(self.page, self.per_page)
    }
}

/// A page of items plus the metadata needed to render a paginator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "shared/")]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

impl<T> Paginated<T> {
    /// Builds the DTO from a page, converting each item with `f`.
    pub fn from_page<U>(page: Page<U>, f: impl FnMut(U) -> T) -> Self {
        let total_pages = page.total_pages();
        let request = page.request;
        let page = page.map(f);

        Self {
            items: page.items,
            total: page.total,
            page: request.page(),
            per_page: request.per_page(),
            total_pages,
        }
    }
}

/// Public identity of the author of an article or a comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "shared/")]
pub struct AuthorDto {
    pub id: Uuid,
    pub username: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_page_copies_metadata() {
        let page = Page {
            items: vec![1, 2, 3],
            total: 23,
            request: PageRequest::new(Some(2), Some(3)),
        };
        let dto = Paginated::from_page(page, |n| n.to_string());
        assert_eq!(dto.items, vec!["1", "2", "3"]);
        assert_eq!(
            (dto.total, dto.page, dto.per_page, dto.total_pages),
            (23, 2, 3, 8)
        );
    }
}
