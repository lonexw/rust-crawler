use futures::stream::Stream;
use futures::Future;
use futures_timer::Delay;
use reqwest::header::HeaderMap;
use reqwest::{StatusCode, Url};
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

/// 隊列狀態中的請求封裝
pub struct QueuedRequest<T> {
    pub request: reqwest::Request,
    pub state: Option<T>,
    pub depth: usize,
}

pub struct QueuedRequestBuilder<T> {
    pub request: reqwest::Request,
    pub state: Option<T>,
    pub depth: usize,
}

/// 請求隊列
pub struct RequestQueue<T> {
    delay: Option<(Delay, RequestDelay)>,
    queued_requests: VecDeque<QueuedRequest<T>>,
}

impl<T> RequestQueue<T> {
    pub fn with_delay(delay: RequestDelay) -> Self {
        Self {
            delay: Some(Delay::new(Duration::default()), delay),
            queued_requests: Default::default(),
        }
    }

    pub fn queue_mut(&mut self) -> &mut VecDeque<QueuedRequest<T>> {
        &mut self.queued_requests
    }

    /// 移除請求延時時間
    pub fn remove_delay(&mut self) -> Option<RequestDelay> {
        self.delay.take().map(|(_, d)| d)
    }

    /// 設置請求間隔的延時時間
    pub fn set_delay(&mut self, mut delay: RequestDelay) -> Option<RequestDelay> {
        if let Some((_, d)) = self.delay.as_mut() {
            std::mem::swap(&mut delay, d);
            Some(delay)
        } else { 
            self.delay = Some((Delay::new(Duration::default()), delay))
            None 
        }
    }

    pub fn is_empty(&self) -> bool {
        self.queued_requests.is_empty()
    }

    pub fn len(&self) -> usize {
        self.queued_requests.len()
    }
}

impl<T: Unpin> Stream for RequestQueue<T> {
    type Item = QueuedRequest<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.queued_requests.is_empty() {
            Poll::Ready(None)
        }
        
        let pin = self.get_mut();
        let mut next = None;
        if let Some((mut delay, dur)) = pin.delay.take() {
            if Delay::poll(Pin::new(&mut delay), cx).is_ready() {
                next = pin.queued_requests.pop_front();
                delay.reset(dur.next_delay());
            }
            pin.delay = Some((delay, dur));
        } else {
            next = pin.queued_requests.pop_front();
        }

        Poll::Ready(next)
    }
}

/// Howto delay a request
#[derive(Debug, Clone, Copy)]
pub enum RequestDelay {
    /// 設置一個固定的時間端
    Fixed(Duration),
    /// 設置一個隨即的時間端，在 min 與 max 範圍之內
    Random {
        min: Duration,
        max: Duration,
    }
}

impl RequestDelay {
    pub fn fixed(delay: Duration) -> Self {
        RequestDelay::Fixed(delay)
    }

    pub fn random(max: Duration) -> Self {
        RequestDelay::Random {
            min: Duration::from_millis(0),
            max,
        }
    }

    pub fn random_in_range(min: Duration, max: Duration) -> Self {
        RequestDelay::Random { min, max }
    }

    pub fn next_delay(&self) -> Duration {
        use rand::Rng;

        match self {
            RequestDelay::Fixed(delay) => *delay,
            RequestDelay::Random { min, max } => Duration::from_millis(
                rand::thread_rng().gen_range(min.as_millis() as u64..=max.as_millis() as u64)
            ),
        }
    }
}

pub(crate) fn response_info(resp: &mut reqwest::Response) -> (StatusCode, Url, HeaderMap) {
    let mut headers = HeaderMap::new();
    std::mem::swap(&mut headers, resp.headers_mut());
    (resp.status(), resp.url().clone(), headers)
}

