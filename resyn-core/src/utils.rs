#[cfg(feature = "ssr")]
use std::time::Duration;

pub fn strip_version_suffix(id: &str) -> String {
    if let Some(pos) = id.rfind('v')
        && pos + 1 < id.len()
        && id[pos + 1..].chars().all(|c| c.is_ascii_digit())
    {
        return id[..pos].to_string();
    }
    id.to_string()
}

#[cfg(feature = "ssr")]
pub fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to create HTTP client")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_version_suffix() {
        assert_eq!(strip_version_suffix("2301.12345v2"), "2301.12345");
        assert_eq!(strip_version_suffix("2301.12345v12"), "2301.12345");
        assert_eq!(strip_version_suffix("2301.12345"), "2301.12345");
        assert_eq!(strip_version_suffix("hep-ph/0601234v1"), "hep-ph/0601234");
        assert_eq!(strip_version_suffix(""), "");
        assert_eq!(strip_version_suffix("v1"), "");
        assert_eq!(strip_version_suffix("noversionhere"), "noversionhere");
    }
}
