use async_trait::async_trait;

use crate::datamodels::paper::Paper;
use crate::error::ResynError;

use super::arxiv_api::get_paper_by_id;
use super::arxiv_utils::aggregate_references_for_arxiv_paper;
use super::html_parser::ArxivHTMLDownloader;
use super::traits::PaperSource;

pub struct ArxivSource {
    pub downloader: ArxivHTMLDownloader,
}

impl ArxivSource {
    pub fn new(downloader: ArxivHTMLDownloader) -> Self {
        Self { downloader }
    }
}

#[async_trait]
impl PaperSource for ArxivSource {
    async fn fetch_paper(&self, id: &str) -> Result<Paper, ResynError> {
        let arxiv_paper = get_paper_by_id(id).await?;
        Paper::from_arxiv_paper(&arxiv_paper)
    }

    async fn fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError> {
        aggregate_references_for_arxiv_paper(paper, &mut self.downloader).await
    }

    fn source_name(&self) -> &'static str {
        "arxiv"
    }
}
