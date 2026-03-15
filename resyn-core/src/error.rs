use std::fmt;

#[derive(Debug)]
pub enum ResynError {
    ArxivApi(String),
    HtmlDownload(String),
    #[cfg(feature = "ssr")]
    HttpRequest(reqwest::Error),
    PaperNotFound(String),
    InvalidPaperId(String),
    NoArxivLink,
    InspireHepApi(String),
    Database(String),
    LlmApi(String),
}

impl fmt::Display for ResynError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResynError::ArxivApi(msg) => write!(f, "arXiv API error: {msg}"),
            ResynError::HtmlDownload(msg) => write!(f, "HTML download error: {msg}"),
            #[cfg(feature = "ssr")]
            ResynError::HttpRequest(e) => write!(f, "HTTP request error: {e}"),
            ResynError::PaperNotFound(id) => write!(f, "paper not found: {id}"),
            ResynError::InvalidPaperId(id) => write!(f, "invalid paper ID: {id}"),
            ResynError::NoArxivLink => write!(f, "no arXiv link found in reference"),
            ResynError::InspireHepApi(msg) => write!(f, "InspireHEP API error: {msg}"),
            ResynError::Database(msg) => write!(f, "database error: {msg}"),
            ResynError::LlmApi(msg) => write!(f, "LLM API error: {msg}"),
        }
    }
}

impl std::error::Error for ResynError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "ssr")]
            ResynError::HttpRequest(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(feature = "ssr")]
impl From<reqwest::Error> for ResynError {
    fn from(e: reqwest::Error) -> Self {
        ResynError::HttpRequest(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_api_error_display() {
        let err = ResynError::LlmApi("connection refused".to_string());
        assert_eq!(format!("{err}"), "LLM API error: connection refused");
    }
}
