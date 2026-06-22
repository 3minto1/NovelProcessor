use crate::domain::Chapter;
use rusqlite::{params, Connection};

pub(crate) fn list_chapters(conn: &Connection, novel_id: &str) -> Result<Vec<Chapter>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, novel_id, chapter_index, title, original_text, is_valid,
                    validation_reason, corrected_text, validation_status, review_status
             FROM chapters WHERE novel_id = ?1 ORDER BY chapter_index",
        )
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(params![novel_id], |row| {
            Ok(Chapter {
                id: row.get(0)?,
                novel_id: row.get(1)?,
                index: row.get(2)?,
                title: row.get(3)?,
                original_text: row.get(4)?,
                is_valid: row.get::<_, i64>(5)? != 0,
                validation_reason: row.get(6)?,
                corrected_text: row.get(7)?,
                validation_status: row.get(8)?,
                review_status: row.get(9)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
}

pub(crate) fn get_chapter(conn: &Connection, chapter_id: &str) -> Result<Chapter, String> {
    conn.query_row(
        "SELECT id, novel_id, chapter_index, title, original_text, is_valid,
                validation_reason, corrected_text, validation_status, review_status
         FROM chapters WHERE id = ?1",
        params![chapter_id],
        |row| {
            Ok(Chapter {
                id: row.get(0)?,
                novel_id: row.get(1)?,
                index: row.get(2)?,
                title: row.get(3)?,
                original_text: row.get(4)?,
                is_valid: row.get::<_, i64>(5)? != 0,
                validation_reason: row.get(6)?,
                corrected_text: row.get(7)?,
                validation_status: row.get(8)?,
                review_status: row.get(9)?,
            })
        },
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn insert_chapter(conn: &Connection, chapter: &Chapter) -> Result<(), String> {
    conn.execute(
        "INSERT INTO chapters (id, novel_id, chapter_index, title, original_text, is_valid,
                              validation_reason, corrected_text, validation_status, review_status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            chapter.id,
            chapter.novel_id,
            chapter.index,
            chapter.title,
            chapter.original_text,
            chapter.is_valid as i64,
            chapter.validation_reason,
            chapter.corrected_text,
            chapter.validation_status,
            chapter.review_status,
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn update_chapter_validation(
    conn: &Connection,
    chapter_id: &str,
    is_valid: bool,
    reason: Option<&str>,
    status: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE chapters SET is_valid = ?1, validation_reason = ?2, validation_status = ?3
         WHERE id = ?4",
        params![is_valid as i64, reason, status, chapter_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn update_chapter_review(
    conn: &Connection,
    chapter_id: &str,
    corrected_text: &str,
    status: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE chapters SET corrected_text = ?1, review_status = ?2
         WHERE id = ?3",
        params![corrected_text, status, chapter_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn delete_chapters_by_novel(conn: &Connection, novel_id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM chapters WHERE novel_id = ?1", params![novel_id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn update_chapter_text(
    conn: &Connection,
    chapter_id: &str,
    title: &str,
    original_text: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE chapters SET title = ?1, original_text = ?2
         WHERE id = ?3",
        params![title, original_text, chapter_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn delete_chapter(conn: &Connection, chapter_id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM chapters WHERE id = ?1", params![chapter_id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn toggle_chapter_validity(
    conn: &Connection,
    chapter_id: &str,
    is_valid: bool,
) -> Result<(), String> {
    conn.execute(
        "UPDATE chapters SET is_valid = ?1 WHERE id = ?2",
        params![is_valid as i64, chapter_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}
