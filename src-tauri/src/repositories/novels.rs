use crate::domain::{Novel, NovelDetail};
use rusqlite::{params, Connection};

pub(crate) fn list_novels(conn: &Connection) -> Result<Vec<Novel>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, source_path, encoding, status, detected_chapters, created_at
             FROM novels ORDER BY created_at DESC",
        )
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Novel {
                id: row.get(0)?,
                title: row.get(1)?,
                source_path: row.get(2)?,
                encoding: row.get(3)?,
                status: row.get(4)?,
                detected_chapters: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
}

pub(crate) fn get_novel(conn: &Connection, novel_id: &str) -> Result<Novel, String> {
    conn.query_row(
        "SELECT id, title, source_path, encoding, status, detected_chapters, created_at
         FROM novels WHERE id = ?1",
        params![novel_id],
        |row| {
            Ok(Novel {
                id: row.get(0)?,
                title: row.get(1)?,
                source_path: row.get(2)?,
                encoding: row.get(3)?,
                status: row.get(4)?,
                detected_chapters: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
            })
        },
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn get_novel_detail(conn: &Connection, novel_id: &str) -> Result<NovelDetail, String> {
    let novel = get_novel(conn, novel_id)?;
    let chapters = crate::repositories::chapters::list_chapters(conn, novel_id)?;
    let batches = crate::repositories::batches::list_batches(conn, novel_id)?;
    Ok(NovelDetail {
        novel,
        chapters,
        batches,
    })
}

pub(crate) fn insert_novel(conn: &Connection, novel: &Novel) -> Result<(), String> {
    conn.execute(
        "INSERT INTO novels (id, title, source_path, encoding, status, detected_chapters, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            novel.id,
            novel.title,
            novel.source_path,
            novel.encoding,
            novel.status,
            novel.detected_chapters as i64,
            novel.created_at,
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn delete_novel(conn: &Connection, novel_id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM novels WHERE id = ?1", params![novel_id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn update_novel_status(
    conn: &Connection,
    novel_id: &str,
    status: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE novels SET status = ?1 WHERE id = ?2",
        params![status, novel_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}
