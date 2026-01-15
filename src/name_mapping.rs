//! Name mapping store for character name translations.
//!
//! Provides persistent storage of character name mappings with a vote-based
//! consensus system for determining the best English translation of Japanese names.

use crate::error::NameMappingError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Regex for detecting bad characters in original names.
/// Names shouldn't contain punctuation, whitespace, or separators.
static BAD_ORIGINAL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[\s・･｡､,，。／/：:;!！?？\-—–‑·（）()［\]{}＜＞<>『』「」〈〉【】]")
        .expect("Invalid BAD_ORIGINAL_REGEX")
});

/// Regex for detecting Japanese honorific suffixes.
static HONORIFIC_SUFFIX_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(さん|ちゃん|くん|君|様|さま|殿|氏|先生|先輩|嬢)$")
        .expect("Invalid HONORIFIC_SUFFIX_REGEX")
});

/// English honorifics to reject.
const ENGLISH_HONORIFICS: &[&str] = &[
    "-san", "-chan", "-kun", "-sama", " san", " chan", " kun", " sama",
];

/// Words that the name scout tends to incorrectly label as character names.
///
/// This is intentionally a simple denylist (exact match) so we don't accidentally
/// filter out legitimate names.
const ORIGINAL_NAME_DENYLIST: &[&str] = &[
    // Common Japanese pronouns / self-references
    "彼",
    "彼女",
    "あいつ",
    "こいつ",
    "そいつ",
    "こちとら",
    "こちら",
    "自分",
    "私",
    "わたし",
    "わたくし",
    "俺",
    "おれ",
    "僕",
    "ぼく",
    "うち",
    "あなた",
    "君",
    "きみ",
    "お前",
    "おまえ",
    "貴様",
    // Plurals and groups
    "彼ら",
    "彼女ら",
    "俺たち",
    "僕ら",
    "私たち",
    "あなたたち",
    "皆",
    "みんな",
];

/// Indicates what part of a name this is (family name, given name, or unknown).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NamePart {
    Family,
    Given,
    #[default]
    Unknown,
}

impl std::str::FromStr for NamePart {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "family" => Self::Family,
            "given" => Self::Given,
            _ => Self::Unknown,
        })
    }
}

/// A name entry for recording votes.
#[derive(Debug, Clone)]
pub struct NameEntry {
    /// Original Japanese name.
    pub original: String,
    /// English translation.
    pub english: String,
    /// Which part of the name this is.
    pub part: NamePart,
}

/// Information about a single name in the mapping store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameInfo {
    /// Which part of the name this is.
    pub part: NamePart,
    /// Vote counts for each English translation.
    pub votes: HashMap<String, u32>,
    /// The winning English translation (highest votes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub english: Option<String>,
    /// The vote count of the winning translation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
}

impl NameInfo {
    /// Create a new empty NameInfo.
    pub fn new(part: NamePart) -> Self {
        Self {
            part,
            votes: HashMap::new(),
            english: None,
            count: None,
        }
    }

    /// Recalculate the winning translation from votes.
    pub fn recalculate_best(&mut self) {
        if self.votes.is_empty() {
            self.english = None;
            self.count = None;
            return;
        }

        // Find the translation with the highest vote count.
        // On tie, prefer the current best for stability.
        let mut best_english: Option<&String> = None;
        let mut best_count: u32 = 0;

        for (english, &count) in &self.votes {
            if count > best_count || (count == best_count && self.english.as_ref() != Some(english))
            {
                // Take this one if it has more votes, or same votes but we have no current best
                if count > best_count {
                    best_english = Some(english);
                    best_count = count;
                }
            }
        }

        // If we found a best, or the current best is still valid
        if let Some(english) = best_english {
            self.english = Some(english.clone());
            self.count = Some(best_count);
        } else if let Some(ref current) = self.english {
            // Keep current if it's still in votes
            if let Some(&count) = self.votes.get(current) {
                self.count = Some(count);
            } else {
                // Current best no longer exists, pick any
                if let Some((english, &count)) = self.votes.iter().next() {
                    self.english = Some(english.clone());
                    self.count = Some(count);
                }
            }
        }
    }
}

/// The full name mapping data structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NameMappingData {
    /// Map from original Japanese names to their info.
    pub names: HashMap<String, NameInfo>,
    /// List of chapter numbers that have been scouted.
    pub coverage: Vec<u32>,
}

/// Name mapping store for a specific novel.
pub struct NameMappingStore {
    /// Path to the JSON file.
    filepath: PathBuf,
    /// The mapping data.
    data: NameMappingData,
}

impl NameMappingStore {
    /// Create a new NameMappingStore for the given module and novel ID.
    ///
    /// # Arguments
    /// * `names_dir` - Directory where name mapping files are stored.
    /// * `module_name` - Name of the scraper module (e.g., "syosetu").
    /// * `novel_id` - Unique identifier for the novel.
    pub fn new(
        names_dir: &Path,
        module_name: &str,
        novel_id: &str,
    ) -> Result<Self, NameMappingError> {
        // Build filename: "{module_name}: {novel_id}.json"
        // On Windows, replace : with - since colons aren't allowed in filenames
        let filename = if cfg!(windows) {
            format!("{} - {}.json", module_name, novel_id)
        } else {
            format!("{}: {}.json", module_name, novel_id)
        };

        let filepath = names_dir.join(&filename);

        let mut store = Self {
            filepath,
            data: NameMappingData::default(),
        };

        // Load from disk if file exists
        if store.filepath.exists() {
            store.reload_from_disk()?;
        }

        // Purge bad votes on load
        store.purge_bad_votes();

        Ok(store)
    }

    /// Get the filepath for this store.
    pub fn filepath(&self) -> &Path {
        &self.filepath
    }

    /// Record votes from a list of name entries.
    pub fn record_votes(&mut self, entries: &[NameEntry]) {
        for entry in entries {
            // Validate entry
            if entry.original.is_empty() || entry.english.is_empty() {
                continue;
            }

            // Skip if original contains bad characters
            if BAD_ORIGINAL_REGEX.is_match(&entry.original) {
                continue;
            }

            // Skip if original is in denylist (e.g. pronouns)
            if ORIGINAL_NAME_DENYLIST.contains(&entry.original.as_str()) {
                continue;
            }

            // Skip if english contains whitespace
            if entry.english.chars().any(|c| c.is_whitespace()) {
                continue;
            }

            // Skip if original or english contains honorifics
            if HONORIFIC_SUFFIX_REGEX.is_match(&entry.original) {
                continue;
            }

            let english_lower = entry.english.to_lowercase();
            if ENGLISH_HONORIFICS.iter().any(|h| english_lower.contains(h)) {
                continue;
            }

            // Get or create entry
            let name_info = self
                .data
                .names
                .entry(entry.original.clone())
                .or_insert_with(|| NameInfo::new(entry.part.clone()));

            // Update part if we have a known part and current is unknown
            if name_info.part == NamePart::Unknown && entry.part != NamePart::Unknown {
                name_info.part = entry.part.clone();
            }

            // Increment vote count
            *name_info.votes.entry(entry.english.clone()).or_insert(0) += 1;

            // Recalculate best
            name_info.recalculate_best();
        }
    }

    /// Purge bad votes from the mapping.
    pub fn purge_bad_votes(&mut self) {
        // Remove entries with bad original names
        self.data.names.retain(|original, info| {
            // Check original for bad characters
            if BAD_ORIGINAL_REGEX.is_match(original) {
                return false;
            }

            // Check original for honorific suffix
            if HONORIFIC_SUFFIX_REGEX.is_match(original) {
                return false;
            }

            // Reject if original is in denylist (e.g. pronouns)
            if ORIGINAL_NAME_DENYLIST.contains(&original.as_str()) {
                return false;
            }

            // Filter out bad votes
            info.votes.retain(|english, _| {
                // Reject if english contains whitespace
                if english.chars().any(|c| c.is_whitespace()) {
                    return false;
                }

                // Reject if english contains honorifics
                let english_lower = english.to_lowercase();
                if ENGLISH_HONORIFICS.iter().any(|h| english_lower.contains(h)) {
                    return false;
                }

                true
            });

            // Recalculate best after filtering
            info.recalculate_best();

            // Keep entry if it still has votes
            !info.votes.is_empty()
        });
    }

    /// Check if a chapter has been scouted.
    pub fn is_chapter_covered(&self, chapter_number: u32) -> bool {
        self.data.coverage.contains(&chapter_number)
    }

    /// Add chapters to the coverage list.
    pub fn add_coverage(&mut self, chapters: &[u32]) {
        let coverage_set: HashSet<u32> = self.data.coverage.iter().copied().collect();
        for &chapter in chapters {
            if !coverage_set.contains(&chapter) {
                self.data.coverage.push(chapter);
            }
        }
        // Sort for consistency
        self.data.coverage.sort_unstable();
    }

    /// Apply name mappings to text, replacing Japanese names with English.
    /// Replaces longest matches first to handle overlapping names.
    pub fn apply_to_text(&self, text: &str) -> String {
        // Build a list of (original, english) pairs, sorted by length descending
        let mut replacements: Vec<(&str, &str)> = self
            .data
            .names
            .iter()
            .filter_map(|(original, info)| {
                info.english
                    .as_ref()
                    .map(|english| (original.as_str(), english.as_str()))
            })
            .collect();

        // Sort by length descending (longest first)
        replacements.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        // Apply replacements
        let mut result = text.to_string();
        for (original, english) in replacements {
            result = result.replace(original, english);
        }

        result
    }

    /// Save the mapping to disk.
    pub fn save(&self) -> Result<(), NameMappingError> {
        // Ensure parent directory exists
        if let Some(parent) = self.filepath.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.data)?;
        std::fs::write(&self.filepath, content)
            .map_err(|e| NameMappingError::WriteError(e.to_string()))?;

        Ok(())
    }

    /// Reload the mapping from disk.
    pub fn reload_from_disk(&mut self) -> Result<(), NameMappingError> {
        let content = std::fs::read_to_string(&self.filepath)?;
        let data: NameMappingData = serde_json::from_str(&content)?;

        // Validate structure (serde already does this, but we ensure required fields)
        self.data = data;

        // Purge bad votes after reload
        self.purge_bad_votes();

        Ok(())
    }

    /// Get the number of names in the mapping.
    pub fn len(&self) -> usize {
        self.data.names.len()
    }

    /// Check if the mapping is empty.
    pub fn is_empty(&self) -> bool {
        self.data.names.is_empty()
    }

    /// Get the coverage list.
    pub fn coverage(&self) -> &[u32] {
        &self.data.coverage
    }

    /// Get an iterator over all name entries.
    pub fn names(&self) -> impl Iterator<Item = (&str, &NameInfo)> {
        self.data.names.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Get the raw data (for testing/debugging).
    pub fn data(&self) -> &NameMappingData {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_name_part_from_str() {
        assert_eq!("family".parse::<NamePart>().unwrap(), NamePart::Family);
        assert_eq!("FAMILY".parse::<NamePart>().unwrap(), NamePart::Family);
        assert_eq!("given".parse::<NamePart>().unwrap(), NamePart::Given);
        assert_eq!("unknown".parse::<NamePart>().unwrap(), NamePart::Unknown);
        assert_eq!("invalid".parse::<NamePart>().unwrap(), NamePart::Unknown);
    }

    #[test]
    fn test_record_votes() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = NameMappingStore::new(temp_dir.path(), "syosetu", "n1234ab").unwrap();

        // Record some votes
        store.record_votes(&[
            NameEntry {
                original: "田中".to_string(),
                english: "Tanaka".to_string(),
                part: NamePart::Family,
            },
            NameEntry {
                original: "田中".to_string(),
                english: "Tanaka".to_string(),
                part: NamePart::Family,
            },
            NameEntry {
                original: "太郎".to_string(),
                english: "Taro".to_string(),
                part: NamePart::Given,
            },
        ]);

        assert_eq!(store.len(), 2);

        let tanaka = store.data.names.get("田中").unwrap();
        assert_eq!(tanaka.english, Some("Tanaka".to_string()));
        assert_eq!(tanaka.count, Some(2));
        assert_eq!(tanaka.part, NamePart::Family);
    }

    #[test]
    fn test_bad_original_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = NameMappingStore::new(temp_dir.path(), "syosetu", "n1234ab").unwrap();

        // Names with spaces or punctuation should be rejected
        store.record_votes(&[
            NameEntry {
                original: "田中 太郎".to_string(), // Contains space
                english: "TanakaTaro".to_string(),
                part: NamePart::Unknown,
            },
            NameEntry {
                original: "田中・太郎".to_string(), // Contains ・
                english: "TanakaTaro".to_string(),
                part: NamePart::Unknown,
            },
        ]);

        assert!(store.is_empty());
    }

    #[test]
    fn test_honorific_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = NameMappingStore::new(temp_dir.path(), "syosetu", "n1234ab").unwrap();

        // Names with honorifics should be rejected
        store.record_votes(&[
            NameEntry {
                original: "田中さん".to_string(), // Contains -san
                english: "Tanaka".to_string(),
                part: NamePart::Family,
            },
            NameEntry {
                original: "田中".to_string(),
                english: "Tanaka-san".to_string(), // English has honorific
                part: NamePart::Family,
            },
        ]);

        assert!(store.is_empty());
    }

    #[test]
    fn test_whitespace_in_english_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = NameMappingStore::new(temp_dir.path(), "syosetu", "n1234ab").unwrap();

        store.record_votes(&[NameEntry {
            original: "田中".to_string(),
            english: "Tanaka San".to_string(), // Contains space
            part: NamePart::Family,
        }]);

        assert!(store.is_empty());
    }

    #[test]
    fn test_original_denylist_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = NameMappingStore::new(temp_dir.path(), "syosetu", "n1234ab").unwrap();

        store.record_votes(&[
            NameEntry {
                original: "彼女".to_string(),
                english: "Kanojo".to_string(),
                part: NamePart::Unknown,
            },
            NameEntry {
                original: "俺".to_string(),
                english: "Ore".to_string(),
                part: NamePart::Unknown,
            },
        ]);

        assert!(store.is_empty());
    }

    #[test]
    fn test_apply_to_text() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = NameMappingStore::new(temp_dir.path(), "syosetu", "n1234ab").unwrap();

        store.record_votes(&[
            NameEntry {
                original: "田中".to_string(),
                english: "Tanaka".to_string(),
                part: NamePart::Family,
            },
            NameEntry {
                original: "太郎".to_string(),
                english: "Taro".to_string(),
                part: NamePart::Given,
            },
        ]);

        let text = "田中太郎は学校に行った。";
        let result = store.apply_to_text(text);
        assert_eq!(result, "TanakaTaroは学校に行った。");
    }

    #[test]
    fn test_longest_match_first() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = NameMappingStore::new(temp_dir.path(), "syosetu", "n1234ab").unwrap();

        store.record_votes(&[
            NameEntry {
                original: "田".to_string(),
                english: "Ta".to_string(),
                part: NamePart::Unknown,
            },
            NameEntry {
                original: "田中".to_string(),
                english: "Tanaka".to_string(),
                part: NamePart::Family,
            },
        ]);

        let text = "田中さんと田さん";
        let result = store.apply_to_text(text);
        // "田中" should be replaced with "Tanaka" first, then "田" with "Ta"
        assert!(result.contains("Tanaka"));
        assert!(result.contains("Ta"));
        // Verify 田中 was replaced as a whole, not as 田 + 中
        assert_eq!(result, "TanakaさんとTaさん");
    }

    #[test]
    fn test_coverage_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = NameMappingStore::new(temp_dir.path(), "syosetu", "n1234ab").unwrap();

        assert!(!store.is_chapter_covered(1));

        store.add_coverage(&[1, 3, 5]);
        assert!(store.is_chapter_covered(1));
        assert!(store.is_chapter_covered(3));
        assert!(!store.is_chapter_covered(2));

        // Adding duplicate should not create duplicates
        store.add_coverage(&[1, 2]);
        assert_eq!(store.coverage(), &[1, 2, 3, 5]);
    }

    #[test]
    fn test_save_and_reload() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = NameMappingStore::new(temp_dir.path(), "syosetu", "n1234ab").unwrap();

        store.record_votes(&[NameEntry {
            original: "田中".to_string(),
            english: "Tanaka".to_string(),
            part: NamePart::Family,
        }]);
        store.add_coverage(&[1, 2, 3]);
        store.save().unwrap();

        // Create a new store from the same file
        let store2 = NameMappingStore::new(temp_dir.path(), "syosetu", "n1234ab").unwrap();
        assert_eq!(store2.len(), 1);
        assert!(store2.is_chapter_covered(2));
    }

    #[test]
    fn test_vote_consensus() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = NameMappingStore::new(temp_dir.path(), "syosetu", "n1234ab").unwrap();

        // Vote for different translations
        store.record_votes(&[
            NameEntry {
                original: "優子".to_string(),
                english: "Yuko".to_string(),
                part: NamePart::Given,
            },
            NameEntry {
                original: "優子".to_string(),
                english: "Yuuko".to_string(),
                part: NamePart::Given,
            },
            NameEntry {
                original: "優子".to_string(),
                english: "Yuko".to_string(),
                part: NamePart::Given,
            },
        ]);

        let info = store.data.names.get("優子").unwrap();
        assert_eq!(info.english, Some("Yuko".to_string())); // Yuko has 2 votes
        assert_eq!(info.count, Some(2));
    }
}
