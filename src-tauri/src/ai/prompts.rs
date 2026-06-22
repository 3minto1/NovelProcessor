use crate::domain::Chapter;

pub(crate) fn build_batch_validation_prompt(chapters: &[Chapter]) -> String {
    let chapter_list = chapters
        .iter()
        .map(|chapter| format!("{}. {}", chapter.index, chapter.title))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"你是一位专业的小说内容分析师。以下是从小说中通过正则识别出的章节目录列表。请判断哪些是有效的章节标题，哪些不是。

有效的章节标题：实际的小说章节（如"第1章 xxx"、"Chapter 1 xxx"、"序章"、"楔子"等）
无效的内容：作者笔记标题、更新通知、广告、目录页标题、装饰性文字、乱码等

请输出JSON数组，按顺序对应每个输入的章节：
[
  {{"chapter_id": "章节ID", "is_valid": true/false, "reason": "简要说明"}},
  ...
]

重要：
- 输出必须是纯JSON数组
- 按输入顺序排列
- 每个元素必须包含chapter_id、is_valid、reason三个字段

章节目录：
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
        r#"你是一位专业的小说编辑和校对员。请审查以下批次的小说章节，检查以下问题：

1. **错别字和拼写错误** - 修正任何错误的中文汉字、错词或拼写错误
2. **无关内容** - 识别并删除不属于小说正文的内容：
   - 混入正文的作者笔记/作者有话说
   - 更新公告（如"今天还有一更"、"求月票"、"求推荐票"、"求收藏"）
   - 广告或推广内容
   - 装饰性分隔线（===, ---, ~~~, *****）
   - 版权声明或免责声明
   - 网站URL或水印
   - 乱码或无意义字符
3. **语法问题** - 修正不通顺的句子、语法错误、标点符号错误
4. **格式问题** - 修复断行、缺失标点或乱码文本

对于每一章，请输出修正后的完整章节文本。

输出格式（每章一个区块）：
<<<CHAPTER_START index=N>>>
修正后的章节正文
<<<CHAPTER_END index=N>>>

待审查章节：
{}"#,
        chapters_text
    )
}

pub(crate) fn truncate_text(text: &str, max_chars: usize) -> String {
    let chars: Vec<char> = text.chars().take(max_chars).collect();
    let result: String = chars.into_iter().collect();
    if result.len() < text.len() {
        format!("{}...", result)
    } else {
        result
    }
}
