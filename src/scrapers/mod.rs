//! Scraper trait and common types for web novel scrapers.
//!
//! This module defines the interface that all scrapers must implement,
//! along with common data types for novels and chapters.

mod kakuyomu;
mod pixiv;
mod syosetu;

pub use kakuyomu::KakuyomuScraper;
pub use pixiv::PixivScraper;
pub use syosetu::SyosetuScraper;

use crate::config::ScrapingConfig;
use crate::error::ScraperError;
use async_trait::async_trait;
use std::time::Duration;

/// Information about a novel.
#[derive(Debug, Clone)]
pub struct NovelInfo {
    /// The novel's title in Japanese.
    pub title: String,

    /// Base URL for the novel (used to fetch chapter list).
    pub base_url: String,

    /// Unique identifier for the novel on the platform.
    pub novel_id: String,
}

/// Information about a single chapter.
#[derive(Debug, Clone)]
pub struct ChapterInfo {
    /// Chapter title.
    pub title: String,

    /// URL to download the chapter content.
    pub url: String,

    /// Chapter number (1-based).
    pub number: u32,
}

/// Represents the chapter list for a novel.
#[derive(Debug, Clone)]
pub enum ChapterList {
    /// Multi-chapter novel with a list of chapters.
    Chapters(Vec<ChapterInfo>),

    /// Single-chapter (one-shot) story.
    OneShot,
}

impl ChapterList {
    /// Returns true if this is a one-shot story.
    pub fn is_oneshot(&self) -> bool {
        matches!(self, ChapterList::OneShot)
    }

    /// Returns the number of chapters, or 1 for one-shots.
    pub fn len(&self) -> usize {
        match self {
            ChapterList::Chapters(chapters) => chapters.len(),
            ChapterList::OneShot => 1,
        }
    }

    /// Returns true if there are no chapters.
    pub fn is_empty(&self) -> bool {
        match self {
            ChapterList::Chapters(chapters) => chapters.is_empty(),
            ChapterList::OneShot => false,
        }
    }
}

/// Trait for web novel scrapers.
///
/// Each scraper implementation handles a specific platform (Syosetu, Kakuyomu, etc.)
/// and provides methods to fetch novel metadata, chapter lists, and content.
#[async_trait]
pub trait Scraper: Send + Sync {
    /// Returns the human-readable name of this scraper.
    fn name(&self) -> &'static str;

    /// Returns the identifier used in file paths (lowercase, no spaces).
    fn id(&self) -> &'static str;

    /// Checks if this scraper can handle the given URL.
    fn can_handle(&self, url: &str) -> bool;

    /// Fetches novel metadata from the given URL.
    async fn get_novel_info(&self, url: &str) -> Result<NovelInfo, ScraperError>;

    /// Fetches the list of chapters for a novel.
    async fn get_chapter_list(&self, base_url: &str) -> Result<ChapterList, ScraperError>;

    /// Downloads the content of a single chapter.
    async fn download_chapter(&self, chapter_url: &str) -> Result<String, ScraperError>;
}

/// Registry of available scrapers.
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn Scraper>>,
}

impl ScraperRegistry {
    /// Creates a new registry with all available scrapers.
    pub fn new(config: &ScrapingConfig) -> Self {
        let scrapers: Vec<Box<dyn Scraper>> = vec![
            Box::new(SyosetuScraper::new(config.clone())),
            Box::new(KakuyomuScraper::new(config.clone())),
            Box::new(PixivScraper::new(config.clone())),
        ];

        Self { scrapers }
    }

    /// Finds a scraper that can handle the given URL.
    pub fn find_for_url(&self, url: &str) -> Option<&dyn Scraper> {
        self.scrapers
            .iter()
            .find(|s| s.can_handle(url))
            .map(|s| s.as_ref())
    }

    /// Returns all registered scrapers.
    pub fn all(&self) -> &[Box<dyn Scraper>] {
        &self.scrapers
    }
}

/// Common HTTP client configuration for scrapers.
pub fn create_http_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .cookie_store(true)
        .timeout(Duration::from_secs(30))
        .build()
}

/// Applies rate limiting delay.
pub async fn rate_limit(delay_sec: f64) {
    if delay_sec > 0.0 {
        tokio::time::sleep(Duration::from_secs_f64(delay_sec)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chapter_list_len() {
        let oneshot = ChapterList::OneShot;
        assert_eq!(oneshot.len(), 1);
        assert!(oneshot.is_oneshot());

        let chapters = ChapterList::Chapters(vec![
            ChapterInfo {
                title: "Ch 1".to_string(),
                url: "http://example.com/1".to_string(),
                number: 1,
            },
            ChapterInfo {
                title: "Ch 2".to_string(),
                url: "http://example.com/2".to_string(),
                number: 2,
            },
        ]);
        assert_eq!(chapters.len(), 2);
        assert!(!chapters.is_oneshot());
    }
}
