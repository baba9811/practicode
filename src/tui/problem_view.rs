use super::localized_status;
use crate::core::{Problem, localized, normalize_ui_language, ui_text};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

pub(super) fn render(problem: &Problem, ui_language: &str, light: bool) -> Text<'static> {
    let lang = normalize_ui_language(ui_language);
    let title_style = if light {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    };
    let section_style = if light {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    };
    let body_style = if light {
        Style::default().fg(Color::Black)
    } else {
        Style::default().fg(Color::Rgb(229, 231, 235))
    };
    let meta_style = if light {
        Style::default().fg(Color::Rgb(75, 85, 99))
    } else {
        Style::default().fg(Color::Rgb(156, 163, 175))
    };
    let code_style = if light {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(229, 231, 235))
    } else {
        Style::default()
            .fg(Color::Rgb(243, 244, 246))
            .bg(Color::Rgb(31, 41, 55))
    };
    let number = problem
        .id
        .split_once('-')
        .map(|(number, _)| number)
        .unwrap_or(&problem.id);
    let mut lines = vec![
        Line::from(Span::styled(
            format!("{number}. {}", localized(&problem.title, &lang)),
            title_style,
        )),
        Line::from(Span::styled(
            format!(
                "{}: {}    {}: {}",
                ui_text(&lang, "difficulty"),
                localized_status(&lang, &problem.difficulty),
                ui_text(&lang, "topics"),
                problem.topics.join(", ")
            ),
            meta_style,
        )),
    ];
    lines.push(Line::default());
    for line in localized(&problem.statement, &lang).trim_end().lines() {
        lines.push(Line::from(Span::styled(line.to_string(), body_style)));
    }
    push_problem_section(
        &mut lines,
        ui_text(&lang, "input"),
        &localized(&problem.input, &lang),
        section_style,
        body_style,
    );
    push_problem_section(
        &mut lines,
        ui_text(&lang, "output"),
        &localized(&problem.output, &lang),
        section_style,
        body_style,
    );
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        ui_text(&lang, "examples").to_string(),
        section_style,
    )));
    for (index, case) in problem.examples.iter().enumerate() {
        lines.push(Line::from(Span::styled(
            format!("  {} {}", ui_text(&lang, "example"), index + 1),
            meta_style.add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!("    {}", ui_text(&lang, "input")),
            meta_style,
        )));
        push_code_lines(&mut lines, &case.input, code_style, &lang);
        lines.push(Line::from(Span::styled(
            format!("    {}", ui_text(&lang, "output")),
            meta_style,
        )));
        push_code_lines(&mut lines, &case.output, code_style, &lang);
    }
    Text::from(lines)
}

fn push_problem_section(
    lines: &mut Vec<Line<'static>>,
    title: &str,
    body: &str,
    section_style: Style,
    body_style: Style,
) {
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(title.to_string(), section_style)));
    for line in body.trim_end().lines() {
        lines.push(Line::from(Span::styled(format!("  {line}"), body_style)));
    }
}

fn push_code_lines(
    lines: &mut Vec<Line<'static>>,
    body: &str,
    code_style: Style,
    ui_language: &str,
) {
    let body = body.trim_end();
    if body.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("      "),
            Span::styled(ui_text(ui_language, "empty_value").to_string(), code_style),
        ]));
        return;
    }
    for line in body.lines() {
        lines.push(Line::from(vec![
            Span::raw("      "),
            Span::styled(line.to_string(), code_style),
        ]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tui_problem_view_localizes_empty_example_values() {
        let mut problem = crate::core::starter_problem();
        problem.examples[0].input.clear();
        problem.examples[0].output.clear();

        let text = render(&problem, "ko", false);
        let rendered = text
            .lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert!(rendered.contains("<비어 있음>"), "{rendered}");
        assert!(!rendered.contains("<empty>"), "{rendered}");
    }

    #[test]
    fn tui_problem_view_localizes_difficulty_tokens() {
        for language in ["ko", "ja", "zh", "es"] {
            let problem = crate::core::starter_problem();
            let text = render(&problem, language, false);
            let rendered = text
                .lines
                .iter()
                .flat_map(|line| line.spans.iter())
                .map(|span| span.content.as_ref())
                .collect::<String>();

            assert!(
                rendered.contains(ui_text(language, "status_easy")),
                "{language}: {rendered}"
            );
            assert!(!rendered.contains(": easy"), "{language}: {rendered}");
        }
    }
}
