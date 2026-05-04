use crate::core::error::SdkError;
use crate::core::pagination::{PaginationAdvance, PaginationState, require_explicit_close_end};
use crate::core::time::TimeInput;
use crate::systems::primitives::client::Primitives;
use crate::systems::primitives::types::{
    RangeOutputsGrpcRequest, RangeOutputsRequest, RangeOutputsResponse, RangeOutputsTraverseResult,
    SearchOutputsGrpcRequest, SearchOutputsRequest, SearchOutputsResponse,
    SearchOutputsTraverseResult, TimeMachineOutputsGrpcRequest, TimeMachineOutputsRequest,
    TimeMachineOutputsResponse, TimeMachineOutputsTraverseResult,
};

#[derive(Debug)]
pub struct RangeOutputsCall<'a> {
    client: &'a Primitives,
    request: RangeOutputsRequest,
}

#[derive(Debug)]
pub struct RangeOutputsGrpcCall<'a> {
    client: &'a Primitives,
    request: RangeOutputsGrpcRequest,
}

#[derive(Debug)]
pub struct SearchOutputsCall<'a> {
    client: &'a Primitives,
    request: SearchOutputsRequest,
}

#[derive(Debug)]
pub struct SearchOutputsGrpcCall<'a> {
    client: &'a Primitives,
    request: SearchOutputsGrpcRequest,
}

#[derive(Debug)]
pub struct TimeMachineOutputsCall<'a> {
    client: &'a Primitives,
    request: TimeMachineOutputsRequest,
}

#[derive(Debug)]
pub struct TimeMachineOutputsGrpcCall<'a> {
    client: &'a Primitives,
    request: TimeMachineOutputsGrpcRequest,
}

#[derive(Debug)]
pub struct RangeOutputsPager<'a> {
    client: &'a Primitives,
    request: RangeOutputsRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct RangeOutputsGrpcPager<'a> {
    client: &'a Primitives,
    request: RangeOutputsGrpcRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct SearchOutputsPager<'a> {
    client: &'a Primitives,
    request: SearchOutputsRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct SearchOutputsGrpcPager<'a> {
    client: &'a Primitives,
    request: SearchOutputsGrpcRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct TimeMachineOutputsPager<'a> {
    client: &'a Primitives,
    request: TimeMachineOutputsRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct TimeMachineOutputsGrpcPager<'a> {
    client: &'a Primitives,
    request: TimeMachineOutputsGrpcRequest,
    state: PaginationState,
    finished: bool,
}

impl<'a> RangeOutputsCall<'a> {
    pub(crate) fn new(client: &'a Primitives, request: RangeOutputsRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<RangeOutputsResponse, SdkError> {
        self.client.range(&self.request).await
    }

    pub fn pager(self) -> Result<RangeOutputsPager<'a>, SdkError> {
        Ok(RangeOutputsPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<RangeOutputsTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(RangeOutputsTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> RangeOutputsGrpcCall<'a> {
    pub(crate) fn new(client: &'a Primitives, request: RangeOutputsGrpcRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<RangeOutputsResponse, SdkError> {
        self.client.range_grpc(&self.request).await
    }

    pub fn pager(self) -> Result<RangeOutputsGrpcPager<'a>, SdkError> {
        Ok(RangeOutputsGrpcPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<RangeOutputsTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(RangeOutputsTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> SearchOutputsCall<'a> {
    pub(crate) fn new(client: &'a Primitives, request: SearchOutputsRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<SearchOutputsResponse, SdkError> {
        self.client.search(&self.request).await
    }

    pub fn pager(self) -> Result<SearchOutputsPager<'a>, SdkError> {
        require_explicit_close_end(self.request.close_end.as_ref(), "search")?;
        Ok(SearchOutputsPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<SearchOutputsTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(SearchOutputsTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> SearchOutputsGrpcCall<'a> {
    pub(crate) fn new(client: &'a Primitives, request: SearchOutputsGrpcRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<SearchOutputsResponse, SdkError> {
        self.client.search_grpc(&self.request).await
    }

    pub fn pager(self) -> Result<SearchOutputsGrpcPager<'a>, SdkError> {
        require_explicit_close_end(self.request.close_end.as_ref(), "search")?;
        Ok(SearchOutputsGrpcPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<SearchOutputsTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(SearchOutputsTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> TimeMachineOutputsCall<'a> {
    pub(crate) fn new(client: &'a Primitives, request: TimeMachineOutputsRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<TimeMachineOutputsResponse, SdkError> {
        self.client.time_machine(&self.request).await
    }

    pub fn pager(self) -> Result<TimeMachineOutputsPager<'a>, SdkError> {
        require_explicit_close_end(self.request.close_end.as_ref(), "time-machine")?;
        Ok(TimeMachineOutputsPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<TimeMachineOutputsTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(TimeMachineOutputsTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> TimeMachineOutputsGrpcCall<'a> {
    pub(crate) fn new(client: &'a Primitives, request: TimeMachineOutputsGrpcRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<TimeMachineOutputsResponse, SdkError> {
        self.client.time_machine_grpc(&self.request).await
    }

    pub fn pager(self) -> Result<TimeMachineOutputsGrpcPager<'a>, SdkError> {
        require_explicit_close_end(self.request.close_end.as_ref(), "time-machine")?;
        Ok(TimeMachineOutputsGrpcPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<TimeMachineOutputsTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(TimeMachineOutputsTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> RangeOutputsPager<'a> {
    fn new(client: &'a Primitives, request: RangeOutputsRequest) -> Self {
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

    pub async fn next(&mut self) -> Result<Option<RangeOutputsResponse>, SdkError> {
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

impl<'a> RangeOutputsGrpcPager<'a> {
    fn new(client: &'a Primitives, request: RangeOutputsGrpcRequest) -> Self {
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

    pub async fn next(&mut self) -> Result<Option<RangeOutputsResponse>, SdkError> {
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

impl<'a> SearchOutputsPager<'a> {
    fn new(client: &'a Primitives, request: SearchOutputsRequest) -> Self {
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

    pub async fn next(&mut self) -> Result<Option<SearchOutputsResponse>, SdkError> {
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

impl<'a> SearchOutputsGrpcPager<'a> {
    fn new(client: &'a Primitives, request: SearchOutputsGrpcRequest) -> Self {
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

    pub async fn next(&mut self) -> Result<Option<SearchOutputsResponse>, SdkError> {
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

impl<'a> TimeMachineOutputsPager<'a> {
    fn new(client: &'a Primitives, request: TimeMachineOutputsRequest) -> Self {
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

    pub async fn next(&mut self) -> Result<Option<TimeMachineOutputsResponse>, SdkError> {
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

impl<'a> TimeMachineOutputsGrpcPager<'a> {
    fn new(client: &'a Primitives, request: TimeMachineOutputsGrpcRequest) -> Self {
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

    pub async fn next(&mut self) -> Result<Option<TimeMachineOutputsResponse>, SdkError> {
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
