use crate::domain::Job;
use rusqlite::Connection;

pub(crate) fn update_job_progress(
    conn: &Connection,
    job_id: &str,
    current_chapter: i64,
    message: &str,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE jobs SET current_chapter = ?1, message = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![current_chapter, message, now, job_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn complete_job(conn: &Connection, job_id: &str, message: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE jobs SET status = 'completed', message = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![message, now, job_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn fail_job(conn: &Connection, job_id: &str, message: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE jobs SET status = 'failed', message = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![message, now, job_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn get_job(conn: &Connection, job_id: &str) -> Result<Job, String> {
    conn.query_row(
        "SELECT id, novel_id, job_type, status, current_chapter, total_chapters, message, created_at, updated_at
         FROM jobs WHERE id = ?1",
        rusqlite::params![job_id],
        |row| {
            Ok(Job {
                id: row.get(0)?,
                novel_id: row.get(1)?,
                job_type: row.get(2)?,
                status: row.get(3)?,
                current_chapter: row.get(4)?,
                total_chapters: row.get(5)?,
                message: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    )
    .map_err(|error| error.to_string())
}
