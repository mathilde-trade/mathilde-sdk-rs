use crate::core::error::SdkError;
use crate::core::pagination::{PaginationAdvance, PaginationState, require_explicit_close_end};
use crate::core::time::TimeInput;
use crate::systems::primitives::client::Primitives;
use crate::systems::primitives::types::{
    RangeGrpcRequest, RangeRequest, RangeResponse, RangeTraverseResult, SearchGrpcRequest,
    SearchRequest, SearchResponse, SearchTraverseResult, TimeMachineGrpcRequest,
    TimeMachineRequest, TimeMachineResponse, TimeMachineTraverseResult,
};

#[derive(Debug)]
pub struct RangeCall<'a> {
    client: &'a Primitives,
    request: RangeRequest,
}

#[derive(Debug)]
pub struct RangeGrpcCall<'a> {
    client: &'a Primitives,
    request: RangeGrpcRequest,
}

#[derive(Debug)]
pub struct SearchCall<'a> {
    client: &'a Primitives,
    request: SearchRequest,
}

#[derive(Debug)]
pub struct SearchGrpcCall<'a> {
    client: &'a Primitives,
    request: SearchGrpcRequest,
}

#[derive(Debug)]
pub struct TimeMachineCall<'a> {
    client: &'a Primitives,
    request: TimeMachineRequest,
}

#[derive(Debug)]
pub struct TimeMachineGrpcCall<'a> {
    client: &'a Primitives,
    request: TimeMachineGrpcRequest,
}

#[derive(Debug)]
pub struct RangePager<'a> {
    client: &'a Primitives,
    request: RangeRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct RangeGrpcPager<'a> {
    client: &'a Primitives,
    request: RangeGrpcRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct SearchPager<'a> {
    client: &'a Primitives,
    request: SearchRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct SearchGrpcPager<'a> {
    client: &'a Primitives,
    request: SearchGrpcRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct TimeMachinePager<'a> {
    client: &'a Primitives,
    request: TimeMachineRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct TimeMachineGrpcPager<'a> {
    client: &'a Primitives,
    request: TimeMachineGrpcRequest,
    state: PaginationState,
    finished: bool,
}

impl<'a> RangeCall<'a> {
    pub(crate) fn new(client: &'a Primitives, request: RangeRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<RangeResponse, SdkError> {
        self.client.range(&self.request).await
    }

    pub fn pager(self) -> Result<RangePager<'a>, SdkError> {
        Ok(RangePager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<RangeTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(RangeTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> RangeGrpcCall<'a> {
    pub(crate) fn new(client: &'a Primitives, request: RangeGrpcRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<RangeResponse, SdkError> {
        self.client.range_grpc(&self.request).await
    }

    pub fn pager(self) -> Result<RangeGrpcPager<'a>, SdkError> {
        Ok(RangeGrpcPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<RangeTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(RangeTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> SearchCall<'a> {
    pub(crate) fn new(client: &'a Primitives, request: SearchRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<SearchResponse, SdkError> {
        self.client.search(&self.request).await
    }

    pub fn pager(self) -> Result<SearchPager<'a>, SdkError> {
        require_explicit_close_end(self.request.close_end.as_ref(), "search")?;
        Ok(SearchPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<SearchTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(SearchTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> SearchGrpcCall<'a> {
    pub(crate) fn new(client: &'a Primitives, request: SearchGrpcRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<SearchResponse, SdkError> {
        self.client.search_grpc(&self.request).await
    }

    pub fn pager(self) -> Result<SearchGrpcPager<'a>, SdkError> {
        require_explicit_close_end(self.request.close_end.as_ref(), "search")?;
        Ok(SearchGrpcPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<SearchTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(SearchTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> TimeMachineCall<'a> {
    pub(crate) fn new(client: &'a Primitives, request: TimeMachineRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<TimeMachineResponse, SdkError> {
        self.client.time_machine(&self.request).await
    }

    pub fn pager(self) -> Result<TimeMachinePager<'a>, SdkError> {
        require_explicit_close_end(self.request.close_end.as_ref(), "time-machine")?;
        Ok(TimeMachinePager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<TimeMachineTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(TimeMachineTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> TimeMachineGrpcCall<'a> {
    pub(crate) fn new(client: &'a Primitives, request: TimeMachineGrpcRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<TimeMachineResponse, SdkError> {
        self.client.time_machine_grpc(&self.request).await
    }

    pub fn pager(self) -> Result<TimeMachineGrpcPager<'a>, SdkError> {
        require_explicit_close_end(self.request.close_end.as_ref(), "time-machine")?;
        Ok(TimeMachineGrpcPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<TimeMachineTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(TimeMachineTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> RangePager<'a> {
    fn new(client: &'a Primitives, request: RangeRequest) -> Self {
        Self {
            client,
            request,
            state: PaginationState::new(),
            finished: false,
        }
    }

    pub fn pages_fetched(&self) -> usize {
        self.state.pages_fetched()
    }

    pub async fn next(&mut self) -> Result<Option<RangeResponse>, SdkError> {
        if self.finished {
            return Ok(None);
        }

        let should_freeze_close_end = self.request.close_end.is_none();
        let response = self.client.range(&self.request).await?;

        if should_freeze_close_end {
            self.request.close_end = Some(TimeInput::from(response.close_end_ms()));
        }

        self.apply_advance(response.next_cursor(), response.done())?;
        Ok(Some(response))
    }

    fn apply_advance(&mut self, next_cursor: Option<&str>, done: bool) -> Result<(), SdkError> {
        match self.state.advance(next_cursor, done)? {
            PaginationAdvance::Continue { cursor } => {
                self.request.cursor = Some(cursor);
            }
            PaginationAdvance::Finished => {
                self.finished = true;
            }
        }

        Ok(())
    }
}

impl<'a> RangeGrpcPager<'a> {
    fn new(client: &'a Primitives, request: RangeGrpcRequest) -> Self {
        Self {
            client,
            request,
            state: PaginationState::new(),
            finished: false,
        }
    }

    pub fn pages_fetched(&self) -> usize {
        self.state.pages_fetched()
    }

    pub async fn next(&mut self) -> Result<Option<RangeResponse>, SdkError> {
        if self.finished {
            return Ok(None);
        }

        let should_freeze_close_end = self.request.close_end.is_none();
        let response = self.client.range_grpc(&self.request).await?;

        if should_freeze_close_end {
            self.request.close_end = Some(TimeInput::from(response.close_end_ms()));
        }

        self.apply_advance(response.next_cursor(), response.done())?;
        Ok(Some(response))
    }

    fn apply_advance(&mut self, next_cursor: Option<&str>, done: bool) -> Result<(), SdkError> {
        match self.state.advance(next_cursor, done)? {
            PaginationAdvance::Continue { cursor } => {
                self.request.cursor = Some(cursor);
            }
            PaginationAdvance::Finished => {
                self.finished = true;
            }
        }

        Ok(())
    }
}

impl<'a> SearchPager<'a> {
    fn new(client: &'a Primitives, request: SearchRequest) -> Self {
        Self {
            client,
            request,
            state: PaginationState::new(),
            finished: false,
        }
    }

    pub fn pages_fetched(&self) -> usize {
        self.state.pages_fetched()
    }

    pub async fn next(&mut self) -> Result<Option<SearchResponse>, SdkError> {
        if self.finished {
            return Ok(None);
        }

        let response = self.client.search(&self.request).await?;
        self.apply_advance(response.next_cursor(), response.done())?;
        Ok(Some(response))
    }

    fn apply_advance(&mut self, next_cursor: Option<&str>, done: bool) -> Result<(), SdkError> {
        match self.state.advance(next_cursor, done)? {
            PaginationAdvance::Continue { cursor } => {
                self.request.cursor = Some(cursor);
            }
            PaginationAdvance::Finished => {
                self.finished = true;
            }
        }

        Ok(())
    }
}

impl<'a> SearchGrpcPager<'a> {
    fn new(client: &'a Primitives, request: SearchGrpcRequest) -> Self {
        Self {
            client,
            request,
            state: PaginationState::new(),
            finished: false,
        }
    }

    pub fn pages_fetched(&self) -> usize {
        self.state.pages_fetched()
    }

    pub async fn next(&mut self) -> Result<Option<SearchResponse>, SdkError> {
        if self.finished {
            return Ok(None);
        }

        let response = self.client.search_grpc(&self.request).await?;
        self.apply_advance(response.next_cursor(), response.done())?;
        Ok(Some(response))
    }

    fn apply_advance(&mut self, next_cursor: Option<&str>, done: bool) -> Result<(), SdkError> {
        match self.state.advance(next_cursor, done)? {
            PaginationAdvance::Continue { cursor } => {
                self.request.cursor = Some(cursor);
            }
            PaginationAdvance::Finished => {
                self.finished = true;
            }
        }

        Ok(())
    }
}

impl<'a> TimeMachinePager<'a> {
    fn new(client: &'a Primitives, request: TimeMachineRequest) -> Self {
        Self {
            client,
            request,
            state: PaginationState::new(),
            finished: false,
        }
    }

    pub fn pages_fetched(&self) -> usize {
        self.state.pages_fetched()
    }

    pub async fn next(&mut self) -> Result<Option<TimeMachineResponse>, SdkError> {
        if self.finished {
            return Ok(None);
        }

        let response = self.client.time_machine(&self.request).await?;
        self.apply_advance(response.next_cursor(), response.done())?;
        Ok(Some(response))
    }

    fn apply_advance(&mut self, next_cursor: Option<&str>, done: bool) -> Result<(), SdkError> {
        match self.state.advance(next_cursor, done)? {
            PaginationAdvance::Continue { cursor } => {
                self.request.cursor = Some(cursor);
            }
            PaginationAdvance::Finished => {
                self.finished = true;
            }
        }

        Ok(())
    }
}

impl<'a> TimeMachineGrpcPager<'a> {
    fn new(client: &'a Primitives, request: TimeMachineGrpcRequest) -> Self {
        Self {
            client,
            request,
            state: PaginationState::new(),
            finished: false,
        }
    }

    pub fn pages_fetched(&self) -> usize {
        self.state.pages_fetched()
    }

    pub async fn next(&mut self) -> Result<Option<TimeMachineResponse>, SdkError> {
        if self.finished {
            return Ok(None);
        }

        let response = self.client.time_machine_grpc(&self.request).await?;
        self.apply_advance(response.next_cursor(), response.done())?;
        Ok(Some(response))
    }

    fn apply_advance(&mut self, next_cursor: Option<&str>, done: bool) -> Result<(), SdkError> {
        match self.state.advance(next_cursor, done)? {
            PaginationAdvance::Continue { cursor } => {
                self.request.cursor = Some(cursor);
            }
            PaginationAdvance::Finished => {
                self.finished = true;
            }
        }

        Ok(())
    }
}
