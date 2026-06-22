use crate::domain::Chapter;

pub(crate) fn build_batch_validation_prompt(chapters: &[Chapter]) -> String {
    let chapter_list = chapters
        .iter()
        .map(|chapter| format!("{}. {}", chapter.index, chapter.title))
        .collect::<Vec<_>>()
        .join("\n");

    let format_hint = detect_dominant_format(chapters);
    let non_standard = detect_non_standard_chapters(chapters);

    format!(
        r#"你是一位专业的小说内容分析师和校对专家。以下是从小说中通过正则识别出的章节目录列表。请逐条检查并列出所有异常。

注意：你只需要标记哪些章节有问题，不需要删除任何内容。程序会保留所有章节，只是将有问题的章节标记出来供用户查看和手动处理。

## 一、编号问题（重点检查）

1. **重复编号**：如两章都是"第25章"
2. **跳号/缺章**：如第25章后直接是第27章，缺少第26章
3. **顺序错乱**：如第20章后出现第18章
4. **编号写错**：如"第一百张"（应为章）、"第一零章"、"第一另章"等明显错误的编号

## 二、标题问题

1. **标题错别字**：标题中有明显的错别字
2. **打错字/用词异常**：明显是打字错误
3. **风格不一致**：与大多数章节标题风格明显不同
4. **过于异常的标题**：
   - 只有符号
   - 极短且不像章节名
   - 重复标题（同名章节）
5. ※ 注意："只有编号没有标题"属于正常情况，不算错误

## 三、非章节内容识别

找出所有混入章节序列中的非章节内容：
- 请假条
- 上架感言
- 完结感言
- 更新说明
- 作者状态记录（生病/停电/有事等）
- 幕间说明类文字
- 生活记录类插入内容
- 广告或推广内容
- 作者与读者互动

并标注它们夹在哪两章之间（如"夹在第5章和第6章之间"）

## 四、结构问题

1. **卷内编号中断**：某一卷内编号突然中断
2. **明显断层**：章节之间有明显的内容断裂
3. **章节拆分**：一章被拆成两条
4. **内容插入**：有内容插入导致阅读顺序异常

## 五、非标准编号格式识别

找出所有偏离主体编号规则的条目：
- 特殊章节标记：最终章/终章/尾声/序章/楔子等
- 番外（编号格式与正文不同）
- 间章/幕间等
- 其他任何偏离主体编号规则的章节条目

标注其位置（在哪一章之后）及与主体格式的差异。

## 六、卷信息

单独列出所有卷信息，标明是第几卷以及包含哪些章节。
例如：卷一（第1-30章）、卷二（第31-60章）

{format_hint}

{non_standard}

## 输出要求

请输出JSON对象，包含以下字段：
{{
  "valid_chapters": ["有效章节的ID列表"],
  "invalid_chapters": [
    {{"chapter_id": "ID", "reason": "问题描述", "category": "编号错误/标题异常/非章节内容/结构问题/非标准格式"}}
  ],
  "structure_issues": ["结构问题描述"],
  "non_standard_chapters": ["非标准格式章节的描述"],
  "volume_info": ["卷信息，如：卷一（第1-30章）"],
  "between_content": ["非章节内容位置，如：上架感言夹在第5章和第6章之间"],
  "suggested_corrections": [
    {{"chapter_id": "ID", "suggested_index": 建议编号或null, "suggested_title": "建议标题或null"}}
  ]
}}

重要：
- 输出必须是纯JSON对象
- invalid_chapters列出所有有问题的章节
- suggested_corrections列出编号或标题需要修正的章节
- is_valid为false表示该章节被标记为无效

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
                "本书绝大多数章节使用{}格式（共{}章中有{}章），如果出现其他格式的标题，需要重点检查是否为错误标题或识别错误。",
                dominant_label, total, count
            );
        }
    }

    "请根据上下文判断章节标题是否符合本书的整体风格。".to_string()
}

fn detect_non_standard_chapters(chapters: &[Chapter]) -> String {
    let mut non_standard = Vec::new();
    let special_keywords = [
        "序章", "楔子", "引子", "引言", "序言", "序幕", "前言",
        "终章", "尾声", "后记", "番外", "特别篇", "外传", "插曲", "间章",
        "简介", "文案", "作品相关", "上架感言", "完本感言",
        "最终章", "结局", "完结", "番外篇", "番外章",
    ];

    for (i, chapter) in chapters.iter().enumerate() {
        let title = &chapter.title;
        let is_special = special_keywords.iter().any(|kw| title.contains(kw));
        let is_volume = title.contains("卷");

        if is_special || is_volume {
            let position = if i > 0 {
                format!("在第{}章之后", chapters[i - 1].index)
            } else {
                "在最前面".to_string()
            };
            let category = if is_volume { "卷" } else { "特殊章节" };
            non_standard.push(format!("第{}章 \"{}\" - 类型：{}，位置：{}", chapter.index, title, category, position));
        }
    }

    if non_standard.is_empty() {
        String::new()
    } else {
        format!("非标准章节：\n{}", non_standard.join("\n"))
    }
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
