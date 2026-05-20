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
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS cards (
                id TEXT PRIMARY KEY,
                chapter TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                source_refs TEXT NOT NULL DEFAULT '[]',
                document_target TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'draft',
                generated_by TEXT NOT NULL DEFAULT 'ai',
                related_cards TEXT NOT NULL DEFAULT '[]',
                param_placeholders TEXT NOT NULL DEFAULT '[]',
                risk_flags TEXT NOT NULL DEFAULT '[]',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cards_document_target ON cards(document_target)",
            [],
        )?;
        if let Err(e) = self.conn.execute(
            "ALTER TABLE cards ADD COLUMN IF NOT EXISTS param_placeholders TEXT NOT NULL DEFAULT '[]'",
            [],
        ) {
            log::warn!("ALTER TABLE cards ADD COLUMN param_placeholders 失败（可能已存在）: {}", e);
        }
        if let Err(e) = self.conn.execute(
            "ALTER TABLE cards ADD COLUMN IF NOT EXISTS risk_flags TEXT NOT NULL DEFAULT '[]'",
            [],
        ) {
            log::warn!("ALTER TABLE cards ADD COLUMN risk_flags 失败（可能已存在）: {}", e);
        }
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS global_params (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
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
        filter: &crate::models::TemplateFilter,
    ) -> Result<Vec<Template>> {
        let mut sql = String::from(
            "SELECT id, title, content, content_html, doc_attr, business_domain,
             security_layer, content_module, project_phase, tags,
             source_file, source_para_range, created_at, updated_at, use_count, rating
             FROM templates WHERE 1=1",
        );
        let mut owned_params: Vec<String> = Vec::new();

        if let Some(v) = &filter.doc_attr {
            sql.push_str(" AND doc_attr = ?");
            owned_params.push(v.to_string());
        }
        if let Some(v) = &filter.business_domain {
            sql.push_str(" AND business_domain = ?");
            owned_params.push(v.to_string());
        }
        if let Some(v) = &filter.content_module {
            sql.push_str(" AND content_module = ?");
            owned_params.push(v.to_string());
        }
        if let Some(v) = &filter.project_phase {
            sql.push_str(" AND project_phase = ?");
            owned_params.push(v.to_string());
        }
        if let Some(kw) = &filter.keyword {
            sql.push_str(" AND (title LIKE ? OR content LIKE ?)");
            let like = format!("%{}%", kw);
            owned_params.push(like.clone());
            owned_params.push(like);
        }
        // LIMIT / OFFSET 直接嵌入 SQL（usize 来自 Rust 内部，安全）
        let limit = filter.limit.min(1000);
        sql.push_str(&format!(
            " ORDER BY use_count DESC, rating DESC LIMIT {}",
            limit
        ));
        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

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

    // --- Phase 2: 卡片持久化 ---

    pub fn save_cards(&self, document_target: &str, cards: &[crate::models::Card]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        // UPSERT：先插入/替换，再删除不在列表中的旧卡片，避免 DELETE 后崩溃导致数据丢失
        for card in cards {
            let source_refs_json = serde_json::to_string(&card.source_refs)?;
            let related_json = serde_json::to_string(&card.related_cards)?;
            let param_json = serde_json::to_string(&card.param_placeholders)?;
            let risk_json = serde_json::to_string(&card.risk_flags)?;
            tx.execute(
                "INSERT INTO cards (
                    id, chapter, title, content, source_refs, document_target,
                    status, generated_by, related_cards, param_placeholders, risk_flags
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(id) DO UPDATE SET
                    chapter = excluded.chapter,
                    title = excluded.title,
                    content = excluded.content,
                    source_refs = excluded.source_refs,
                    document_target = excluded.document_target,
                    status = excluded.status,
                    generated_by = excluded.generated_by,
                    related_cards = excluded.related_cards,
                    param_placeholders = excluded.param_placeholders,
                    risk_flags = excluded.risk_flags",
                params![
                    card.id,
                    card.chapter,
                    card.title,
                    card.content,
                    source_refs_json,
                    card.document_target,
                    card.status.as_str(),
                    card.generated_by,
                    related_json,
                    param_json,
                    risk_json,
                ],
            )?;
        }
        // 清理不在新列表中的旧卡片
        if cards.is_empty() {
            tx.execute("DELETE FROM cards WHERE document_target = ?1", [document_target])?;
        } else {
            let ids: Vec<&str> = cards.iter().map(|c| c.id.as_str()).collect();
            let placeholders = (0..ids.len()).map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "DELETE FROM cards WHERE document_target = ?1 AND id NOT IN ({})",
                placeholders
            );
            let mut stmt = tx.prepare(&sql)?;
            let mut params: Vec<&dyn rusqlite::ToSql> = vec![&document_target];
            for id in &ids {
                params.push(id);
            }
            stmt.execute(rusqlite::params_from_iter(params))?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_cards(&self, document_target: &str) -> Result<Vec<crate::models::Card>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, chapter, title, content, source_refs, document_target,
                    status, generated_by, related_cards, param_placeholders, risk_flags
             FROM cards WHERE document_target = ?1 ORDER BY chapter"
        )?;
        let rows = stmt.query_map([document_target], |row| {
            let source_refs_str: String = row.get(4)?;
            let related_str: String = row.get(8)?;
            let status_str: String = row.get(6)?;
            let param_str: String = row.get(9)?;
            let risk_str: String = row.get(10)?;
            Ok(crate::models::Card {
                id: row.get(0)?,
                chapter: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                source_refs: serde_json::from_str(&source_refs_str).unwrap_or_default(),
                document_target: row.get(5)?,
                status: crate::models::CardStatus::from_str(&status_str)
                    .unwrap_or(crate::models::CardStatus::Draft),
                generated_by: row.get(7)?,
                related_cards: serde_json::from_str(&related_str).unwrap_or_default(),
                param_placeholders: serde_json::from_str(&param_str).unwrap_or_default(),
                risk_flags: serde_json::from_str(&risk_str).unwrap_or_default(),
            })
        })?;
        let mut cards = Vec::new();
        for card in rows {
            cards.push(card?);
        }
        Ok(cards)
    }

    pub fn update_card(&self, card: &crate::models::Card) -> Result<()> {
        let source_refs_json = serde_json::to_string(&card.source_refs)?;
        let related_json = serde_json::to_string(&card.related_cards)?;
        let param_json = serde_json::to_string(&card.param_placeholders)?;
        let risk_json = serde_json::to_string(&card.risk_flags)?;
        let rows = self.conn.execute(
            "UPDATE cards SET
                chapter = ?1,
                title = ?2,
                content = ?3,
                source_refs = ?4,
                document_target = ?5,
                status = ?6,
                generated_by = ?7,
                related_cards = ?8,
                param_placeholders = ?9,
                risk_flags = ?10,
                updated_at = CURRENT_TIMESTAMP
             WHERE id = ?11",
            params![
                card.chapter,
                card.title,
                card.content,
                source_refs_json,
                card.document_target,
                card.status.as_str(),
                card.generated_by,
                related_json,
                param_json,
                risk_json,
                card.id,
            ],
        )?;
        if rows == 0 {
            return Err(AppError::Validation(format!("卡片 {} 不存在", card.id)));
        }
        Ok(())
    }

    // --- Phase 3: 全局参数表 ---

    pub fn save_global_params(&self, params: &crate::models::GlobalParams) -> Result<()> {
        let value = serde_json::to_string(params)?;
        self.conn.execute(
            "INSERT INTO global_params (key, value) VALUES ('default', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [value],
        )?;
        Ok(())
    }

    pub fn get_global_params(&self) -> Result<Option<crate::models::GlobalParams>> {
        let mut stmt = self.conn.prepare(
            "SELECT value FROM global_params WHERE key = 'default'"
        )?;
        let result = stmt.query_row([], |row| {
            let value: String = row.get(0)?;
            Ok(value)
        }).optional()?;
        match result {
            Some(value) => {
                let params = serde_json::from_str(&value)
                    .map_err(|e| AppError::Database(format!("全局参数解析失败: {}", e)))?;
                Ok(Some(params))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BusinessDomain, ContentModule, DocAttr, ProjectPhase, SecurityLayer};

    fn create_test_db() -> Database {
        Database::new(":memory:").unwrap()
    }

    fn sample_template() -> Template {
        Template {
            id: None,
            title: "等保2.0技术方案".to_string(),
            content: "这是一份技术方案内容".to_string(),
            content_html: Some("<p>这是一份技术方案内容</p>".to_string()),
            doc_attr: Some(DocAttr::TechnicalProposal),
            business_domain: Some(BusinessDomain::NetworkSecurity),
            security_layer: Some(SecurityLayer::Defense),
            content_module: Some(ContentModule::TechnicalProposal),
            project_phase: Some(ProjectPhase::Bidding),
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
        assert_eq!(fetched.doc_attr, Some(DocAttr::TechnicalProposal));
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
            .search_templates(&crate::models::TemplateFilter {
                doc_attr: Some(DocAttr::TechnicalProposal),
                limit: 10,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 1);

        let results = db
            .search_templates(&crate::models::TemplateFilter {
                doc_attr: Some(DocAttr::BidResponse),
                limit: 10,
                ..Default::default()
            })
            .unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_by_keyword() {
        let db = create_test_db();
        let mut tmpl = sample_template();
        db.insert_template(&mut tmpl).unwrap();

        let results = db
            .search_templates(&crate::models::TemplateFilter {
                keyword: Some("技术方案".to_string()),
                limit: 10,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 1);

        let results = db
            .search_templates(&crate::models::TemplateFilter {
                keyword: Some("不存在的关键词".to_string()),
                limit: 10,
                ..Default::default()
            })
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
            .search_templates(&crate::models::TemplateFilter {
                limit: 3,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_search_offset_pagination() {
        let db = create_test_db();
        for i in 0..5 {
            let mut tmpl = sample_template();
            tmpl.title = format!("方案{}", i);
            db.insert_template(&mut tmpl).unwrap();
        }
        // offset=0 时返回前 2 条
        let results = db
            .search_templates(&crate::models::TemplateFilter {
                limit: 2,
                offset: Some(0),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "方案0");
        assert_eq!(results[1].title, "方案1");

        // offset=2 时返回第 3、4 条
        let results = db
            .search_templates(&crate::models::TemplateFilter {
                limit: 2,
                offset: Some(2),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "方案2");
        assert_eq!(results[1].title, "方案3");

        // offset=4 时返回第 5 条
        let results = db
            .search_templates(&crate::models::TemplateFilter {
                limit: 2,
                offset: Some(4),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "方案4");

        // offset 超过总数时返回空
        let results = db
            .search_templates(&crate::models::TemplateFilter {
                limit: 2,
                offset: Some(10),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 0);
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
