// Input validation utilities

use regex::Regex;

lazy_static::lazy_static! {
    static ref URL_REGEX: Regex =
        Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
    static ref EMAIL_REGEX: Regex =
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    static ref DOMAIN_REGEX: Regex =
        Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
}

/// Validate a URL
pub fn validate_url(url: &str) -> anyhow::Result<()> {
    if URL_REGEX.is_match(url) {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Invalid URL: {}", url))
    }
}

/// Validate an email address
pub fn validate_email(email: &str) -> anyhow::Result<()> {
    if EMAIL_REGEX.is_match(email) {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Invalid email: {}", email))
    }
}

/// Validate a domain name
pub fn validate_domain(domain: &str) -> anyhow::Result<()> {
    if DOMAIN_REGEX.is_match(domain) {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Invalid domain: {}", domain))
    }
}

/// Validate a name (entry name, etc.)
pub fn validate_name(name: &str) -> anyhow::Result<()> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("Name cannot be empty"));
    }

    if name.len() > 100 {
        return Err(anyhow::anyhow!("Name too long (max 100 characters)"));
    }

    // Check for valid characters (alphanumeric, hyphen, underscore, dot)
    let valid_chars = name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

    if !valid_chars {
        return Err(anyhow::anyhow!(
            "Name contains invalid characters (only alphanumeric, -, _, . allowed)"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://api.example.com").is_ok());
        assert!(validate_url("http://localhost:8080").is_ok());
        assert!(validate_url("invalid-url").is_err());
        assert!(validate_url("").is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user+tag@domain.co.uk").is_ok());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
    }

    #[test]
    fn test_validate_name() {
        assert!(validate_name("valid-name").is_ok());
        assert!(validate_name("valid_name.test").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name("invalid name").is_err());
        assert!(validate_name("a".repeat(101).as_str()).is_err());
    }
}
