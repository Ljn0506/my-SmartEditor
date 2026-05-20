use meilisearch_sdk::client::Client;
use meilisearch_sdk::settings::Settings;

use crate::error::{AppError, Result};
use crate::models::{BusinessDomain, ContentModule, DocAttr, ProjectPhase, SearchResult, Template};

pub struct SearchEngine {
    client: Client,
    index_name: String,
}

impl SearchEngine {
    pub fn new(host: &str, api_key: Option<&str>) -> Result<Self> {
        crate::ai::validate_ai_url(host)?;
        let client = Client::new(host, api_key).map_err(|e| AppError::Search(e.to_string()))?;
        Ok(Self {
            client,
            index_name: "templates".to_string(),
        })
    }

    pub async fn init_index(&self) -> Result<()> {
        let index = self.client.index(&self.index_name);
        let settings = Settings::new().with_searchable_attributes(["title", "content", "tags"]);
        index.set_settings(&settings).await?;
        Ok(())
    }

    pub async fn add_or_update_templates(&self, templates: &[Template]) -> Result<()> {
        let index = self.client.index(&self.index_name);
        let docs: Vec<SearchDoc> = templates
            .iter()
            .filter(|t| t.id.is_some())
            .map(|t| SearchDoc::from(t.clone()))
            .collect();
        index.add_or_update(&docs, Some("id")).await?;
        Ok(())
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        offset: Option<usize>,
    ) -> Result<Vec<SearchResult>> {
        let index = self.client.index(&self.index_name);
        let mut search = index.search();
        search.with_query(query).with_limit(limit);
        if let Some(o) = offset {
            search.with_offset(o);
        }
        let results = search.execute::<SearchDoc>().await?;

        let hits = results
            .hits
            .into_iter()
            .map(|hit| SearchResult {
                id: hit.result.id,
                title: hit.result.title,
                content: hit.result.content.chars().take(300).collect::<String>() + "...",
                doc_attr: hit.result.doc_attr,
                business_domain: hit.result.business_domain,
                content_module: hit.result.content_module,
                project_phase: hit.result.project_phase,
                tags: hit.result.tags,
                relevance: hit.ranking_score.unwrap_or(0.0),
                source_file: hit.result.source_file,
            })
            .collect();
        Ok(hits)
    }

    #[allow(dead_code)]
    pub async fn delete_template(&self, id: i64) -> Result<()> {
        let index = self.client.index(&self.index_name);
        index.delete_document(id.to_string()).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SearchDoc {
    id: String,
    title: String,
    content: String,
    doc_attr: Option<DocAttr>,
    business_domain: Option<BusinessDomain>,
    content_module: Option<ContentModule>,
    project_phase: Option<ProjectPhase>,
    tags: Vec<String>,
    source_file: Option<String>,
}

impl From<Template> for SearchDoc {
    fn from(t: Template) -> Self {
        Self {
            id: t.id.map(|i| i.to_string()).unwrap_or_default(),
            title: t.title,
            content: t.content,
            doc_attr: t.doc_attr,
            business_domain: t.business_domain,
            content_module: t.content_module,
            project_phase: t.project_phase,
            tags: t.tags,
            source_file: t.source_file,
        }
    }
}
