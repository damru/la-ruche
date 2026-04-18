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
    pub data: Value,     // flexible JSONB per document type
    pub confidence: f32, // 0.0 - 1.0
}

#[allow(async_fn_in_trait)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> DocumentMeta {
        DocumentMeta {
            id: Uuid::new_v4(),
            original_name: "facture.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            storage_path: "documents/facture.pdf".to_string(),
            bucket: "laruche".to_string(),
        }
    }

    #[test]
    fn noop_processor_supports_no_types() {
        assert!(NoopProcessor.supported_types().is_empty());
    }

    #[tokio::test]
    async fn noop_processor_extract_returns_empty_low_confidence_result() {
        let result = NoopProcessor.extract(&sample_doc()).await.unwrap();
        assert_eq!(result.data, serde_json::json!({}));
        assert_eq!(result.confidence, 0.0);
    }
}
