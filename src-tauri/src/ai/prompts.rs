use crate::domain::Chapter;

pub(crate) fn build_batch_validation_prompt(chapters: &[Chapter]) -> String {
    let chapter_list = chapters
        .iter()
        .map(|chapter| format!("{}. {}", chapter.index, chapter.title))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"你是一位小说目录分析专家。请分析以下从TXT小说中识别出的章节目录，判断每个章节标题是否合理。

判断标准：
- "第X章"、"第X节"、"Chapter X"、"序章"、"楔子"等格式 → 有效，reason 填"有效"
- 纯数字如"12345" → 无效，reason 填"纯数字"
- 正文句子被误识别为标题（如"第九场赢了"夹在第25章和第26章之间）→ 无效，reason 填"正文误识别"
- 作者笔记、更新通知、勘误 → 无效，reason 填"作者笔记"
- 广告标题、推广内容 → 无效，reason 填"广告内容"
- 标题过长（超过30字）很可能是正文 → 无效，reason 填"标题过长"
- 其他无法归类的异常标题 → 无效，reason 填"格式异常"

请严格按顺序输出JSON数组，每个元素对应一个章节（序号从1开始）：
[
  {{"is_valid": true或false, "reason": "上述分类之一"}}
]

注意：reason 必须使用上述分类中的一个，不要自由发挥。只输出JSON，不要多余文字。

章节列表：
{}"#,
        chapter_list
    )
}

pub(crate) fn build_batch_review_prompt(chapters: &[Chapter]) -> String {
    let chapters_text = chapters
        .iter()
        .map(|chapter| {
            format!(
                "=== 章节 {} : {} ===\n{}",
                chapter.index,
                chapter.title,
                truncate_text(&chapter.original_text, 15000)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        r#"你是一位专业的小说编辑和校对员。请审查以下章节，修正错别字、删除无关内容（作者笔记、广告等）、修复语法问题。

重要规则：
1. 必须按章节序号顺序输出所有章节，不能遗漏
2. 即使章节不需要修改，也必须原样输出
3. 严格使用以下标记格式，标记之间不要添加额外文字：

<<<CHAPTER_START index=N>>>
修正后的章节正文
<<<CHAPTER_END index=N>>>

其中 N 是章节序号（如 1、2、3...），必须与输入的章节序号一致。

待审查章节：
{}"#,
        chapters_text
    )
}

pub(crate) fn truncate_text(text: &str, max_chars: usize) -> String {
    let result: String = text.chars().take(max_chars).collect();
    if result.chars().count() < text.chars().count() {
        format!("{}...", result)
    } else {
        result
    }
}
