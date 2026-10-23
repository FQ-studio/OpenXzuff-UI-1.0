//! Gravity SQL AI: persistent SQLite-backed document storage and lexical retrieval.
//!
//! This is a real SQL-backed baseline for RAG. It does not yet implement
//! embeddings, vector search, model inference, or a user-mode Gravity service.

use rusqlite::{params, Connection, Result};
use std::path::Path;

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

pub struct KnowledgeBase {
    connection: Connection,
}

impl KnowledgeBase {
    /// Create an in-memory SQLite database. Useful for tests and temporary work.
    pub fn new() -> Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    /// Open or create a durable SQLite database at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }

    fn from_connection(connection: Connection) -> Result<Self> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS documents (
                 id TEXT PRIMARY KEY NOT NULL,
                 title TEXT NOT NULL,
                 body TEXT NOT NULL,
                 updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
             );
             CREATE INDEX IF NOT EXISTS documents_title_idx ON documents(title);",
        )?;
        Ok(Self { connection })
    }

    /// Insert a document or replace its contents when the ID already exists.
    pub fn upsert(&self, document: &Document) -> Result<()> {
        self.connection.execute(
            "INSERT INTO documents (id, title, body, updated_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET
                 title = excluded.title,
                 body = excluded.body,
                 updated_at = CURRENT_TIMESTAMP",
            params![document.id, document.title, document.text],
        )?;
        Ok(())
    }

    pub fn remove(&self, id: &str) -> Result<bool> {
        Ok(self.connection.execute(
            "DELETE FROM documents WHERE id = ?1",
            params![id],
        )? != 0)
    }

    /// Search SQL-stored documents using a simple title-weighted lexical score.
    /// All SQL values are bound parameters; query text is never interpolated into SQL.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let terms = tokens(query);
        if terms.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let mut statement = self.connection.prepare(
            "SELECT id, title, body FROM documents ORDER BY id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(Document {
                id: row.get(0)?,
                title: row.get(1)?,
                text: row.get(2)?,
            })
        })?;

        let mut hits = Vec::new();
        for row in rows {
            let document = row?;
            let title_tokens = tokens(&document.title);
            let body_tokens = tokens(&document.text);
            let score = terms.iter().map(|term| {
                (if title_tokens.contains(term) { 3 } else { 0 })
                    + if body_tokens.contains(term) { 1 } else { 0 }
            }).sum::<usize>();

            if score > 0 {
                hits.push(SearchHit {
                    id: document.id,
                    title: document.title,
                    excerpt: document.text.chars().take(320).collect(),
                    score,
                });
            }
        }

        hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
        hits.truncate(limit);
        Ok(hits)
    }

    pub fn len(&self) -> Result<usize> {
        self.connection.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }
}

fn tokens(input: &str) -> Vec<String> {
    input.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_persists_documents_and_ranks_title_matches() -> Result<()> {
        let db = KnowledgeBase::new()?;
        db.upsert(&Document {
            id: "a".into(),
            title: "Gravity IPC".into(),
            text: "Processes exchange messages through endpoints.".into(),
        })?;
        db.upsert(&Document {
            id: "b".into(),
            title: "Desktop".into(),
            text: "Gravity IPC lets tasks exchange messages.".into(),
        })?;

        let hits = db.search("gravity ipc", 5)?;
        assert_eq!(hits[0].id, "a");
        assert_eq!(hits[0].score, 6);
        assert_eq!(db.len()?, 2);
        Ok(())
    }

    #[test]
    fn upsert_replaces_existing_record_and_remove_works() -> Result<()> {
        let db = KnowledgeBase::new()?;
        let doc = Document { id: "one".into(), title: "A".into(), text: "old".into() };
        db.upsert(&doc)?;
        db.upsert(&Document { text: "updated".into(), ..doc })?;

        assert_eq!(db.len()?, 1);
        assert_eq!(db.search("updated", 1)?[0].excerpt, "updated");
        assert!(db.remove("one")?);
        assert!(!db.remove("one")?);
        assert!(db.is_empty()?);
        Ok(())
    }

    #[test]
    fn empty_query_returns_no_hits() -> Result<()> {
        let db = KnowledgeBase::new()?;
        db.upsert(&Document {
            id: "one".into(), title: "A".into(), text: "text".into(),
        })?;
        assert!(db.search(" !!! ", 5)?.is_empty());
        Ok(())
    }
}
