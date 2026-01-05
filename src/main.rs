//! Tsundoku CLI - Japanese web novel downloader and translator.

use anyhow::{Context, Result};
use clap::Parser;
use tsundoku::config::Config;
use tsundoku::console::Console;
use tsundoku::scrapers::ScraperRegistry;

/// Japanese web novel downloader and translator.
#[derive(Parser, Debug)]
#[command(name = "tsundoku")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL of the novel to download.
    novel_url: String,

    /// Start downloading from chapter N (1-based).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
    start: Option<u32>,

    /// Stop downloading at chapter N (1-based, inclusive).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
    end: Option<u32>,

    /// Skip manual name mapping review pause.
    #[arg(long)]
    no_name_pause: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let console = Console::new();

    console.section("Tsundoku - Web Novel Downloader");

    // Load configuration
    console.step("Loading configuration...");
    let config = Config::load().context("Failed to load configuration")?;

    // Check if this is first run (API key not configured)
    if !config.api.is_configured() {
        let config_path = Config::config_path()?;
        console.warning(&format!(
            "API key not configured. Please edit: {}",
            config_path.display()
        ));
        console.info("Set your OpenAI-compatible API key in the config file and run again.");
        return Ok(());
    }

    config.validate().context("Invalid configuration")?;
    console.success("Configuration loaded");

    // Find appropriate scraper
    console.step("Finding scraper for URL...");
    let registry = ScraperRegistry::new(&config.scraping);
    let scraper = registry
        .find_for_url(&args.novel_url)
        .ok_or_else(|| anyhow::anyhow!("No scraper found for URL: {}", args.novel_url))?;

    console.success(&format!("Using {} scraper", scraper.name()));

    // Fetch novel info
    console.step("Fetching novel information...");
    let novel_info = scraper
        .get_novel_info(&args.novel_url)
        .await
        .context("Failed to fetch novel info")?;

    console.success(&format!("Found: {}", novel_info.title));
    console.info(&format!("Novel ID: {}", novel_info.novel_id));

    // Fetch chapter list
    console.step("Fetching chapter list...");
    let chapter_list = scraper
        .get_chapter_list(&novel_info.base_url)
        .await
        .context("Failed to fetch chapter list")?;

    match &chapter_list {
        tsundoku::ChapterList::OneShot => {
            console.success("This is a one-shot story");
        }
        tsundoku::ChapterList::Chapters(chapters) => {
            console.success(&format!("Found {} chapters", chapters.len()));
        }
    }

    // Validate chapter range
    let (_start_chapter, _end_chapter) = validate_chapter_range(
        args.start,
        args.end,
        &chapter_list,
        &console,
    )?;

    // For now, let's just demonstrate downloading the first chapter
    if let tsundoku::ChapterList::Chapters(chapters) = &chapter_list {
        if let Some(chapter) = chapters.first() {
            console.step(&format!("Downloading chapter 1: {}", chapter.title));
            
            match scraper.download_chapter(&chapter.url).await {
                Ok(content) => {
                    console.success(&format!(
                        "Downloaded {} characters",
                        content.chars().count()
                    ));
                    
                    // Print first 200 chars as preview
                    let preview: String = content.chars().take(200).collect();
                    console.info(&format!("Preview: {}...", preview));
                }
                Err(e) => {
                    console.error(&format!("Failed to download chapter: {}", e));
                }
            }
        }
    } else {
        // One-shot - download from base URL
        console.step("Downloading one-shot content...");
        match scraper.download_chapter(&novel_info.base_url).await {
            Ok(content) => {
                console.success(&format!(
                    "Downloaded {} characters",
                    content.chars().count()
                ));
            }
            Err(e) => {
                console.error(&format!("Failed to download content: {}", e));
            }
        }
    }

    console.section("Done!");
    Ok(())
}

/// Validates the chapter range arguments.
fn validate_chapter_range(
    start: Option<u32>,
    end: Option<u32>,
    chapter_list: &tsundoku::ChapterList,
    console: &Console,
) -> Result<(u32, u32)> {
    let total_chapters = chapter_list.len() as u32;

    // One-shots cannot use range
    if chapter_list.is_oneshot() {
        if start.is_some() || end.is_some() {
            anyhow::bail!("Cannot use --start or --end with one-shot stories");
        }
        return Ok((1, 1));
    }

    let start_chapter = start.unwrap_or(1);
    let end_chapter = end.unwrap_or(total_chapters);

    // Validate range
    if start_chapter > end_chapter {
        anyhow::bail!(
            "Start chapter ({}) cannot be greater than end chapter ({})",
            start_chapter,
            end_chapter
        );
    }

    if end_chapter > total_chapters {
        anyhow::bail!(
            "End chapter ({}) exceeds total chapters ({})",
            end_chapter,
            total_chapters
        );
    }

    console.info(&format!(
        "Processing chapters {} to {} of {}",
        start_chapter, end_chapter, total_chapters
    ));

    Ok((start_chapter, end_chapter))
}
