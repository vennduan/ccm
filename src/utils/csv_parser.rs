// CSV Parser - Parse CSV files and detect browser format

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserFormat {
    Edge,
    Chrome,
    Firefox,
    Safari,
    Generic,
}

impl BrowserFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            BrowserFormat::Edge => "edge",
            BrowserFormat::Chrome => "chrome",
            BrowserFormat::Firefox => "firefox",
            BrowserFormat::Safari => "safari",
            BrowserFormat::Generic => "generic",
        }
    }
}

/// Browser format specifications
struct BrowserFormatSpec {
    required_columns: Vec<&'static str>,
}

fn get_browser_specs() -> HashMap<BrowserFormat, BrowserFormatSpec> {
    let mut specs = HashMap::new();

    specs.insert(
        BrowserFormat::Edge,
        BrowserFormatSpec {
            required_columns: vec!["name", "url", "username", "password"],
        },
    );

    specs.insert(
        BrowserFormat::Chrome,
        BrowserFormatSpec {
            required_columns: vec!["url", "username", "password"],
        },
    );

    specs.insert(
        BrowserFormat::Firefox,
        BrowserFormatSpec {
            required_columns: vec!["url", "username", "password"],
        },
    );

    specs.insert(
        BrowserFormat::Safari,
        BrowserFormatSpec {
            required_columns: vec!["Username", "Password", "URL"],
        },
    );

    specs.insert(
        BrowserFormat::Generic,
        BrowserFormatSpec {
            required_columns: vec!["password"],
        },
    );

    specs
}

/// Detect the browser format based on CSV headers
pub fn detect_browser_format(headers: &[String]) -> BrowserFormat {
    let headers_lower: Vec<String> = headers.iter().map(|h| h.to_lowercase()).collect();
    let headers_original: Vec<&str> = headers.iter().map(|h| h.as_str()).collect();
    let _specs = get_browser_specs();

    // Edge format check (has "name" column)
    if headers_lower.contains(&"name".to_string())
        && headers_lower.contains(&"url".to_string())
        && headers_lower.contains(&"username".to_string())
        && headers_lower.contains(&"password".to_string())
    {
        return BrowserFormat::Edge;
    }

    // Safari format check (case-sensitive: "Username", "Password", "URL")
    if headers_original.contains(&"Username")
        && headers_original.contains(&"Password")
        && headers_original.contains(&"URL")
    {
        return BrowserFormat::Safari;
    }

    // Chrome/Firefox format check
    if headers_lower.contains(&"url".to_string())
        && headers_lower.contains(&"username".to_string())
        && headers_lower.contains(&"password".to_string())
    {
        // Firefox has specific columns
        if headers_lower.contains(&"httprealm".to_string())
            || headers_lower.contains(&"formactionorigin".to_string())
        {
            return BrowserFormat::Firefox;
        }
        return BrowserFormat::Chrome;
    }

    // Generic fallback
    if headers_lower.contains(&"password".to_string()) {
        return BrowserFormat::Generic;
    }

    BrowserFormat::Generic
}

/// Parse CSV content into rows
pub fn parse_csv(content: &str) -> Vec<HashMap<String, String>> {
    let lines = split_csv_lines(content);

    if lines.is_empty() {
        return vec![];
    }

    // First line is headers
    let headers = parse_csv_line(&lines[0]);

    if headers.is_empty() {
        return vec![];
    }

    // Parse remaining lines as data rows
    let mut rows = Vec::new();
    for line in lines.iter().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let values = parse_csv_line(trimmed);
        let mut row = HashMap::new();

        for (i, header) in headers.iter().enumerate() {
            let value = values.get(i).cloned().unwrap_or_default();
            row.insert(header.clone(), value);
        }

        rows.push(row);
    }

    rows
}

/// Split CSV content into lines, handling quoted fields with newlines
fn split_csv_lines(content: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut in_quotes = false;

    for ch in content.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current_line.push(ch);
            }
            '\n' if !in_quotes => {
                let trimmed = current_line.trim_end_matches('\r').to_string();
                if !trimmed.is_empty() {
                    lines.push(trimmed);
                }
                current_line.clear();
            }
            _ => {
                current_line.push(ch);
            }
        }
    }

    // Don't forget the last line
    if !current_line.is_empty() {
        let trimmed = current_line.trim_end_matches('\r').to_string();
        if !trimmed.is_empty() {
            lines.push(trimmed);
        }
    }

    lines
}

/// Parse a single CSV line into values
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut prev_char = '\0';

    for ch in line.chars() {
        match ch {
            '"' if !in_quotes => {
                in_quotes = true;
            }
            '"' if in_quotes && prev_char == '"' => {
                // Escaped quote
                current.push('"');
                prev_char = '\0';
                continue;
            }
            '"' if in_quotes => {
                in_quotes = false;
            }
            ',' if !in_quotes => {
                values.push(current.clone());
                current.clear();
                prev_char = ch;
                continue;
            }
            _ => {
                current.push(ch);
            }
        }
        prev_char = ch;
    }

    values.push(current);

    values
}

/// Decode CSV content, handling UTF-8 BOM
pub fn decode_csv_content(bytes: &[u8]) -> String {
    // Check for UTF-8 BOM and skip it
    let content = if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        &bytes[3..]
    } else {
        bytes
    };

    String::from_utf8_lossy(content).to_string()
}

/// Mapped entry from CSV
#[derive(Debug, Clone)]
pub struct MappedEntry {
    pub name: String,
    pub entry_type: String,
    pub secret: String,
    pub metadata: HashMap<String, String>,
}

/// Map CSV rows to entries based on browser format
pub fn map_csv_to_entries(
    rows: &[HashMap<String, String>],
    format: BrowserFormat,
) -> Vec<MappedEntry> {
    let mut entries = Vec::new();

    for (index, row) in rows.iter().enumerate() {
        let entry = match format {
            BrowserFormat::Edge => map_edge_row(row, index),
            BrowserFormat::Chrome => map_chrome_row(row, index),
            BrowserFormat::Firefox => map_firefox_row(row, index),
            BrowserFormat::Safari => map_safari_row(row, index),
            BrowserFormat::Generic => map_generic_row(row, index),
        };

        if let Some(e) = entry {
            entries.push(e);
        }
    }

    entries
}

fn map_edge_row(row: &HashMap<String, String>, index: usize) -> Option<MappedEntry> {
    let password = get_field(row, &["password"])?;
    if password.is_empty() {
        return None;
    }

    let name = get_field(row, &["name"])
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("import-{}", index + 1));

    let mut metadata = HashMap::new();
    if let Some(url) = get_field(row, &["url"]) {
        if !url.is_empty() {
            metadata.insert("url".to_string(), url);
        }
    }
    if let Some(username) = get_field(row, &["username"]) {
        if !username.is_empty() {
            metadata.insert("username".to_string(), username);
        }
    }

    Some(MappedEntry {
        name,
        entry_type: "password".to_string(),
        secret: password,
        metadata,
    })
}

fn map_chrome_row(row: &HashMap<String, String>, index: usize) -> Option<MappedEntry> {
    let password = get_field(row, &["password"])?;
    if password.is_empty() {
        return None;
    }

    // Try to derive name from URL or use group/name
    let name = get_field(row, &["name", "group"])
        .filter(|s| !s.is_empty())
        .or_else(|| get_field(row, &["url"]).and_then(|url| extract_domain(&url)))
        .unwrap_or_else(|| format!("import-{}", index + 1));

    let mut metadata = HashMap::new();
    if let Some(url) = get_field(row, &["url"]) {
        if !url.is_empty() {
            metadata.insert("url".to_string(), url);
        }
    }
    if let Some(username) = get_field(row, &["username"]) {
        if !username.is_empty() {
            metadata.insert("username".to_string(), username);
        }
    }

    Some(MappedEntry {
        name,
        entry_type: "password".to_string(),
        secret: password,
        metadata,
    })
}

fn map_firefox_row(row: &HashMap<String, String>, index: usize) -> Option<MappedEntry> {
    map_chrome_row(row, index) // Same mapping as Chrome
}

fn map_safari_row(row: &HashMap<String, String>, index: usize) -> Option<MappedEntry> {
    let password = get_field(row, &["Password"])?;
    if password.is_empty() {
        return None;
    }

    let name = get_field(row, &["Title"])
        .filter(|s| !s.is_empty())
        .or_else(|| get_field(row, &["URL"]).and_then(|url| extract_domain(&url)))
        .unwrap_or_else(|| format!("import-{}", index + 1));

    let mut metadata = HashMap::new();
    if let Some(url) = get_field(row, &["URL"]) {
        if !url.is_empty() {
            metadata.insert("url".to_string(), url);
        }
    }
    if let Some(username) = get_field(row, &["Username"]) {
        if !username.is_empty() {
            metadata.insert("username".to_string(), username);
        }
    }

    Some(MappedEntry {
        name,
        entry_type: "password".to_string(),
        secret: password,
        metadata,
    })
}

fn map_generic_row(row: &HashMap<String, String>, index: usize) -> Option<MappedEntry> {
    let password = get_field(row, &["password"])?;
    if password.is_empty() {
        return None;
    }

    let name = get_field(row, &["name", "title"])
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("import-{}", index + 1));

    let mut metadata = HashMap::new();
    if let Some(url) = get_field(row, &["url"]) {
        if !url.is_empty() {
            metadata.insert("url".to_string(), url);
        }
    }
    if let Some(username) = get_field(row, &["username", "user"]) {
        if !username.is_empty() {
            metadata.insert("username".to_string(), username);
        }
    }

    Some(MappedEntry {
        name,
        entry_type: "password".to_string(),
        secret: password,
        metadata,
    })
}

/// Get field from row, trying multiple possible column names
fn get_field(row: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    for key in keys {
        // Try exact match
        if let Some(value) = row.get(*key) {
            return Some(value.clone());
        }
        // Try case-insensitive match
        for (k, v) in row {
            if k.to_lowercase() == key.to_lowercase() {
                return Some(v.clone());
            }
        }
    }
    None
}

/// Extract domain from URL for use as name
fn extract_domain(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    // Remove protocol
    let without_protocol = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    // Get domain (before any path)
    let domain = without_protocol.split('/').next()?;

    // Remove www. prefix
    let clean_domain = domain.strip_prefix("www.").unwrap_or(domain);

    if clean_domain.is_empty() {
        None
    } else {
        Some(clean_domain.to_string())
    }
}

/// Resolve duplicate names by appending suffixes
pub fn resolve_duplicate_names(
    entries: Vec<MappedEntry>,
    existing_names: &std::collections::HashSet<String>,
) -> (Vec<MappedEntry>, usize, Vec<(String, String)>) {
    let mut result = Vec::new();
    let mut used_names = existing_names.clone();
    let mut renamed_count = 0;
    let mut renamed_entries = Vec::new();

    for mut entry in entries {
        let original_name = entry.name.clone();
        let mut name = original_name.clone();
        let mut suffix = 1;

        while used_names.contains(&name) {
            name = format!("{}-{}", original_name, suffix);
            suffix += 1;
        }

        if name != original_name {
            renamed_count += 1;
            renamed_entries.push((original_name, name.clone()));
        }

        used_names.insert(name.clone());
        entry.name = name;
        result.push(entry);
    }

    (result, renamed_count, renamed_entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_simple() {
        let csv = "name,url,username,password\nTest,https://example.com,user,pass123";
        let rows = parse_csv(csv);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("name"), Some(&"Test".to_string()));
        assert_eq!(rows[0].get("password"), Some(&"pass123".to_string()));
    }

    #[test]
    fn test_parse_csv_quoted_fields() {
        let csv = r#"name,url,username,password
"Test, with comma","https://example.com","user","pass,123""#;
        let rows = parse_csv(csv);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("name"), Some(&"Test, with comma".to_string()));
        assert_eq!(rows[0].get("password"), Some(&"pass,123".to_string()));
    }

    #[test]
    fn test_detect_edge_format() {
        let headers = vec![
            "name".to_string(),
            "url".to_string(),
            "username".to_string(),
            "password".to_string(),
        ];
        assert_eq!(detect_browser_format(&headers), BrowserFormat::Edge);
    }

    #[test]
    fn test_detect_safari_format() {
        let headers = vec![
            "Title".to_string(),
            "URL".to_string(),
            "Username".to_string(),
            "Password".to_string(),
        ];
        assert_eq!(detect_browser_format(&headers), BrowserFormat::Safari);
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://www.example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_domain("http://example.com"),
            Some("example.com".to_string())
        );
    }
}
