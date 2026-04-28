use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{AppError, Result};
use crate::models::Template;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                content_html TEXT,
                doc_attr TEXT,
                business_domain TEXT,
                security_layer TEXT,
                content_module TEXT,
                project_phase TEXT,
                tags TEXT,
                source_file TEXT,
                source_para_range TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                use_count INTEGER DEFAULT 0,
                rating INTEGER DEFAULT 0
            )",
            [],
        )?;
        for stmt in [
            "CREATE INDEX IF NOT EXISTS idx_templates_doc_attr ON templates(doc_attr)",
            "CREATE INDEX IF NOT EXISTS idx_templates_business_domain ON templates(business_domain)",
            "CREATE INDEX IF NOT EXISTS idx_templates_content_module ON templates(content_module)",
            "CREATE INDEX IF NOT EXISTS idx_templates_project_phase ON templates(project_phase)",
            "CREATE INDEX IF NOT EXISTS idx_templates_use_count ON templates(use_count DESC, rating DESC)",
        ] {
            self.conn.execute(stmt, [])?;
        }
        Ok(())
    }

    pub fn insert_template(&self, template: &mut Template) -> Result<i64> {
        let tags_json = serde_json::to_string(&template.tags)?;
        self.conn.execute(
            "INSERT INTO templates (
                title, content, content_html, doc_attr, business_domain,
                security_layer, content_module, project_phase, tags,
                source_file, source_para_range, use_count, rating
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                template.title,
                template.content,
                template.content_html,
                template.doc_attr,
                template.business_domain,
                template.security_layer,
                template.content_module,
                template.project_phase,
                tags_json,
                template.source_file,
                template.source_para_range,
                template.use_count,
                template.rating,
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        template.id = Some(id);
        Ok(id)
    }

    pub fn get_template(&self, id: i64) -> Result<Option<Template>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, content_html, doc_attr, business_domain,
             security_layer, content_module, project_phase, tags,
             source_file, source_para_range, created_at, updated_at, use_count, rating
             FROM templates WHERE id = ?1",
        )?;
        let template = stmt.query_row([id], Self::row_to_template).optional()?;
        Ok(template)
    }

    pub fn search_templates(
        &self,
        doc_attr: Option<&str>,
        business_domain: Option<&str>,
        content_module: Option<&str>,
        project_phase: Option<&str>,
        keyword: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Template>> {
        let mut sql = String::from(
            "SELECT id, title, content, content_html, doc_attr, business_domain,
             security_layer, content_module, project_phase, tags,
             source_file, source_para_range, created_at, updated_at, use_count, rating
             FROM templates WHERE 1=1",
        );
        let mut owned_params: Vec<String> = Vec::new();

        if let Some(v) = doc_attr {
            sql.push_str(" AND doc_attr = ?");
            owned_params.push(v.to_string());
        }
        if let Some(v) = business_domain {
            sql.push_str(" AND business_domain = ?");
            owned_params.push(v.to_string());
        }
        if let Some(v) = content_module {
            sql.push_str(" AND content_module = ?");
            owned_params.push(v.to_string());
        }
        if let Some(v) = project_phase {
            sql.push_str(" AND project_phase = ?");
            owned_params.push(v.to_string());
        }
        if let Some(kw) = keyword {
            sql.push_str(" AND (title LIKE ? OR content LIKE ?)");
            let like = format!("%{}%", kw);
            owned_params.push(like.clone());
            owned_params.push(like);
        }
        // LIMIT 直接嵌入 SQL（usize 来自 Rust 内部，安全）
        sql.push_str(&format!(
            " ORDER BY use_count DESC, rating DESC LIMIT {}",
            limit
        ));

        let param_refs: Vec<&dyn rusqlite::ToSql> = owned_params
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let templates = stmt
            .query_map(param_refs.as_slice(), Self::row_to_template)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(templates)
    }

    pub fn update_template(&self, template: &Template) -> Result<()> {
        let id = template
            .id
            .ok_or_else(|| AppError::Validation("模板 ID 不能为空".to_string()))?;
        let tags_json = serde_json::to_string(&template.tags)?;
        self.conn.execute(
            "UPDATE templates SET
                title = ?1, content = ?2, content_html = ?3, doc_attr = ?4,
                business_domain = ?5, security_layer = ?6, content_module = ?7,
                project_phase = ?8, tags = ?9, source_file = ?10,
                source_para_range = ?11, use_count = ?12, rating = ?13,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?14",
            params![
                template.title,
                template.content,
                template.content_html,
                template.doc_attr,
                template.business_domain,
                template.security_layer,
                template.content_module,
                template.project_phase,
                tags_json,
                template.source_file,
                template.source_para_range,
                template.use_count,
                template.rating,
                id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_template(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM templates WHERE id = ?1", [id])?;
        Ok(())
    }

    fn row_to_template(row: &rusqlite::Row) -> std::result::Result<Template, rusqlite::Error> {
        let tags_json: String = row.get(9)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        Ok(Template {
            id: Some(row.get(0)?),
            title: row.get(1)?,
            content: row.get(2)?,
            content_html: row.get(3)?,
            doc_attr: row.get(4)?,
            business_domain: row.get(5)?,
            security_layer: row.get(6)?,
            content_module: row.get(7)?,
            project_phase: row.get(8)?,
            tags,
            source_file: row.get(10)?,
            source_para_range: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
            use_count: row.get(14)?,
            rating: row.get(15)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Database {
        Database::new(":memory:").unwrap()
    }

    fn sample_template() -> Template {
        Template {
            id: None,
            title: "等保2.0技术方案".to_string(),
            content: "这是一份技术方案内容".to_string(),
            content_html: Some("<p>这是一份技术方案内容</p>".to_string()),
            doc_attr: Some("技术方案".to_string()),
            business_domain: Some("网络安全".to_string()),
            security_layer: Some("防御层".to_string()),
            content_module: Some("技术方案".to_string()),
            project_phase: Some("投标阶段".to_string()),
            tags: vec!["等保".to_string(), "三级".to_string()],
            source_file: Some("/tmp/test.docx".to_string()),
            source_para_range: None,
            created_at: None,
            updated_at: None,
            use_count: 5,
            rating: 4,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let db = create_test_db();
        let mut tmpl = sample_template();
        let id = db.insert_template(&mut tmpl).unwrap();
        assert_eq!(tmpl.id, Some(id));

        let fetched = db.get_template(id).unwrap().unwrap();
        assert_eq!(fetched.title, "等保2.0技术方案");
        assert_eq!(fetched.content, "这是一份技术方案内容");
        assert_eq!(fetched.doc_attr, Some("技术方案".to_string()));
        assert_eq!(fetched.tags, vec!["等保", "三级"]);
        assert_eq!(fetched.use_count, 5);
        assert_eq!(fetched.rating, 4);
    }

    #[test]
    fn test_get_nonexistent() {
        let db = create_test_db();
        assert!(db.get_template(999).unwrap().is_none());
    }

    #[test]
    fn test_search_by_doc_attr() {
        let db = create_test_db();
        let mut tmpl = sample_template();
        db.insert_template(&mut tmpl).unwrap();

        let results = db
            .search_templates(Some("技术方案"), None, None, None, None, 10)
            .unwrap();
        assert_eq!(results.len(), 1);

        let results = db
            .search_templates(Some("投标应答"), None, None, None, None, 10)
            .unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_by_keyword() {
        let db = create_test_db();
        let mut tmpl = sample_template();
        db.insert_template(&mut tmpl).unwrap();

        let results = db
            .search_templates(None, None, None, None, Some("技术方案"), 10)
            .unwrap();
        assert_eq!(results.len(), 1);

        let results = db
            .search_templates(None, None, None, None, Some("不存在的关键词"), 10)
            .unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_limit() {
        let db = create_test_db();
        for i in 0..5 {
            let mut tmpl = sample_template();
            tmpl.title = format!("方案{}", i);
            db.insert_template(&mut tmpl).unwrap();
        }
        let results = db
            .search_templates(None, None, None, None, None, 3)
            .unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_update_template() {
        let db = create_test_db();
        let mut tmpl = sample_template();
        db.insert_template(&mut tmpl).unwrap();
        let id = tmpl.id.unwrap();

        tmpl.title = "更新后的标题".to_string();
        tmpl.use_count = 10;
        db.update_template(&tmpl).unwrap();

        let fetched = db.get_template(id).unwrap().unwrap();
        assert_eq!(fetched.title, "更新后的标题");
        assert_eq!(fetched.use_count, 10);
    }

    #[test]
    fn test_update_without_id_fails() {
        let db = create_test_db();
        let tmpl = sample_template();
        let result = db.update_template(&tmpl);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("模板 ID 不能为空"));
    }

    #[test]
    fn test_delete_template() {
        let db = create_test_db();
        let mut tmpl = sample_template();
        db.insert_template(&mut tmpl).unwrap();
        let id = tmpl.id.unwrap();

        assert!(db.get_template(id).unwrap().is_some());
        db.delete_template(id).unwrap();
        assert!(db.get_template(id).unwrap().is_none());
    }

    #[test]
    fn test_tags_roundtrip() {
        let db = create_test_db();
        let mut tmpl = sample_template();
        tmpl.tags = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        db.insert_template(&mut tmpl).unwrap();

        let fetched = db.get_template(tmpl.id.unwrap()).unwrap().unwrap();
        assert_eq!(fetched.tags, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_empty_tags_roundtrip() {
        let db = create_test_db();
        let mut tmpl = sample_template();
        tmpl.tags = vec![];
        db.insert_template(&mut tmpl).unwrap();

        let fetched = db.get_template(tmpl.id.unwrap()).unwrap().unwrap();
        assert!(fetched.tags.is_empty());
    }
}
