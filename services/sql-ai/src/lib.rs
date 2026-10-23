//! Gravity SQL AI: dependency-free lexical retrieval foundation.
//!
//! This is a small, testable RAG retrieval component, not a SQL database,
//! embedding model, or production vector store. It returns source passages
//! for a separate language model to use as grounded context.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchHit {
    pub id: String,
    pub title: String,
    pub excerpt: String,
    pub score: usize,
}

#[derive(Default, Debug)]
pub struct KnowledgeBase {
    documents: Vec<Document>,
}

impl KnowledgeBase {
    pub fn new() -> Self { Self::default() }

    pub fn upsert(&mut self, document: Document) {
        if let Some(existing) = self.documents.iter_mut().find(|d| d.id == document.id) {
            *existing = document;
        } else {
            self.documents.push(document);
        }
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.documents.len();
        self.documents.retain(|d| d.id != id);
        before != self.documents.len()
    }

    /// Rank documents by distinct query terms found in title and body.
    /// This is lexical baseline retrieval; embeddings/vector search come later.
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        let terms = tokens(query);
        if terms.is_empty() || limit == 0 { return Vec::new(); }

        let mut hits: Vec<SearchHit> = self.documents.iter().filter_map(|doc| {
            let title = tokens(&doc.title);
            let body = tokens(&doc.text);
            let score = terms.iter().map(|term| {
                (if title.contains(term) { 3 } else { 0 })
                    + if body.contains(term) { 1 } else { 0 }
            }).sum::<usize>();
            if score == 0 { return None; }
            let excerpt: String = doc.text.chars().take(320).collect();
            Some(SearchHit { id: doc.id.clone(), title: doc.title.clone(), excerpt, score })
        }).collect();

        hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
        hits.truncate(limit);
        hits
    }

    pub fn len(&self) -> usize { self.documents.len() }
    pub fn is_empty(&self) -> bool { self.documents.is_empty() }
}

fn tokens(input: &str) -> Vec<String> {
    input.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieves_relevant_document_and_prefers_title_match() {
        let mut kb = KnowledgeBase::new();
        kb.upsert(Document { id: "a".into(), title: "Gravity IPC".into(),
            text: "Processes exchange messages through endpoints.".into() });
        kb.upsert(Document { id: "b".into(), title: "Desktop".into(),
            text: "Gravity IPC lets tasks exchange messages.".into() });
        let hits = kb.search("gravity ipc", 5);
        assert_eq!(hits[0].id, "a");
        assert_eq!(hits[0].score, 6);
    }

    #[test]
    fn upsert_and_remove_work() {
        let mut kb = KnowledgeBase::new();
        let doc = Document { id: "one".into(), title: "A".into(), text: "text".into() };
        kb.upsert(doc.clone());
        kb.upsert(Document { text: "updated".into(), ..doc });
        assert_eq!(kb.len(), 1);
        assert!(kb.remove("one"));
        assert!(kb.is_empty());
    }

    #[test]
    fn empty_query_returns_no_hits() {
        let mut kb = KnowledgeBase::new();
        kb.upsert(Document { id: "one".into(), title: "A".into(), text: "text".into() });
        assert!(kb.search(" !!! ", 5).is_empty());
    }
}
