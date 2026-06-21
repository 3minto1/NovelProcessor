use rusqlite::Connection;

pub(crate) fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS novels (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            source_path TEXT NOT NULL,
            encoding TEXT NOT NULL,
            status TEXT NOT NULL,
            detected_chapters INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS chapters (
            id TEXT PRIMARY KEY,
            novel_id TEXT NOT NULL,
            chapter_index INTEGER NOT NULL,
            title TEXT NOT NULL,
            original_text TEXT NOT NULL,
            is_valid INTEGER NOT NULL DEFAULT 1,
            validation_reason TEXT,
            corrected_text TEXT,
            validation_status TEXT NOT NULL DEFAULT 'pending',
            review_status TEXT NOT NULL DEFAULT 'pending',
            FOREIGN KEY(novel_id) REFERENCES novels(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS chapter_batches (
            id TEXT PRIMARY KEY,
            novel_id TEXT NOT NULL,
            batch_index INTEGER NOT NULL,
            label TEXT NOT NULL,
            start_chapter INTEGER NOT NULL,
            end_chapter INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL,
            FOREIGN KEY(novel_id) REFERENCES novels(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS model_profiles (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            provider TEXT NOT NULL,
            base_url TEXT NOT NULL,
            model TEXT NOT NULL,
            temperature REAL NOT NULL,
            top_p REAL NOT NULL DEFAULT 1.0,
            thinking_mode TEXT NOT NULL DEFAULT 'auto',
            api_key TEXT,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS jobs (
            id TEXT PRIMARY KEY,
            novel_id TEXT NOT NULL,
            job_type TEXT NOT NULL,
            status TEXT NOT NULL,
            current_chapter INTEGER NOT NULL,
            total_chapters INTEGER NOT NULL,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_logs (
            id TEXT PRIMARY KEY,
            novel_id TEXT,
            profile_id TEXT NOT NULL,
            action TEXT NOT NULL,
            chapter_title TEXT,
            status TEXT NOT NULL,
            content TEXT NOT NULL,
            reasoning TEXT,
            raw_response TEXT,
            input_tokens INTEGER,
            output_tokens INTEGER,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_chapters_novel ON chapters(novel_id, chapter_index);
        CREATE INDEX IF NOT EXISTS idx_jobs_novel ON jobs(novel_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_ai_logs_created ON ai_logs(created_at);
        CREATE INDEX IF NOT EXISTS idx_chapter_batches_novel ON chapter_batches(novel_id, batch_index);
        "#,
    )?;
    Ok(())
}
