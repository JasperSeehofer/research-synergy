pub struct SearchQueryHandler {
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
    pub fn new() -> SearchQueryHandler {
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
    pub fn get_arxiv_search_query_string(&self) -> String {
        let mut search_query_vector: Vec<String> = Vec::new();

        if !self.title.is_empty() {
            search_query_vector
                .push("ti:%22".to_string() + &self.title.as_str().replace(" ", "+") + "%22");
        }
        if !self.author.is_empty() {
            search_query_vector
                .push("au:%22".to_string() + &self.author.as_str().replace(" ", "+") + "%22");
        }
        if !self.paper_abstract.is_empty() {
            search_query_vector.push(
                "abs:%22".to_string() + &self.paper_abstract.as_str().replace(" ", "+") + "%22",
            );
        }
        if !self.comment.is_empty() {
            search_query_vector
                .push("co:%22".to_string() + &self.comment.as_str().replace(" ", "+") + "%22");
        }
        if !self.journal_reference.is_empty() {
            search_query_vector.push(
                "jr:%22".to_string() + &self.journal_reference.as_str().replace(" ", "+") + "%22",
            );
        }
        if !self.category.is_empty() {
            search_query_vector
                .push("cat:%22".to_string() + &self.category.as_str().replace(" ", "+") + "%22");
        }
        if !self.report_number.is_empty() {
            search_query_vector.push(
                "rn:%22".to_string() + &self.report_number.as_str().replace(" ", "+") + "%22",
            );
        }
        if !self.id.is_empty() {
            search_query_vector
                .push("id:%22".to_string() + &self.id.as_str().replace(" ", "+") + "%22");
        }
        if !self.all_categories.is_empty() {
            search_query_vector.push(
                "all:%22".to_string() + &self.all_categories.as_str().replace(" ", "+") + "%22",
            );
        }

        search_query_vector.join("+AND+")
    }
    pub fn title(mut self, title: &str) -> SearchQueryHandler {
        self.title = String::from(title);
        self
    }
    pub fn author(mut self, author: &str) -> SearchQueryHandler {
        self.author = String::from(author);
        self
    }
    pub fn paper_abstract(mut self, paper_abstract: &str) -> SearchQueryHandler {
        self.paper_abstract = String::from(paper_abstract);
        self
    }
    pub fn comment(mut self, comment: &str) -> SearchQueryHandler {
        self.comment = String::from(comment);
        self
    }
    pub fn journal_reference(mut self, journal_reference: &str) -> SearchQueryHandler {
        self.journal_reference = String::from(journal_reference);
        self
    }
    pub fn category(mut self, category: &str) -> SearchQueryHandler {
        self.category = String::from(category);
        self
    }
    pub fn report_number(mut self, report_number: &str) -> SearchQueryHandler {
        self.report_number = String::from(report_number);
        self
    }
    pub fn id(mut self, id: &str) -> SearchQueryHandler {
        self.id = String::from(id);
        self
    }
    pub fn all_categories(mut self, all_categories: &str) -> SearchQueryHandler {
        self.all_categories = String::from(all_categories);
        self
    }
}

#[cfg(test)]
mod test {
    use crate::data_aggregation::search_query_handler;

    #[test]
    fn test_get_arxiv_search_query_string() {
        use super::SearchQueryHandler;

        let search_query_handler = SearchQueryHandler::new().title("dark sirens");

        assert_eq!(
            search_query_handler.get_arxiv_search_query_string(),
            String::from("ti:dark+sirens")
        );

        let search_query_handler_several_fields = SearchQueryHandler::new()
            .title("dark sirens")
            .author("Bob Ross");

        assert_eq!(
            search_query_handler_several_fields.get_arxiv_search_query_string(),
            String::from("ti:dark+sirens+AND+au:Bob+Ross")
        );
    }
    #[test]
    fn test_seach_query_setter_function() {
        use super::SearchQueryHandler;

        let search_query_handler = SearchQueryHandler::new().report_number("123").title("");

        assert!(search_query_handler.title.is_empty());
    }
}
