use crate::error::ResynError;

pub fn validate_arxiv_id(id: &str) -> Result<(), ResynError> {
    // Standard arXiv ID format: YYMM.NNNNN (4-5 digit number) with optional version
    // Also supports old format: category/YYMMNNN with optional version
    let is_new_format = {
        let parts: Vec<&str> = id.splitn(2, 'v').collect();
        let base = parts[0];
        let version_ok = parts
            .get(1)
            .is_none_or(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit()));
        let base_ok = if let Some((prefix, suffix)) = base.split_once('.') {
            prefix.len() == 4
                && prefix.chars().all(|c| c.is_ascii_digit())
                && (4..=5).contains(&suffix.len())
                && suffix.chars().all(|c| c.is_ascii_digit())
        } else {
            false
        };
        version_ok && base_ok
    };

    let is_old_format = {
        let parts: Vec<&str> = id.splitn(2, 'v').collect();
        let base = parts[0];
        let version_ok = parts
            .get(1)
            .is_none_or(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit()));
        let base_ok = base.contains('/');
        version_ok && base_ok
    };

    if is_new_format || is_old_format {
        Ok(())
    } else {
        Err(ResynError::InvalidPaperId(id.to_string()))
    }
}

pub fn validate_url(url: &str) -> Result<(), ResynError> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err(ResynError::HtmlDownload(format!("invalid URL: {url}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_arxiv_ids() {
        assert!(validate_arxiv_id("2503.18887").is_ok());
        assert!(validate_arxiv_id("2301.12345").is_ok());
        assert!(validate_arxiv_id("2301.12345v2").is_ok());
        assert!(validate_arxiv_id("2301.12345v12").is_ok());
        assert!(validate_arxiv_id("2301.1234").is_ok());
    }

    #[test]
    fn test_old_format_arxiv_ids() {
        assert!(validate_arxiv_id("hep-ph/0601234").is_ok());
        assert!(validate_arxiv_id("hep-ph/0601234v1").is_ok());
    }

    #[test]
    fn test_invalid_arxiv_ids() {
        assert!(validate_arxiv_id("").is_err());
        assert!(validate_arxiv_id("not-an-id").is_err());
        assert!(validate_arxiv_id("2301").is_err());
        assert!(validate_arxiv_id("2301.").is_err());
        assert!(validate_arxiv_id("2301.123").is_err());
        assert!(validate_arxiv_id("2301.123456").is_err());
        assert!(validate_arxiv_id("230.12345").is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://arxiv.org/abs/2301.12345").is_ok());
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("not-a-url").is_err());
    }
}
