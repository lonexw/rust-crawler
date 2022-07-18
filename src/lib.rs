use anyhow::Result;
use futures::stream::Stream;
use futures::FutureExt;
use reqwest::IntoUrl;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

mod domain;
pub mod error;

pub use domain::{AllowList, AllowListConfig, BlockList, DomainListing};

/// Reexport all the scraper types
pub trait Scraper {}

/// Crawler 负责执行需要爬取的网站地址列表, 
/// 并将成功的响应结果转发给 Scraper 处理
pub struct Crawler<T: Scraper> {
    client: reqwest::Client,
    /// 域名列表：黑名單與白名單
    lsit: DomainListing<T::State>,
    /// 追蹤已提交請求的深度
    current_depth: usize,
    /// The maximum depth reqwest are allowed to next.
    max_depth: usize,
    /// 是否遵循目標網站爬蟲規則
    respect_robots_txt: bool,
    /// 是否忽略未成功請求響應
    skip_non_successful_response: bool,
}

impl<T: Scraper> Crawler<T> {
    /// 从 CrawlerConfig 中创建 Crawler 实例
    pub fn new(config: CrawlerConfig) -> Self {
        let client = config.client.unwrap_or_default(); 

        let list = if config.allow_domains.is_empty() {
            let block_list = BlockList::new(
                config.disallowed_domain,
                client.clone(),
                config.respect_robots_txt,
                config.skip_non_successful_response,
                config.max_depth.unwrap_or(usize::MAX),
                config
                    .max_requests
                    .unwrap_or(CrawlerConfig::MAX_CONCURRENT_REQUESTS),
            );

            DomainListing::BlockList(block_list)
        } else {
            let mut allow_list = AllowList::Default();
            let max_requests = config
                .max_requests
                .unwrap_or(CrawlerConfig::MAX_CONCURRENT_REQUESTS)
                / config.allowed_domains.len();
            for (domain, delay) in config.allowed_domains {
                let allow = AllowListConfig {
                    delay,
                    respect_robots_txt: config.respect_robots_txt,
                    client: client.clone(),
                    skip_non_successful_response: config.skip_non_successful_response,
                    max_depth: config.max_depth.unwrap_or(usize::MAX),
                    max_requests,
                };
                allow_list.allow(domain, allow);
            }
            DomainListing::AllowList(allow_list)
        };

        Self {
            client,
            list,
            current_depth: 0,
            max_depth: config.max_depth.unwrap_or(usize::MAX),
            respect_robots_txt: config.respect_robots_txt, 
            skip_non_successful_response: config.skip_non_successful_response,
        }
    }

    // 获取允许的最大请求深度
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// 是否遵循目标网站的爬虫规则
    pub fn respect_robots_txt(&self) -> bool {
        self.respect_robots_txt
    }

    /// 是否跳过访问不成功的请求响应
    pub fn skip_non_successful_response(&self) -> bool {
        self.skip_non_successful_response
    }
}

struct RequestDelay {}

pub struct CrawlerConfig {
    /// 访问的请求递归深度限制
    max_depth: Option<usize>,
    /// 最大请求数, 默认为 MAX_CONCURRENT_REQUESTS
    max_requests: Option<usize>,
    /// 跳过失败请求
    skip_non_successful_response: bool,
    /// 白名单，如果为空，不作限制
    allowed_domains: HashMap<String, Option<RequestDelay>>,
    /// 黑名单 🚫
    disallowed_domains: HashSet<String>,
    /// 是否遵从目标网站的数据获取规则 
    /// robots.txt: <http://www.robotstxt.org>
    respect_robots_txt: bool,
    /// 执行 http 请求的客户端
    client: Option<reqwest::Client>,
}
    
impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            max_depth: None,
            max_request: None,
            skip_non_successful_response: true,
            allowed_domains: Default::default(),
            disallowed_domains: Default::default(),
            respect_robots_txt: false,
            client: None,
        }    
    }
}
            
impl CrawlerConfig {
    const MAX_CONCURRENT_REQUESTS: usize = 1_00;

    pub fn max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    pub fn respect_robots_txt(mut self) -> Self {
        self.respect_robots_txt = true;
        self
    }

    pub fn scrape_non_sucess_response(mut self) -> Self {
        self.skip_non_successful_response = false;
        self
    }

    pub fn max_current_requests(mut self, max_requests: usize) -> Self {
        self.max_requests = Some(max_requests);
        self
    }

    pub fn set_client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn disallowed_domain(mut self, domain: impl Into<String>) -> Self {
        self.disallowed_domains.insert(domain);
        self
    }

    pub fn disallowed_domains<D, T>(mut self, domains: D) -> Self 
        where
            D: IntoIterator<Item = T>,
            T: Into<String>,
    {
        for domain in domains.into_iter() {
            self.disallowed_domains.insert(domain.into());
        }
        self
    }

    pub fn allow_domain(mut self, domain: impl Into<String>) -> Self {
        self.allowed_domains.insert(domain.into(), None);
        self
    }

    pub fn allow_domain_with_delay(mut self, 
        domain: impl Into<String>, 
        delay: RequestDelay,
    ) -> Self {
        self.allowed_domains.insert(domain, delay);
        self
    }

    pub fn allow_domains<D, T>(mut self, domains: D) -> Self 
        where
            D: IntoIterator<Item = T>,
            T: Into<String>,
    {
        for domain in domains.into_iter() {
            self.allowed_domains.insert(domain.into(), None)
        }

        self
    }

    pub fn allow_domains_with_delay<D, T>(mut self, domains: D) -> Self
        where
            D: IntoIterator<Item = (T, RequestDelay)>,
            T: Into<String>,
    {
        for (domain, deplay) in domains.into_iter() {
            self.allowed_domains.insert(domain.into(), deplay);
        }
        self
    }
}
