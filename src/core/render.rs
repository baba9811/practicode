use super::*;

pub fn localized(map: &HashMap<String, String>, lang: &str) -> String {
    let lang = normalize_ui_language(lang);
    map.get(lang.as_str())
        .or_else(|| map.get("en"))
        .or_else(|| map.get("ko"))
        .or_else(|| map.values().next())
        .cloned()
        .unwrap_or_default()
}
pub fn render_problem(problem: &Problem, ui_language: &str) -> String {
    let lang = normalize_ui_language(ui_language);
    let examples = problem
        .examples
        .iter()
        .enumerate()
        .map(|(index, case)| {
            format!(
                "### {} {}\n\n{}\n\n{}\n\n{}\n\n{}",
                ui_text(&lang, "example"),
                index + 1,
                ui_text(&lang, "input"),
                fenced_text(&case.input),
                ui_text(&lang, "output"),
                fenced_text(&case.output)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let number = problem
        .id
        .split_once('-')
        .map(|(number, _)| number)
        .unwrap_or(&problem.id);
    format!(
        "# {number}. {}\n\n{}: {}\n{}: {}\n\n{}\n\n## {}\n\n{}\n\n## {}\n\n{}\n\n## {}\n\n{}",
        localized(&problem.title, &lang),
        ui_text(&lang, "difficulty"),
        problem.difficulty,
        ui_text(&lang, "topics"),
        problem.topics.join(", "),
        localized(&problem.statement, &lang),
        ui_text(&lang, "input"),
        localized(&problem.input, &lang),
        ui_text(&lang, "output"),
        localized(&problem.output, &lang),
        ui_text(&lang, "examples"),
        examples
    )
}

pub fn render_problem_tui(problem: &Problem, ui_language: &str) -> String {
    let lang = normalize_ui_language(ui_language);
    let number = problem
        .id
        .split_once('-')
        .map(|(number, _)| number)
        .unwrap_or(&problem.id);
    let mut lines = vec![
        format!("{number}. {}", localized(&problem.title, &lang)),
        format!(
            "{}: {}    {}: {}",
            ui_text(&lang, "difficulty"),
            problem.difficulty,
            ui_text(&lang, "topics"),
            problem.topics.join(", ")
        ),
        String::new(),
        localized(&problem.statement, &lang),
    ];
    push_tui_section(
        &mut lines,
        ui_text(&lang, "input"),
        &localized(&problem.input, &lang),
    );
    push_tui_section(
        &mut lines,
        ui_text(&lang, "output"),
        &localized(&problem.output, &lang),
    );
    lines.push(String::new());
    lines.push(ui_text(&lang, "examples").to_string());
    for (index, case) in problem.examples.iter().enumerate() {
        lines.push(format!("  {} {}", ui_text(&lang, "example"), index + 1));
        lines.push(format!("    {}:", ui_text(&lang, "input")));
        push_case_text(&mut lines, &case.input, &lang);
        lines.push(format!("    {}:", ui_text(&lang, "output")));
        push_case_text(&mut lines, &case.output, &lang);
    }
    lines.join("\n").trim_end().to_string()
}

fn push_tui_section(lines: &mut Vec<String>, title: &str, body: &str) {
    lines.push(String::new());
    lines.push(title.to_string());
    for line in body.trim_end().lines() {
        lines.push(format!("  {line}"));
    }
}

fn push_case_text(lines: &mut Vec<String>, body: &str, ui_language: &str) {
    let body = body.trim_end();
    if body.is_empty() {
        lines.push(format!("      {}", ui_text(ui_language, "empty_value")));
    } else {
        for line in body.lines() {
            lines.push(format!("      {line}"));
        }
    }
}

pub fn fenced_text(value: &str) -> String {
    let mut body = value.to_string();
    if !body.ends_with('\n') {
        body.push('\n');
    }
    format!("```text\n{body}```")
}
