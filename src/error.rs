use reqwest::{Url, Request, Response, Error};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CrawlError<T: fmt::Debug> {
    #[error("Received response with non 2xx status {:?} for {:?} carrying state: {:?}", .response, .request_url, .state)]
    NoSuccessResponse {
        request_url: Option<Url>,
        /// 接收到的請求返回結果
        response: Response,
        /// 請求可能存在的狀態 state
        state: Option<T>,
    },
    #[error("Failed to construct a request: {} while carrying state: {:?}", .error, .state)]
    FailedToBuildRequest {
        error: Error,
        state: Option<T>,
        depth: usize,
    },
    #[error("Failed to process invalid request while carrying state: {:?}", .state)]
    InvalidRequest {
        request: Request,
        state: Option<T>,
    },
    #[error("Reached max depth at {} while carrying state: {:?}", .depth ,.state)]
    ReachedMaxDepth {
        request: Request,
        state: Option<T>,
        depth: usize,
    },
    #[error("Failed to fetch robots.txt from host: {}", .host)]
    RobotsTxtError {
        host: String,
    },
    #[error("Rejected a request, because its url is disallowed due to {}, while carrying state: {:?}", .reason, .state)]
    DisallowedRequest {
        reason: DisallowReason,
        request: Request,
        state: Option<T>,
    },
}

impl<T: fmt::Debug> CrawlError<T> {
    /// 獲取錯誤發生時的請求狀態
    pub fn state(&self) -> Option<&T> {
        match self {
            CrawlError::NoSuccessResponse { state, .. } => state.as_ref(),
            CrawlError::FailedToBuildRequest { state, .. } => state.as_ref(),
            CrawlError::InvalidRequest { state, .. } => state.as_ref(),
            CrawlError::ReachedMaxDepth { state, .. } => state.as_ref(),
            CrawlError::RobotsTxtError { .. } => None,
            CrawlError::DisallowedRequest { state, .. } => state.as_ref(),
        }
    }

    /// 數據轉換
    pub fn into_state(self) -> Option<T> {
        match self {
            CrawlError::NoSuccessResponse { state, .. } => state,
            CrawlError::FailedToBuildRequest { state, .. } => state,
            CrawlError::InvalidRequest { state, .. } => state,
            CrawlError::ReachedMaxDepth { state, .. } => state,
            CrawlError::RobotsTxtError { .. } => None,
            CrawlError::DisallowedRequest { state, .. } => state,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DisallowReason {
    /// 由於 robots 規則被禁止訪問
    RobotsTxt,
    /// 由於用戶配置文件禁止
    UserConfig,
}

impl fmt::Display for DisallowReason {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DisallowReason::RobotsTxt => {
                write!(fmt, "URL blocked by robots.txt")
            }
            DisallowReason::UserConfig => {
                write!(fmt, "URL blocked by user config")
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Error)]
#[error("Received response with unexpected status {0}")]
pub struct UnexpectedStatusError(u16);

impl UnexpectedStatusError {
    pub fn new(status: u16) -> Self {
        UnexpectedStatusError(status)
    }

    pub fn status(&self) -> u16 {
        self.0
    }
}