/// Trait for extracting data from documents (OCR/NLP).
/// Default NoopProcessor — plug in a real lib post-MVP.
///
/// ADR: anticipatory decision, zero refactoring needed when adding a real processor.

use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DocumentMeta {
    pub id: Uuid,
    pub original_name: String,
    pub mime_type: String,
    pub storage_path: String,
    pub bucket: String,
}

#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub data: Value,      // flexible JSONB per document type
    pub confidence: f32,  // 0.0 - 1.0
}

pub trait DocumentProcessor: Send + Sync {
    async fn extract(&self, doc: &DocumentMeta) -> anyhow::Result<ExtractionResult>;
    fn supported_types(&self) -> &[&str];
}

/// Default implementation — does nothing (MVP)
pub struct NoopProcessor;

impl DocumentProcessor for NoopProcessor {
    async fn extract(&self, _doc: &DocumentMeta) -> anyhow::Result<ExtractionResult> {
        Ok(ExtractionResult {
            data: serde_json::json!({}),
            confidence: 0.0,
        })
    }

    fn supported_types(&self) -> &[&str] {
        &[]
    }
}
