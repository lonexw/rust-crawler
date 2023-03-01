#[allow(dead_code)]

use anyhow::Result;
use futures::StreamExt;
use rust_crawler::scraper::Selector;
use std::time::Duration;
use rust_crawler::{Collector, Crawler, CrawlerConfig, RequestDelay, Response, Scraper};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    struct BookScraper {
        cover_selector: Selector,
        title_selector: Selector,
        // info_selector: Selector,
        score_selector: Selector,
        people_selector: Selector,
        collector_selector: Selector,
        // isbn_selector: Selector,
    }

    impl Default for BookScraper {
        fn default() -> Self {
            Self {
                cover_selector: Selector::parse("a.cover-link").unwrap(),
                title_selector: Selector::parse("div#wrapper h1 span").unwrap(),
                // info_selector: Selector::parse("div.meta.abstract").unwrap(),
                score_selector: Selector::parse("strong.rating_num").unwrap(),
                people_selector: Selector::parse("a.rating_people span").unwrap(),
                // isbn_selector: Selector::parse("a.isbn").unwrap(),
                collector_selector: Selector::parse("div#collector p a").unwrap(),
                // isbn_selector: Selector::parse("h1._3pxf09j4e").unwrap(),
            }
        }
    }

    #[derive(Debug)]
    enum BookState {
        Page(usize),
        Book,
    }

    #[derive(Debug)]
    struct Book {
        title: String,
        // info: String,
        score: String,
        people: String,
        stats: Vec<String>,
        // isbn: String,
    }

    impl Scraper for BookScraper {
        type Output = Book;
        type State = BookState;

        fn scrape(
            &mut self,
            response: Response<Self::State>,
            crawler: &mut Crawler<Self>
        ) -> Result<Option<Self::Output>> {
            let html = response.html();

            if let Some(state) = response.state {
                match state {
                    BookState::Page(_page) => {
                        // Find all books in the page
                        for (_idx, el) in html.select(&self.cover_selector).enumerate() 
                        {
                            dbg!(el.value().attr("href"));
                            crawler.visit_with_state(
                                el.value().attr("href").unwrap(), 
                                BookState::Book,
                            )   
                        }
                    },
                    BookState::Book => {
                        let el_title = html.select(&self.title_selector).next().unwrap();
                        let el_score = html.select(&self.score_selector).next().unwrap();
                        let el_people = html.select(&self.people_selector).next().unwrap();

                        let mut stats = vec![];
                        for (_, link) in html.select(&self.collector_selector).enumerate() {
                            stats.push(link.inner_html());
                        }

                        // scrape books info
                        let entry = Book {
                            title: el_title.inner_html(), 
                            score: el_score.inner_html(),
                            people: el_people.inner_html(),
                            stats
                        };

                        return Ok(Some(entry));
                    }
                }
            }

            Ok(None)
        }
    }

    let config = CrawlerConfig::default()
        .allow_domain_with_delay("127.0.0.1", RequestDelay::Fixed(Duration::from_millis(1_000)))
        .allow_domain_with_delay("book.douban.com", RequestDelay::Fixed(Duration::from_millis(1_000)));
    let mut collector = Collector::new(BookScraper::default(), config);

    collector.crawler_mut().visit_with_state(
        // "https://search.douban.com/book/subject_search?search_text=9787513349369&cat=1001", 
        "http://127.0.0.1:8000/9787513349369.html",
        BookState::Page(1)
    );

    dbg!("开始查询书籍：9787513349369");

    while let Some(output) = collector.next().await {
        let book = output?;
        dbg!(book);
    }

    Ok(())
}