use crate::core::error::SdkError;
use crate::core::pagination::{PaginationAdvance, PaginationState, require_explicit_close_end};
use crate::core::time::TimeInput;
use crate::systems::aggregator::client::Aggregator;
use crate::systems::aggregator::types::{
    RangeBarsGrpcRequest, RangeBarsRequest, RangeBarsResponse, RangeBarsTraverseResult,
    SearchBarsGrpcRequest, SearchBarsRequest, SearchBarsResponse, SearchBarsTraverseResult,
    TimeMachineBarsGrpcRequest, TimeMachineBarsRequest, TimeMachineBarsResponse,
    TimeMachineBarsTraverseResult,
};

#[derive(Debug)]
pub struct RangeBarsCall<'a> {
    client: &'a Aggregator,
    request: RangeBarsRequest,
}

#[derive(Debug)]
pub struct RangeBarsGrpcCall<'a> {
    client: &'a Aggregator,
    request: RangeBarsGrpcRequest,
}

#[derive(Debug)]
pub struct SearchBarsCall<'a> {
    client: &'a Aggregator,
    request: SearchBarsRequest,
}

#[derive(Debug)]
pub struct SearchBarsGrpcCall<'a> {
    client: &'a Aggregator,
    request: SearchBarsGrpcRequest,
}

#[derive(Debug)]
pub struct TimeMachineBarsCall<'a> {
    client: &'a Aggregator,
    request: TimeMachineBarsRequest,
}

#[derive(Debug)]
pub struct TimeMachineBarsGrpcCall<'a> {
    client: &'a Aggregator,
    request: TimeMachineBarsGrpcRequest,
}

#[derive(Debug)]
pub struct RangeBarsPager<'a> {
    client: &'a Aggregator,
    request: RangeBarsRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct RangeBarsGrpcPager<'a> {
    client: &'a Aggregator,
    request: RangeBarsGrpcRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct SearchBarsPager<'a> {
    client: &'a Aggregator,
    request: SearchBarsRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct SearchBarsGrpcPager<'a> {
    client: &'a Aggregator,
    request: SearchBarsGrpcRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct TimeMachineBarsPager<'a> {
    client: &'a Aggregator,
    request: TimeMachineBarsRequest,
    state: PaginationState,
    finished: bool,
}

#[derive(Debug)]
pub struct TimeMachineBarsGrpcPager<'a> {
    client: &'a Aggregator,
    request: TimeMachineBarsGrpcRequest,
    state: PaginationState,
    finished: bool,
}

impl<'a> RangeBarsCall<'a> {
    pub(crate) fn new(client: &'a Aggregator, request: RangeBarsRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<RangeBarsResponse, SdkError> {
        self.client.range(&self.request).await
    }

    pub fn pager(self) -> Result<RangeBarsPager<'a>, SdkError> {
        Ok(RangeBarsPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<RangeBarsTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(RangeBarsTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> RangeBarsGrpcCall<'a> {
    pub(crate) fn new(client: &'a Aggregator, request: RangeBarsGrpcRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<RangeBarsResponse, SdkError> {
        self.client.range_grpc(&self.request).await
    }

    pub fn pager(self) -> Result<RangeBarsGrpcPager<'a>, SdkError> {
        Ok(RangeBarsGrpcPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<RangeBarsTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(RangeBarsTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> SearchBarsCall<'a> {
    pub(crate) fn new(client: &'a Aggregator, request: SearchBarsRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<SearchBarsResponse, SdkError> {
        self.client.search(&self.request).await
    }

    pub fn pager(self) -> Result<SearchBarsPager<'a>, SdkError> {
        require_explicit_close_end(self.request.close_end.as_ref(), "search")?;
        Ok(SearchBarsPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<SearchBarsTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(SearchBarsTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> SearchBarsGrpcCall<'a> {
    pub(crate) fn new(client: &'a Aggregator, request: SearchBarsGrpcRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<SearchBarsResponse, SdkError> {
        self.client.search_grpc(&self.request).await
    }

    pub fn pager(self) -> Result<SearchBarsGrpcPager<'a>, SdkError> {
        require_explicit_close_end(self.request.close_end.as_ref(), "search")?;
        Ok(SearchBarsGrpcPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<SearchBarsTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(SearchBarsTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> TimeMachineBarsCall<'a> {
    pub(crate) fn new(client: &'a Aggregator, request: TimeMachineBarsRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<TimeMachineBarsResponse, SdkError> {
        self.client.time_machine(&self.request).await
    }

    pub fn pager(self) -> Result<TimeMachineBarsPager<'a>, SdkError> {
        require_explicit_close_end(self.request.close_end.as_ref(), "time-machine")?;
        Ok(TimeMachineBarsPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<TimeMachineBarsTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(TimeMachineBarsTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> TimeMachineBarsGrpcCall<'a> {
    pub(crate) fn new(client: &'a Aggregator, request: TimeMachineBarsGrpcRequest) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<TimeMachineBarsResponse, SdkError> {
        self.client.time_machine_grpc(&self.request).await
    }

    pub fn pager(self) -> Result<TimeMachineBarsGrpcPager<'a>, SdkError> {
        require_explicit_close_end(self.request.close_end.as_ref(), "time-machine")?;
        Ok(TimeMachineBarsGrpcPager::new(self.client, self.request))
    }

    pub async fn traverse(self) -> Result<TimeMachineBarsTraverseResult, SdkError> {
        let mut pager = self.pager()?;
        let mut pages = Vec::new();

        while let Some(page) = pager.next().await? {
            pages.push(page);
        }

        Ok(TimeMachineBarsTraverseResult {
            pages,
            pages_fetched: pager.pages_fetched(),
        })
    }
}

impl<'a> RangeBarsPager<'a> {
    fn new(client: &'a Aggregator, request: RangeBarsRequest) -> Self {
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

    pub async fn next(&mut self) -> Result<Option<RangeBarsResponse>, SdkError> {
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

impl<'a> RangeBarsGrpcPager<'a> {
    fn new(client: &'a Aggregator, request: RangeBarsGrpcRequest) -> Self {
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

    pub async fn next(&mut self) -> Result<Option<RangeBarsResponse>, SdkError> {
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

impl<'a> SearchBarsPager<'a> {
    fn new(client: &'a Aggregator, request: SearchBarsRequest) -> Self {
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

    pub async fn next(&mut self) -> Result<Option<SearchBarsResponse>, SdkError> {
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

impl<'a> SearchBarsGrpcPager<'a> {
    fn new(client: &'a Aggregator, request: SearchBarsGrpcRequest) -> Self {
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

    pub async fn next(&mut self) -> Result<Option<SearchBarsResponse>, SdkError> {
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

impl<'a> TimeMachineBarsPager<'a> {
    fn new(client: &'a Aggregator, request: TimeMachineBarsRequest) -> Self {
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

    pub async fn next(&mut self) -> Result<Option<TimeMachineBarsResponse>, SdkError> {
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

impl<'a> TimeMachineBarsGrpcPager<'a> {
    fn new(client: &'a Aggregator, request: TimeMachineBarsGrpcRequest) -> Self {
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

    pub async fn next(&mut self) -> Result<Option<TimeMachineBarsResponse>, SdkError> {
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
