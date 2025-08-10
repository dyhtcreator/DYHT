use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Uuid,
    pub content: String,
    pub embedding: Vec<f32>,
    pub timestamp: DateTime<Utc>,
    pub entry_type: MemoryEntryType,
    pub metadata: serde_json::Value,
    pub relevance_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryEntryType {
    Conversation,
    AudioTranscription,
    UserPreference,
    SystemEvent,
    Knowledge,
}

#[derive(Debug)]
pub struct MemoryManager {
    pub db_pool: PgPool,
    pub embedding_dimension: usize,
}

impl MemoryManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        let db_pool = PgPool::connect(database_url).await?;
        
        // Run database migrations to ensure tables exist
        Self::setup_database(&db_pool).await?;
        
        Ok(Self {
            db_pool,
            embedding_dimension: 384, // Standard dimension for sentence transformers
        })
    }

    async fn setup_database(pool: &PgPool) -> Result<()> {
        // Enable pgvector extension
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(pool)
            .await?;

        // Create memories table with vector support
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS memories (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                content TEXT NOT NULL,
                embedding vector(384) NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                entry_type VARCHAR(50) NOT NULL,
                metadata JSONB NOT NULL DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#)
        .execute(pool)
        .await?;

        // Create index for vector similarity search
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS memories_embedding_idx 
            ON memories USING ivfflat (embedding vector_cosine_ops) 
            WITH (lists = 100)
        "#)
        .execute(pool)
        .await?;

        // Create index for timestamp-based queries
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS memories_timestamp_idx 
            ON memories (timestamp DESC)
        "#)
        .execute(pool)
        .await?;

        // Create index for entry type filtering
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS memories_entry_type_idx 
            ON memories (entry_type)
        "#)
        .execute(pool)
        .await?;

        log::info!("Database setup completed successfully");
        Ok(())
    }

    pub async fn store_memory(
        &self,
        content: String,
        entry_type: MemoryEntryType,
        metadata: Option<serde_json::Value>,
    ) -> Result<Uuid> {
        // TODO: Generate embedding using a proper embedding model
        // For now, create a placeholder embedding
        let embedding = self.generate_placeholder_embedding(&content);
        
        let id = Uuid::new_v4();
        let metadata = metadata.unwrap_or_else(|| serde_json::json!({}));
        
        sqlx::query(r#"
            INSERT INTO memories (id, content, embedding, entry_type, metadata)
            VALUES ($1, $2, $3, $4, $5)
        "#)
        .bind(&id)
        .bind(&content)
        .bind(&embedding)
        .bind(format!("{:?}", entry_type))
        .bind(&metadata)
        .execute(&self.db_pool)
        .await?;

        log::info!("Stored memory entry: {} (type: {:?})", id, entry_type);
        Ok(id)
    }

    pub async fn retrieve_context(&self, query: &str) -> Result<Vec<String>> {
        // TODO: Generate embedding for query using the same model as storage
        let query_embedding = self.generate_placeholder_embedding(query);
        
        // Retrieve similar memories using cosine similarity
        let rows = sqlx::query(r#"
            SELECT content, embedding <=> $1 as distance
            FROM memories
            WHERE embedding <=> $1 < 0.5
            ORDER BY embedding <=> $1
            LIMIT 10
        "#)
        .bind(&query_embedding)
        .fetch_all(&self.db_pool)
        .await?;

        let context: Vec<String> = rows
            .iter()
            .map(|row| row.get::<String, _>("content"))
            .collect();

        log::info!("Retrieved {} context entries for query", context.len());
        Ok(context)
    }

    pub async fn retrieve_memories_by_type(
        &self,
        entry_type: MemoryEntryType,
        limit: Option<i64>,
    ) -> Result<Vec<MemoryEntry>> {
        let limit = limit.unwrap_or(50);
        
        let rows = sqlx::query(r#"
            SELECT id, content, embedding, timestamp, entry_type, metadata
            FROM memories
            WHERE entry_type = $1
            ORDER BY timestamp DESC
            LIMIT $2
        "#)
        .bind(format!("{:?}", entry_type))
        .bind(limit)
        .fetch_all(&self.db_pool)
        .await?;

        let memories: Result<Vec<MemoryEntry>> = rows
            .iter()
            .map(|row| {
                Ok(MemoryEntry {
                    id: row.get("id"),
                    content: row.get("content"),
                    embedding: row.get::<Vec<f32>, _>("embedding"),
                    timestamp: row.get("timestamp"),
                    entry_type: self.parse_entry_type(&row.get::<String, _>("entry_type"))?,
                    metadata: row.get("metadata"),
                    relevance_score: None,
                })
            })
            .collect();

        memories
    }

    pub async fn search_memories(
        &self,
        query: &str,
        entry_types: Option<Vec<MemoryEntryType>>,
        limit: Option<i64>,
    ) -> Result<Vec<MemoryEntry>> {
        let query_embedding = self.generate_placeholder_embedding(query);
        let limit = limit.unwrap_or(20);

        let type_filter = if let Some(types) = entry_types {
            let type_strings: Vec<String> = types.iter().map(|t| format!("{:?}", t)).collect();
            format!("AND entry_type = ANY('{{{}}}')", type_strings.join(","))
        } else {
            String::new()
        };

        let sql = format!(r#"
            SELECT id, content, embedding, timestamp, entry_type, metadata,
                   embedding <=> $1 as relevance_score
            FROM memories
            WHERE embedding <=> $1 < 0.7
            {}
            ORDER BY embedding <=> $1
            LIMIT $2
        "#, type_filter);

        let rows = sqlx::query(&sql)
            .bind(&query_embedding)
            .bind(limit)
            .fetch_all(&self.db_pool)
            .await?;

        let memories: Result<Vec<MemoryEntry>> = rows
            .iter()
            .map(|row| {
                Ok(MemoryEntry {
                    id: row.get("id"),
                    content: row.get("content"),
                    embedding: row.get::<Vec<f32>, _>("embedding"),
                    timestamp: row.get("timestamp"),
                    entry_type: self.parse_entry_type(&row.get::<String, _>("entry_type"))?,
                    metadata: row.get("metadata"),
                    relevance_score: Some(row.get("relevance_score")),
                })
            })
            .collect();

        memories
    }

    pub async fn update_memory(&self, id: Uuid, content: String) -> Result<()> {
        let embedding = self.generate_placeholder_embedding(&content);
        
        sqlx::query(r#"
            UPDATE memories
            SET content = $1, embedding = $2, updated_at = NOW()
            WHERE id = $3
        "#)
        .bind(&content)
        .bind(&embedding)
        .bind(&id)
        .execute(&self.db_pool)
        .await?;

        log::info!("Updated memory entry: {}", id);
        Ok(())
    }

    pub async fn delete_memory(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM memories WHERE id = $1")
            .bind(&id)
            .execute(&self.db_pool)
            .await?;

        log::info!("Deleted memory entry: {}", id);
        Ok(())
    }

    pub async fn cleanup_old_memories(&self, days_old: i64) -> Result<u64> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days_old);
        
        let result = sqlx::query(r#"
            DELETE FROM memories
            WHERE timestamp < $1
            AND entry_type NOT IN ('UserPreference', 'Knowledge')
        "#)
        .bind(cutoff_date)
        .execute(&self.db_pool)
        .await?;

        log::info!("Cleaned up {} old memory entries", result.rows_affected());
        Ok(result.rows_affected())
    }

    pub async fn get_memory_stats(&self) -> Result<serde_json::Value> {
        let rows = sqlx::query(r#"
            SELECT 
                entry_type,
                COUNT(*) as count,
                MIN(timestamp) as oldest,
                MAX(timestamp) as newest
            FROM memories
            GROUP BY entry_type
        "#)
        .fetch_all(&self.db_pool)
        .await?;

        let stats: serde_json::Value = rows
            .iter()
            .map(|row| {
                (
                    row.get::<String, _>("entry_type"),
                    serde_json::json!({
                        "count": row.get::<i64, _>("count"),
                        "oldest": row.get::<DateTime<Utc>, _>("oldest"),
                        "newest": row.get::<DateTime<Utc>, _>("newest")
                    })
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into();

        Ok(stats)
    }

    fn generate_placeholder_embedding(&self, text: &str) -> Vec<f32> {
        // TODO: Replace with actual embedding generation using sentence transformers
        // For now, generate a simple hash-based embedding
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Convert hash to embedding-like vector
        (0..self.embedding_dimension)
            .map(|i| {
                let shifted_hash = hash.wrapping_add(i as u64);
                ((shifted_hash % 1000) as f32 - 500.0) / 500.0
            })
            .collect()
    }

    fn parse_entry_type(&self, type_str: &str) -> Result<MemoryEntryType> {
        match type_str {
            "Conversation" => Ok(MemoryEntryType::Conversation),
            "AudioTranscription" => Ok(MemoryEntryType::AudioTranscription),
            "UserPreference" => Ok(MemoryEntryType::UserPreference),
            "SystemEvent" => Ok(MemoryEntryType::SystemEvent),
            "Knowledge" => Ok(MemoryEntryType::Knowledge),
            _ => Err(anyhow::anyhow!("Unknown memory entry type: {}", type_str)),
        }
    }
}