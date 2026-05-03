use crate::core::error::SdkError;

pub const DEFAULT_TRAVERSE_MAX_PAGES: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaginationAdvance {
    Continue { cursor: String },
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationState {
    pages_fetched: usize,
    max_pages: usize,
    last_cursor: Option<String>,
}

impl Default for PaginationState {
    fn default() -> Self {
        Self::new()
    }
}

impl PaginationState {
    pub fn new() -> Self {
        Self::with_max_pages(DEFAULT_TRAVERSE_MAX_PAGES)
    }

    pub fn with_max_pages(max_pages: usize) -> Self {
        Self {
            pages_fetched: 0,
            max_pages,
            last_cursor: None,
        }
    }

    pub fn pages_fetched(&self) -> usize {
        self.pages_fetched
    }

    pub fn max_pages(&self) -> usize {
        self.max_pages
    }

    pub fn advance(
        &mut self,
        next_cursor: Option<&str>,
        done: bool,
    ) -> Result<PaginationAdvance, SdkError> {
        if self.pages_fetched >= self.max_pages {
            return Err(SdkError::contract_drift(format!(
                "traversal page budget exhausted at {} pages",
                self.max_pages
            )));
        }

        self.pages_fetched += 1;

        if done {
            return Ok(PaginationAdvance::Finished);
        }

        let Some(cursor) = next_cursor else {
            return Ok(PaginationAdvance::Finished);
        };

        if self.last_cursor.as_deref() == Some(cursor) {
            return Err(SdkError::contract_drift(format!(
                "cursor repeated without progress: {cursor}"
            )));
        }

        let cursor = cursor.to_string();
        self.last_cursor = Some(cursor.clone());
        Ok(PaginationAdvance::Continue { cursor })
    }
}

pub fn require_explicit_close_end<T>(
    close_end: Option<&T>,
    endpoint_name: &str,
) -> Result<(), SdkError> {
    if close_end.is_some() {
        return Ok(());
    }

    Err(SdkError::unsupported_or_unproved_usage(format!(
        "{endpoint_name} traversal requires explicit close_end"
    )))
}
