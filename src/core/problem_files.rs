use super::*;

pub fn ensure_problem_files(root: &Path, problem: &Problem) -> Result<()> {
    let problem_dir = root.join("problems").join(&problem.id);
    create_dir_all_beneath(root, &problem_dir)?;
    let readme = problem_dir.join("README.md");
    if regular_file_exists(&readme)? {
        return Ok(());
    }
    let examples = problem
        .examples
        .iter()
        .map(|case| format!("input:\n{}output:\n{}", case.input, case.output))
        .collect::<Vec<_>>()
        .join("\n");
    save_user_text(
        &readme,
        &format!(
            "# {}. {}\n\n난이도: {}\n\n{}\n\n## 입력\n\n{}\n\n## 출력\n\n{}\n\n## 예시\n\n```text\n{}\n```\n",
            problem.id,
            localized(&problem.title, "ko"),
            problem.difficulty,
            localized(&problem.statement, "ko"),
            localized(&problem.input, "ko"),
            localized(&problem.output, "ko"),
            examples
        ),
    )
}

pub fn upsert_problem_index(root: &Path, problem: &Problem, status: &str) -> Result<()> {
    let index = root.join("problems/INDEX.md");
    if let Some(parent) = index.parent() {
        create_dir_all_beneath(root, parent)?;
    }
    let mut rows: HashMap<String, (String, String, String, String)> = HashMap::new();
    if regular_file_exists(&index)? {
        for line in fs::read_to_string(&index)?.lines() {
            let parts = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>();
            if parts.len() == 5 && parts[0].chars().all(|c| c.is_ascii_digit()) {
                rows.insert(
                    parts[0].to_string(),
                    (
                        parts[1].to_string(),
                        parts[2].to_string(),
                        parts[3].to_string(),
                        parts[4].to_string(),
                    ),
                );
            }
        }
    }
    let number = problem
        .id
        .split_once('-')
        .map(|(number, _)| number)
        .unwrap_or(&problem.id);
    rows.insert(
        number.to_string(),
        (
            problem.slug.clone(),
            problem.difficulty.clone(),
            problem.topics.join(", "),
            status.to_string(),
        ),
    );
    let mut numbers = rows.keys().cloned().collect::<Vec<_>>();
    numbers.sort();
    let body = numbers
        .into_iter()
        .filter_map(|number| {
            rows.get(&number)
                .map(|(slug, difficulty, topics, row_status)| {
                    format!("| {number} | {slug} | {difficulty} | {topics} | {row_status} |")
                })
        })
        .collect::<Vec<_>>()
        .join("\n");
    save_user_text(
        &index,
        &format!(
            "# Problem Index\n\n| # | Slug | Difficulty | Topics | Status |\n|---|------|------------|--------|--------|\n{body}\n"
        ),
    )
}
