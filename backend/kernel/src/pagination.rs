//! Pagination primitives shared by every repository and use case.

/// Maximum number of items a single page may contain.
pub const MAX_PER_PAGE: u64 = 50;

/// Default number of items per page when the caller does not specify it.
pub const DEFAULT_PER_PAGE: u64 = 10;

/// A normalized, always valid pagination request (1-based page index).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageRequest {
    page: u64,
    per_page: u64,
}

impl PageRequest {
    /// Creates a request, clamping `page` to at least 1 and `per_page` to `1..=MAX_PER_PAGE`.
    pub fn new(page: Option<u64>, per_page: Option<u64>) -> Self {
        Self {
            page: page.unwrap_or(1).max(1),
            per_page: per_page.unwrap_or(DEFAULT_PER_PAGE).clamp(1, MAX_PER_PAGE),
        }
    }

    /// Returns the 1-based page index.
    pub fn page(&self) -> u64 {
        self.page
    }

    /// Returns the number of items per page.
    pub fn per_page(&self) -> u64 {
        self.per_page
    }

    /// Returns the number of rows to skip before this page starts.
    pub fn offset(&self) -> u64 {
        (self.page - 1) * self.per_page
    }
}

impl Default for PageRequest {
    /// Returns the first page with the default page size.
    fn default() -> Self {
        Self::new(None, None)
    }
}

/// One page of results plus the metadata needed to render a paginator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub request: PageRequest,
}

impl<T> Page<T> {
    /// Returns the total number of pages (at least 1, even when empty).
    pub fn total_pages(&self) -> u64 {
        self.total.div_ceil(self.request.per_page()).max(1)
    }

    /// Transforms every item of the page while keeping the metadata.
    pub fn map<U>(self, f: impl FnMut(T) -> U) -> Page<U> {
        Page {
            items: self.items.into_iter().map(f).collect(),
            total: self.total,
            request: self.request,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_first_page_with_default_size() {
        let request = PageRequest::default();
        assert_eq!(request.page(), 1);
        assert_eq!(request.per_page(), DEFAULT_PER_PAGE);
        assert_eq!(request.offset(), 0);
    }

    #[test]
    fn clamps_out_of_range_values() {
        let request = PageRequest::new(Some(0), Some(1_000));
        assert_eq!(request.page(), 1);
        assert_eq!(request.per_page(), MAX_PER_PAGE);

        let request = PageRequest::new(Some(3), Some(0));
        assert_eq!(request.per_page(), 1);
    }

    #[test]
    fn computes_offset_from_page_and_size() {
        assert_eq!(PageRequest::new(Some(3), Some(10)).offset(), 20);
    }

    #[test]
    fn computes_total_pages() {
        let page = |total| Page::<()> {
            items: vec![],
            total,
            request: PageRequest::new(Some(1), Some(10)),
        };
        assert_eq!(page(0).total_pages(), 1);
        assert_eq!(page(10).total_pages(), 1);
        assert_eq!(page(11).total_pages(), 2);
    }

    #[test]
    fn map_keeps_metadata() {
        let page = Page {
            items: vec![1, 2],
            total: 12,
            request: PageRequest::new(Some(2), Some(2)),
        };
        let mapped = page.map(|n| n * 10);
        assert_eq!(mapped.items, vec![10, 20]);
        assert_eq!(mapped.total, 12);
        assert_eq!(mapped.request.page(), 2);
    }
}
