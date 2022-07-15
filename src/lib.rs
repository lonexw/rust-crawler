use std::collections::{HashMap, HashSet};

pub trait Scraper {}

/// Crawler 负责执行需要爬取的网站地址列表, 
/// 并将成功的响应结果转发给 Scraper 处理
pub struct Crawler<T: Scraper> {
    client: reqwest::Client,
}

impl<T: Scraper> Crawler<T> {
    /// 从 CrawlerConfig 中创建 Crawler 实例
    pub fn new(config: CrawlerConfig) -> Self {
        let client = config.client.unwrap_or_default(); 

        Self {
            client: client,
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