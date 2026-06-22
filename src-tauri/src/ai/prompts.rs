use crate::domain::Chapter;

pub(crate) fn build_batch_validation_prompt(chapters: &[Chapter]) -> String {
    let chapter_list = chapters
        .iter()
        .map(|chapter| format!("{}. {}", chapter.index, chapter.title))
        .collect::<Vec<_>>()
        .join("\n");

    let format_hint = detect_dominant_format(chapters);

    format!(
        r#"你是一位专业的小说内容分析师。以下是从小说中通过正则识别出的章节目录列表。请判断哪些是有效的章节标题，哪些不是。

有效的章节标题：实际的小说章节（如"第1章 xxx"、"Chapter 1 xxx"、"序章"、"楔子"等）
无效的内容：作者笔记标题、更新通知、广告、目录页标题、装饰性文字、乱码等

## 判断规则

### 规则1：正误判为章节的正文内容必须删除
当前一章是第X章，后一章是第X+1章，中间夹着的内容如果看起来像正文句子（不是章节标题格式），则判定为无效。
例如：前一章是第25章，中间出现"第九场获胜了的第九场"，后面是第26章，这个"第九场获胜了的第九场"是正文内容被错判为章节，应删除。

### 规则2：章节编号连续性检查
检查整个列表的章节编号是否连续。如果出现编号跳跃或重复，则说明编号可能有错误：
- 前一章索引是26，这一章索引还是26，下一章是28，这一章应该标记为编号错误
- 编号跳跃超过1的章节需要重点关注
- 对于编号错误的章节，reason中要说明正确的编号应该是什么

### 规则3：章节标题风格一致性检查
{format_hint}

### 规则4：明显非章节标题的内容
以下内容即使被正则识别也不应作为章节：
- 纯数字标题（如"12345"）
- 纯标点符号
- 过长的标题（超过50个字很可能是正文句子）
- 包含明显正文词汇的标题（如包含逗号、句号等标点的长句子）

## 输出要求

请输出JSON数组，按顺序对应每个输入的章节：
[
  {{"chapter_id": "章节ID", "is_valid": true/false, "reason": "简要说明", "suggested_index": null或建议的正确编号}},
  ...
]

重要：
- 输出必须是纯JSON数组
- 按输入顺序排列
- 每个元素必须包含chapter_id、is_valid、reason三个字段
- 如果编号错误，suggested_index填写建议的正确编号；否则为null

章节目录：
{}"#,
        chapter_list
    )
}

fn detect_dominant_format(chapters: &[Chapter]) -> String {
    let mut format_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for chapter in chapters {
        let title = &chapter.title;
        let format = if title.contains("章") {
            "DYNAMIC_CHAPTER"
        } else if title.contains("回") {
            "DYNAMIC_HUI"
        } else if title.contains("节") {
            "DYNAMIC_JIE"
        } else if title.contains("卷") {
            "DYNAMIC_JUAN"
        } else if title.contains("Chapter") || title.contains("chapter") {
            "DYNAMIC_CHAPTER_EN"
        } else {
            "OTHER"
        };
        *format_counts.entry(format.to_string()).or_insert(0) += 1;
    }

    if let Some((dominant, count)) = format_counts.iter().max_by_key(|(_, c)| **c) {
        let total: usize = format_counts.values().sum();
        if *count > total * 8 / 10 {
            let dominant_label = match dominant.as_str() {
                "DYNAMIC_CHAPTER" => "第xx章",
                "DYNAMIC_HUI" => "第xx回",
                "DYNAMIC_JIE" => "第xx节",
                "DYNAMIC_JUAN" => "第xx卷",
                "DYNAMIC_CHAPTER_EN" => "Chapter X",
                _ => "其他",
            };
            return format!(
                "本书绝大多数章节使用{}格式（共{}章中有{}章），如果出现其他格式的标题（如第xx回出现在全是第xx章的书中），需要重点检查是否为错误标题或识别错误。",
                dominant_label, total, count
            );
        }
    }

    "请根据上下文判断章节标题是否符合本书的整体风格。".to_string()
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
