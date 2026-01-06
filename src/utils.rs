//! Utility functions for common operations.

use crate::error::TranslationError;

/// Splits text into chunks by lines, respecting a maximum chunk size.
///
/// This function splits text into chunks where each chunk is at most `chunk_size`
/// characters. It prefers to split on line boundaries to maintain context.
///
/// # Arguments
/// * `text` - The text to split
/// * `chunk_size` - Maximum size of each chunk in characters
///
/// # Returns
/// A vector of text chunks, each no larger than `chunk_size` (unless a single line
/// exceeds the limit, in which case that line becomes its own chunk).
pub fn split_text_into_line_chunks(text: &str, chunk_size: usize) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut chunks: Vec<String> = Vec::new();
    let mut current_chunk: Vec<&str> = Vec::new();
    let mut current_size: usize = 0;

    for line in lines {
        let line_size = line.len() + if current_chunk.is_empty() { 0 } else { 1 };

        if current_size + line_size > chunk_size && !current_chunk.is_empty() {
            // Push current chunk and start new one
            chunks.push(current_chunk.join("\n"));
            current_chunk = vec![line];
            current_size = line.len();
        } else {
            current_chunk.push(line);
            current_size += line_size;
        }
    }

    // Remember the last chunk
    if !current_chunk.is_empty() {
        chunks.push(current_chunk.join("\n"));
    }

    chunks
}

/// Checks if an HTTP response is successful, and if not, returns a detailed error.
///
/// This helper extracts both the status code and response body for better error messages.
///
/// # Arguments
/// * `response` - The reqwest Response to check
///
/// # Returns
/// Ok(response) if successful, or Err(TranslationError) with details if not
pub async fn check_response_status(
    response: reqwest::Response,
) -> Result<reqwest::Response, TranslationError> {
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(TranslationError::ApiError(format!(
            "HTTP {}: {}",
            status, text
        )));
    }
    Ok(response)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_empty_text() {
        let chunks = split_text_into_line_chunks("", 100);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_split_single_line() {
        let text = "Hello world";
        let chunks = split_text_into_line_chunks(text, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_split_multiple_lines_fits() {
        let text = "Line 1\nLine 2\nLine 3";
        let chunks = split_text_into_line_chunks(text, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_split_multiple_chunks_needed() {
        let text = "Line 1\nLine 2\nLine 3\nLine 4";
        let chunks = split_text_into_line_chunks(text, 15);
        // "Line 1\nLine 2" = 13 chars
        // "Line 3\nLine 4" = 13 chars
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "Line 1\nLine 2");
        assert_eq!(chunks[1], "Line 3\nLine 4");
    }

    #[test]
    fn test_split_single_long_line() {
        let text = "This is a very long line that exceeds the chunk size limit";
        let chunks = split_text_into_line_chunks(text, 20);
        // Should keep the whole line as one chunk even though it exceeds limit
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_split_with_empty_lines() {
        let text = "Line 1\n\nLine 3";
        let chunks = split_text_into_line_chunks(text, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }
}
