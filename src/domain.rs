use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

use anyhow::Result;
use futures::stream::Stream;
use futures::{Future, FutureExt};

use crate::error::{CrawlError, DisallowReason};
use crate::requests::{
    response_info, QueuedRequest, QueuedRequestBuilder, RequestDelay, RequestQueue,
};
use crate::response::Response;
use crate::robots::{RobotsData, RobotsHandler};

pub enum DomainListing<T> {
    AllowList(AllowList<T>),
    BlockList(BlockList<T>),
}

impl<T> DomainListing<T> 
where
    T: Unpin + Send + Sync + 'static + fmt::Debug
{
    pub(crate) fn add_request(
        &mut self, 
        request: QueuedRequestBuilder<T>,
    ) -> Result<(), CrawlError> {
        let QueuedRequestBuilder {
            request,
            state,
            depth,
        } = request;

        match request.build() {
            Ok(request) => {
                let queued = QueuedRequest {
                    request,
                    state,
                    depth,
                };

                match self {
                    DomainListing::AllowList(list) => Ok(list.add_request(queued)?),
                    DomainListing::BlockList(list) => Ok(list.add_request(queued)?),
                }
            }
            Err(error) => Err(CrawlError::<T>::FailedToBuildRequest {
                error,
                state,
                depth,
            })
        }
    }
}

pub struct AllowList<T> {
    /// 所有允許的域名
    allowed: HashMap<String, AllowedDomain<T>>,
    domains: Vec<String>,
    /// 請求結果的集合
    queued_results: VecDeque<Result<Response<T>>>,
}

impl<T> Default for AllowList<T> {
    fn default() -> Self {
        Self {
            allowed: Default::default(),
            domains: Vec::new(),
            queued_results: Default::default(),
        }
    }
}

impl<T: fmt::Debug> AllowList<T> {
    pub fn allow(&mut self, domain: String, config: AllowListConfig) {
        self.allowed
            .insert(domain.clone(), AllowedDomain::new(config));
        self.domains.push(domain)
    }

    pub fn disallow(&mut self, domain: &str) -> Option<AllowedDomain<T>> {
        if let Some(list) = self.allowed.remove(domain) {
            let idx = self.iter().position(|d| d == domain).unwrap();
            self.domains.remove(idx);
            Some(list)
        } else {
            None
        }
    }

    /// The matching handler for the allowed domain if any
    pub fn get_domain(&self, domain: impl AsRef<str>) -> Option<&mut AllowedDomain<T>> {
        self.allowed.get(domain.as_ref())
    }

    /// Get mutable access to the matching handler for the allowed domain if any
    pub fn get_domain_mut(&mut self, domain: impl AsRef<str>) -> Option<&mut AllowedDomain<T>> {
        self.allowed.get_mut(domain.as_ref())
    }
}


