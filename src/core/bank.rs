use super::*;

pub fn starter_problem() -> Problem {
    Problem {
        id: "001-hello-world".to_string(),
        slug: "hello-world".to_string(),
        difficulty: "easy".to_string(),
        topics: vec!["io".to_string()],
        title: localized_map(&[
            ("en", "Hello World"),
            ("ko", "Hello World"),
            ("ja", "Hello World"),
            ("zh", "Hello World"),
            ("es", "Hello World"),
        ]),
        statement: localized_map(&[
            ("en", "Print exactly `Hello, World!` to stdout."),
            ("ko", "표준 출력으로 정확히 `Hello, World!`를 출력하세요."),
            ("ja", "標準出力に正確に `Hello, World!` を出力してください。"),
            ("zh", "向标准输出准确打印 `Hello, World!`。"),
            ("es", "Imprime exactamente `Hello, World!` en stdout."),
        ]),
        input: localized_map(&[
            ("en", "No input."),
            ("ko", "입력은 없습니다."),
            ("ja", "入力はありません。"),
            ("zh", "没有输入。"),
            ("es", "No hay entrada."),
        ]),
        output: localized_map(&[
            ("en", "One line: `Hello, World!`"),
            ("ko", "`Hello, World!` 한 줄"),
            ("ja", "1行: `Hello, World!`"),
            ("zh", "一行: `Hello, World!`"),
            ("es", "Una linea: `Hello, World!`"),
        ]),
        examples: vec![IoCase {
            input: String::new(),
            output: "Hello, World!\n".to_string(),
        }],
        cases: vec![IoCase {
            input: String::new(),
            output: "Hello, World!\n".to_string(),
        }],
        answers: HashMap::from([
            ("python".to_string(), "print('Hello, World!')\n".to_string()),
            (
                "ts".to_string(),
                "console.log('Hello, World!');\n".to_string(),
            ),
            (
                "java".to_string(),
                "class Solution {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, World!\");\n    }\n}\n".to_string(),
            ),
            (
                "rust".to_string(),
                "fn main() {\n    println!(\"Hello, World!\");\n}\n".to_string(),
            ),
        ]),
    }
}

pub fn map2(k1: &str, v1: &str, k2: &str, v2: &str) -> HashMap<String, String> {
    HashMap::from([
        (k1.to_string(), v1.to_string()),
        (k2.to_string(), v2.to_string()),
    ])
}

pub fn localized_map(entries: &[(&str, &str)]) -> HashMap<String, String> {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

pub fn load_bank(root: &Path) -> Result<Vec<Problem>> {
    let path = root.join(BANK_PATH);
    if regular_file_exists(&path)? {
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let bank: Vec<Problem> =
            serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
        validate_bank(&bank, &path)?;
        Ok(bank)
    } else {
        Ok(vec![starter_problem()])
    }
}

pub fn save_bank(root: &Path, bank: &[Problem]) -> Result<()> {
    let path = root.join(BANK_PATH);
    validate_bank(bank, &path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    save_user_text(&path, &(serde_json::to_string_pretty(bank)? + "\n"))
}

fn validate_bank(bank: &[Problem], path: &Path) -> Result<()> {
    if bank.is_empty() {
        bail!("{} must contain at least one problem", path.display());
    }
    let mut ids = Vec::new();
    let mut slugs = Vec::new();
    for problem in bank {
        if !is_safe_name(&problem.id) {
            bail!("{} has invalid problem id {:?}", path.display(), problem.id);
        }
        if !is_safe_name(&problem.slug) {
            bail!(
                "{} has invalid slug {:?} for {}",
                path.display(),
                problem.slug,
                problem.id
            );
        }
        if ids.contains(&problem.id.as_str()) {
            bail!("{} has duplicate problem id {}", path.display(), problem.id);
        }
        if slugs.contains(&problem.slug.as_str()) {
            bail!("{} has duplicate slug {}", path.display(), problem.slug);
        }
        ids.push(problem.id.as_str());
        slugs.push(problem.slug.as_str());
        if problem.cases.is_empty() {
            bail!(
                "{} problem {} has no judge cases",
                path.display(),
                problem.id
            );
        }
        if problem.answers.is_empty() {
            bail!(
                "{} problem {} must contain at least one answer",
                path.display(),
                problem.id
            );
        }
        for language in problem.answers.keys() {
            if !LANGUAGES.contains(&language.as_str()) {
                bail!(
                    "{} problem {} has unsupported answer language {language}",
                    path.display(),
                    problem.id,
                );
            }
        }
    }
    Ok(())
}

fn is_safe_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|char| char.is_ascii_alphanumeric() || matches!(char, '-' | '_'))
}
