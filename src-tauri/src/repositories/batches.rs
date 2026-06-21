use crate::domain::ChapterBatch;
use rusqlite::{params, Connection};

pub(crate) fn list_batches(conn: &Connection, novel_id: &str) -> Result<Vec<ChapterBatch>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, novel_id, batch_index, label, start_chapter, end_chapter, status, created_at
             FROM chapter_batches WHERE novel_id = ?1 ORDER BY batch_index",
        )
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(params![novel_id], |row| {
            Ok(ChapterBatch {
                id: row.get(0)?,
                novel_id: row.get(1)?,
                batch_index: row.get(2)?,
                label: row.get(3)?,
                start_chapter: row.get(4)?,
                end_chapter: row.get(5)?,
                status: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
}

pub(crate) fn get_batch(conn: &Connection, batch_id: &str) -> Result<ChapterBatch, String> {
    conn.query_row(
        "SELECT id, novel_id, batch_index, label, start_chapter, end_chapter, status, created_at
         FROM chapter_batches WHERE id = ?1",
        params![batch_id],
        |row| {
            Ok(ChapterBatch {
                id: row.get(0)?,
                novel_id: row.get(1)?,
                batch_index: row.get(2)?,
                label: row.get(3)?,
                start_chapter: row.get(4)?,
                end_chapter: row.get(5)?,
                status: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn insert_batch(conn: &Connection, batch: &ChapterBatch) -> Result<(), String> {
    conn.execute(
        "INSERT INTO chapter_batches (id, novel_id, batch_index, label, start_chapter, end_chapter, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            batch.id,
            batch.novel_id,
            batch.batch_index,
            batch.label,
            batch.start_chapter,
            batch.end_chapter,
            batch.status,
            batch.created_at,
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn update_batch_status(
    conn: &Connection,
    batch_id: &str,
    status: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE chapter_batches SET status = ?1 WHERE id = ?2",
        params![status, batch_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn delete_batches_by_novel(conn: &Connection, novel_id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM chapter_batches WHERE novel_id = ?1",
        params![novel_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}
