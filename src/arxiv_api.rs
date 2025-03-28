use arxiv::{Arxiv, ArxivQueryBuilder};

struct SearchQueryHandler {
    title: String,
    author: String,
    paper_abstract: String,
    comment: String,
    journal_reference: String,
    category: String,
    report_number: String,
    id: String,
    all_categories: String,
}

impl SearchQueryHandler {
    fn new() -> SearchQueryHandler {
        SearchQueryHandler {
            title: String::new(),
            author: String::new(),
            paper_abstract: String::new(),
            comment: String::new(),
            journal_reference: String::new(),
            category: String::new(),
            report_number: String::new(),
            id: String::new(),
            all_categories: String::new(),
        }
    }
    fn get_search_query_string(&self) -> String {
        let mut search_query_vector: Vec<String> = Vec::new();

        if !self.title.is_empty() {
            search_query_vector.push("ti:".to_string() + &self.title.as_str().replace(" ", "+"));
        }
        if !self.author.is_empty() {
            search_query_vector.push("au:".to_string() + &self.author.as_str().replace(" ", "+"));
        }
        if !self.paper_abstract.is_empty() {
            search_query_vector
                .push("abs:".to_string() + &self.paper_abstract.as_str().replace(" ", "+"));
        }
        if !self.comment.is_empty() {
            search_query_vector.push("co:".to_string() + &self.comment.as_str().replace(" ", "+"));
        }
        if !self.journal_reference.is_empty() {
            search_query_vector
                .push("jr:".to_string() + &self.journal_reference.as_str().replace(" ", "+"));
        }
        if !self.category.is_empty() {
            search_query_vector
                .push("cat:".to_string() + &self.category.as_str().replace(" ", "+"));
        }
        if !self.report_number.is_empty() {
            search_query_vector
                .push("rn:".to_string() + &self.report_number.as_str().replace(" ", "+"));
        }
        if !self.id.is_empty() {
            search_query_vector.push("id:".to_string() + &self.id.as_str().replace(" ", "+"));
        }
        if !self.all_categories.is_empty() {
            search_query_vector
                .push("all:".to_string() + &self.all_categories.as_str().replace(" ", "+"));
        }

        search_query_vector.join("+AND+")
    }
    fn title(mut self, title: String) -> SearchQueryHandler {
        self.title = title;
        self
    }
    fn author(mut self, author: String) -> SearchQueryHandler {
        self.author = author;
        self
    }
    fn paper_abstract(mut self, paper_abstract: String) -> SearchQueryHandler {
        self.paper_abstract = paper_abstract;
        self
    }
    fn comment(mut self, comment: String) -> SearchQueryHandler {
        self.comment = comment;
        self
    }
    fn journal_reference(mut self, journal_reference: String) -> SearchQueryHandler {
        self.journal_reference = journal_reference;
        self
    }
    fn category(mut self, category: String) -> SearchQueryHandler {
        self.category = category;
        self
    }
    fn report_number(mut self, report_number: String) -> SearchQueryHandler {
        self.report_number = report_number;
        self
    }
    fn id(mut self, id: String) -> SearchQueryHandler {
        self.id = id;
        self
    }
    fn all_categories(mut self, all_categories: String) -> SearchQueryHandler {
        self.all_categories = all_categories;
        self
    }
}

#[tokio::main]
pub async fn get_papers() -> Vec<Arxiv> {
    let search_query_handler: SearchQueryHandler =
        SearchQueryHandler::new().all_categories(String::from("gravitational waves"));

    let query = ArxivQueryBuilder::new()
        .search_query(search_query_handler.get_search_query_string().as_str())
        .start(0)
        .max_results(5)
        .sort_by("submittedDate")
        .sort_order("descending")
        .build();
    arxiv::fetch_arxivs(query)
        .await
        .inspect(|x| println!("Fetched {} papers.", x.len()))
        .inspect_err(|x| println!("Fetching failed: {x}"))
        .unwrap_or_default()
}
