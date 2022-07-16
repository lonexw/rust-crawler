# rust-crawler

> 說明：這個項目構建階段，是我初步學習 Rust 構建項目的時期，基本很多地方都在模仿複製，僅作爲學習用途。

參考項目和學習資料來源：

- https://kerkour.com/rust-crawler-implementation
- https://github.com/mattsse/voyager
- https://kaisery.github.io/trpl-zh-cn/title-page.html
- https://course.rs/about-book.html
- https://rusty.rs/about.html
- https://github.com/rustlang-cn/async-book

## 爬虫程序设计

Why use Rust?
1. Async I/O Model, best performance possible when making requests.
2. Memory-related performance.
3. Safety when parsing.
4. Associated types.
5. 并发支持
6. 学习构建 rust project

业务架构示意图

- Crawler 爬虫：负责对访问的 url 列表进行管理，获取网页响应；
	- Control loop：queue urls


1）构建一个 CrawlerConfig 来对 Crawler 进行参数配置和初始化
	- 默认值和便捷方法
	- allow or disallow domain
	- concurrent_requests
2）需要构建一个 Crawler 结构体来实现主体功能

依赖库：
- 网络请求库：reqwest

