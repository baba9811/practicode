mod common;

use common::tmp_root;
use practicode::{
    core::{
        IoCase, JudgeFailureKind, JudgeResult, command_for, judge_path, normalize_judge_output,
    },
    process::run_capture,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

struct LanguageContract {
    dir: &'static str,
    runtime: &'static str,
    ids: &'static str,
    core_lessons: &'static str,
    checkpoints: &'static str,
    capstone: &'static str,
}

const CONTRACTS: &[LanguageContract] = &[
    LanguageContract {
        dir: "python",
        runtime: "python",
        ids: "py-output py-input py-variables py-numbers py-tuples-sets py-strings py-pathlib py-control-flow py-functions py-modules-imports py-lambdas-closures py-decorators py-async py-lists-dicts py-sorting-keys py-counter-defaultdict py-deque py-comprehensions py-generators py-itertools py-errors py-files-context py-dataclasses py-typing py-testing-assert",
        core_lessons: "py-output py-input py-variables py-numbers py-strings py-control-flow py-lists-dicts py-errors py-dataclasses",
        checkpoints: "py-functions py-comprehensions",
        capstone: "py-testing-assert",
    },
    LanguageContract {
        dir: "typescript",
        runtime: "ts",
        ids: "ts-output ts-input ts-primitives ts-let-const ts-strings-templates ts-arrays-tuples ts-iterables ts-array-methods ts-objects ts-classes ts-readonly ts-control-flow ts-functions ts-union-narrowing ts-literal-types ts-optional-nullish ts-interfaces-aliases ts-generics ts-keyof-typeof ts-indexed-access ts-mapped-types ts-conditional-types ts-utility-types ts-satisfies-as-const ts-discriminated-unions ts-async-promise ts-modules ts-error-handling",
        core_lessons: "ts-output ts-input ts-primitives ts-arrays-tuples ts-objects ts-union-narrowing ts-optional-nullish ts-interfaces-aliases ts-generics",
        checkpoints: "ts-functions ts-discriminated-unions",
        capstone: "ts-error-handling",
    },
    LanguageContract {
        dir: "java",
        runtime: "java",
        ids: "java-output java-input java-variables-types java-numbers-operators java-strings java-control-flow java-enum-switch java-methods java-overloading-varargs java-packages-imports java-arrays-collections java-generics java-streams-lambdas java-comparators-sorting java-classes-objects java-constructors java-encapsulation java-static-members java-interfaces java-inheritance-composition java-exceptions java-optional java-try-with-resources java-equality-hashcode java-records java-annotations java-sealed-classes java-testing-assert",
        core_lessons: "java-output java-input java-variables-types java-numbers-operators java-strings java-control-flow java-arrays-collections java-classes-objects java-exceptions",
        checkpoints: "java-methods java-records",
        capstone: "java-testing-assert",
    },
    LanguageContract {
        dir: "rust",
        runtime: "rust",
        ids: "rust-output rust-variables rust-numbers-tuples rust-input rust-vec-hashmap rust-control-flow rust-iterators rust-functions rust-modules-use rust-generics rust-traits rust-macros rust-cargo-workspaces rust-ownership rust-smart-pointers rust-interior-mutability rust-strings rust-borrowing-slices rust-lifetimes rust-traits-lifetimes rust-unsafe rust-structs-impl rust-enum-match rust-option rust-result rust-testing rust-async-await rust-concurrency rust-shared-state",
        core_lessons: "rust-output rust-variables rust-input rust-control-flow rust-ownership rust-strings rust-structs-impl rust-enum-match rust-result",
        checkpoints: "rust-functions rust-borrowing-slices",
        capstone: "rust-shared-state",
    },
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Course {
    schema_version: u8,
    runtime: String,
    lessons: Vec<CourseLesson>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CourseLesson {
    id: String,
    aliases: Vec<String>,
    track: String,
    kind: String,
    level: String,
    title: String,
    body: String,
    example: String,
    starter: String,
    cases: Vec<IoCase>,
    refs: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EnglishCatalog {
    schema_version: u8,
    programming_language: String,
    ui_language: String,
    lessons: BTreeMap<String, EnglishCopy>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EnglishCopy {
    title: String,
    concept: String,
    worked_example: String,
    common_mistakes: Vec<String>,
    self_check: Vec<String>,
    exercise_prompt: String,
    #[serde(default)]
    objective: String,
    #[serde(default)]
    language_delta: String,
    #[serde(default)]
    prediction_prompt: String,
    #[serde(default)]
    transfer_trap: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QualityFixture {
    schema_version: u8,
    lessons: BTreeMap<String, LessonFixture>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LessonFixture {
    reference_solution: String,
    starter_failure: FailureExpectation,
    mutants: Vec<Mutant>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum FailureExpectation {
    Output,
    TypeCheck,
    Compile,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mutant {
    name: String,
    expected_failure: FailureExpectation,
    replacements: Vec<Replacement>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Replacement {
    from: String,
    to: String,
}

fn read_json<T: for<'de> Deserialize<'de>>(path: impl AsRef<Path>) -> T {
    let path = path.as_ref();
    serde_json::from_str(&fs::read_to_string(path).unwrap())
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn course_path(contract: &LanguageContract) -> String {
    format!("assets/lessons/{}/course.json", contract.dir)
}

fn english_path(contract: &LanguageContract) -> String {
    format!("assets/lessons/{}/en.json", contract.dir)
}

fn fixture_path(contract: &LanguageContract) -> String {
    format!("tests/fixtures/lesson-quality/{}.json", contract.dir)
}

fn expected_ids(contract: &LanguageContract) -> Vec<&str> {
    contract.ids.split_whitespace().collect()
}

fn expected_classification(contract: &LanguageContract, id: &str) -> (&'static str, &'static str) {
    if contract
        .core_lessons
        .split_whitespace()
        .any(|core| core == id)
    {
        ("core", "lesson")
    } else if contract
        .checkpoints
        .split_whitespace()
        .any(|checkpoint| checkpoint == id)
    {
        ("core", "checkpoint")
    } else if contract.capstone == id {
        ("core", "capstone")
    } else {
        ("lab", "lesson")
    }
}

fn official_primary_reference(runtime: &str, reference: &str) -> bool {
    match runtime {
        "python" => {
            reference.starts_with("https://docs.python.org/3.12/")
                || reference.starts_with("https://peps.python.org/")
        }
        "ts" => {
            reference.starts_with("https://www.typescriptlang.org/docs/handbook/")
                || reference.starts_with("https://nodejs.org/docs/latest-v22.x/api/")
        }
        "java" => {
            reference.starts_with("https://dev.java/")
                || reference.starts_with("https://docs.oracle.com/en/java/javase/21/docs/api/")
                || reference.starts_with("https://docs.oracle.com/javase/specs/jls/se21/html/")
        }
        "rust" => reference.starts_with("https://doc.rust-lang.org/"),
        _ => false,
    }
}

fn starter_reads_stdin(runtime: &str, starter: &str) -> bool {
    let accepted = match runtime {
        "python" => &["sys.stdin", "from sys import stdin", "input("][..],
        "ts" => &["readFileSync(0", "process.stdin"][..],
        "java" => &["System.in"][..],
        "rust" => &["io::stdin", "std::io::stdin"][..],
        _ => &[][..],
    };
    accepted.iter().any(|pattern| starter.contains(pattern))
}

fn add_version_anchor_errors(
    errors: &mut Vec<String>,
    contract: &LanguageContract,
    course: &Course,
) {
    let references = course
        .lessons
        .iter()
        .flat_map(|lesson| lesson.refs.iter().map(String::as_str))
        .collect::<Vec<_>>();
    let has = |prefix: &str| {
        references
            .iter()
            .any(|reference| reference.starts_with(prefix))
    };
    match contract.runtime {
        "python" => {
            if !has("https://docs.python.org/3.12/") {
                errors.push("python: course needs a Python 3.12 documentation anchor".to_string());
            }
        }
        "ts" => {
            if !has(
                "https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-9.html",
            ) {
                errors.push("typescript: course needs a TypeScript 5.9 release anchor".to_string());
            }
            if !has("https://nodejs.org/docs/latest-v22.x/api/") {
                errors.push("typescript: course needs a Node latest-v22 anchor".to_string());
            }
        }
        "java" => {
            if !has("https://docs.oracle.com/en/java/javase/21/docs/api/") {
                errors.push("java: course needs a Java 21 API anchor".to_string());
            }
            if !has("https://docs.oracle.com/javase/specs/jls/se21/html/") {
                errors.push("java: course needs a JLS 21 anchor".to_string());
            }
        }
        "rust" => {
            if !has("https://doc.rust-lang.org/edition-guide/rust-2024/") {
                errors.push("rust: course needs a Rust 2024 Edition Guide anchor".to_string());
            }
        }
        _ => unreachable!("unsupported contract runtime"),
    }
}

fn normalized_8grams(copy: &EnglishCopy) -> HashSet<String> {
    let prose = format!(
        "{} {} {} {} {} {} {} {} {} {}",
        copy.title,
        copy.concept,
        copy.worked_example,
        copy.common_mistakes.join(" "),
        copy.self_check.join(" "),
        copy.exercise_prompt,
        copy.objective,
        copy.language_delta,
        copy.prediction_prompt,
        copy.transfer_trap,
    );
    let words = prose
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>();
    words.windows(8).map(|words| words.join(" ")).collect()
}

fn asset_contract_errors(
    contract: &LanguageContract,
    course: &Course,
    english: &EnglishCatalog,
) -> Vec<String> {
    let mut errors = Vec::new();
    let expected = expected_ids(contract);
    let actual = course
        .lessons
        .iter()
        .map(|lesson| lesson.id.as_str())
        .collect::<Vec<_>>();

    if course.schema_version != 1 || course.runtime != contract.runtime {
        errors.push(format!(
            "{}: expected schema 1/runtime {}, got {}/{}",
            contract.dir, contract.runtime, course.schema_version, course.runtime
        ));
    }
    if english.schema_version != 1
        || english.programming_language != contract.runtime
        || english.ui_language != "en"
    {
        errors.push(format!(
            "{}: invalid English catalog metadata",
            contract.dir
        ));
    }
    if actual != expected {
        let position = actual
            .iter()
            .zip(&expected)
            .position(|(actual, expected)| actual != expected)
            .unwrap_or(actual.len().min(expected.len()));
        errors.push(format!(
            "{}: lesson catalog differs from the required order at position {}",
            contract.dir,
            position + 1
        ));
    }

    let actual_ids = actual.iter().copied().collect::<BTreeSet<_>>();
    if actual_ids.len() != actual.len() {
        errors.push(format!("{}: course IDs contain duplicates", contract.dir));
    }
    if actual_ids != expected.iter().copied().collect() {
        errors.push(format!(
            "{}: course IDs differ from the exact contract",
            contract.dir
        ));
    }
    if english
        .lessons
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != actual_ids
    {
        errors.push(format!(
            "{}: English-copy IDs differ from course IDs",
            contract.dir
        ));
    }

    let mut core_lessons = 0;
    let mut checkpoints = 0;
    let mut capstones = 0;
    for lesson in &course.lessons {
        let expected = expected_classification(contract, &lesson.id);
        if (lesson.track.as_str(), lesson.kind.as_str()) != expected {
            errors.push(format!(
                "{}: expected {}/{} for {}, got {}/{}",
                contract.dir, expected.0, expected.1, lesson.id, lesson.track, lesson.kind
            ));
        }
        match (lesson.track.as_str(), lesson.kind.as_str()) {
            ("core", "lesson") => core_lessons += 1,
            ("core", "checkpoint") => checkpoints += 1,
            ("core", "capstone") => capstones += 1,
            ("lab", "lesson") => {}
            _ => errors.push(format!(
                "{}:{} must be core lesson/checkpoint/capstone or lab/lesson",
                contract.dir, lesson.id
            )),
        }

        for (field, value) in [
            ("id", lesson.id.as_str()),
            ("level", lesson.level.as_str()),
            ("title", lesson.title.as_str()),
            ("body", lesson.body.as_str()),
            ("example", lesson.example.as_str()),
            ("starter", lesson.starter.as_str()),
        ] {
            if value.trim().is_empty() {
                errors.push(format!("{}:{} has empty {field}", contract.dir, lesson.id));
            }
        }
        if lesson.starter.len() > 20_000 {
            errors.push(format!(
                "{}:{} starter exceeds 20k",
                contract.dir, lesson.id
            ));
        }
        if !starter_reads_stdin(contract.runtime, &lesson.starter) {
            errors.push(format!(
                "{}:{} starter must read judge input",
                contract.dir, lesson.id
            ));
        }
        if lesson.aliases.iter().any(|alias| alias.trim().is_empty()) {
            errors.push(format!("{}:{} has an empty alias", contract.dir, lesson.id));
        }
        if !(3..=5).contains(&lesson.cases.len()) {
            errors.push(format!(
                "{}:{} has {} cases; expected 3..=5",
                contract.dir,
                lesson.id,
                lesson.cases.len()
            ));
        }
        if lesson
            .cases
            .iter()
            .map(|case| case.input.as_str())
            .collect::<HashSet<_>>()
            .len()
            < 2
        {
            errors.push(format!(
                "{}:{} needs at least 2 distinct inputs",
                contract.dir, lesson.id
            ));
        }
        if lesson
            .cases
            .iter()
            .map(|case| case.output.as_str())
            .collect::<HashSet<_>>()
            .len()
            < 2
        {
            errors.push(format!(
                "{}:{} needs at least 2 distinct outputs",
                contract.dir, lesson.id
            ));
        }
        if lesson.cases.iter().any(|case| case.output.is_empty()) {
            errors.push(format!(
                "{}:{} has an empty case output",
                contract.dir, lesson.id
            ));
        }
        if lesson
            .cases
            .iter()
            .any(|case| case.input.len() > 4_096 || case.output.len() > 4_096)
        {
            errors.push(format!(
                "{}:{} case input/output exceeds 4096 bytes",
                contract.dir, lesson.id
            ));
        }
        if lesson
            .refs
            .iter()
            .any(|reference| reference.trim().is_empty() || !reference.starts_with("https://"))
        {
            errors.push(format!(
                "{}:{} has an invalid reference",
                contract.dir, lesson.id
            ));
        }
        if !lesson
            .refs
            .first()
            .is_some_and(|reference| official_primary_reference(contract.runtime, reference))
        {
            errors.push(format!(
                "{}:{} needs a version-aware official primary reference",
                contract.dir, lesson.id
            ));
        }

        let Some(copy) = english.lessons.get(&lesson.id) else {
            continue;
        };
        let missing = [
            ("title", copy.title.as_str()),
            ("concept", copy.concept.as_str()),
            ("worked_example", copy.worked_example.as_str()),
            ("exercise_prompt", copy.exercise_prompt.as_str()),
            ("objective", copy.objective.as_str()),
            ("language_delta", copy.language_delta.as_str()),
            ("prediction_prompt", copy.prediction_prompt.as_str()),
            ("transfer_trap", copy.transfer_trap.as_str()),
        ]
        .into_iter()
        .filter_map(|(field, value)| value.trim().is_empty().then_some(field))
        .collect::<Vec<_>>();
        if !missing.is_empty() {
            errors.push(format!(
                "{}:{} has empty English fields: {}",
                contract.dir,
                lesson.id,
                missing.join(", ")
            ));
        }
        for (field, values) in [
            ("common_mistakes", &copy.common_mistakes),
            ("self_check", &copy.self_check),
        ] {
            if values.len() < 2 || values.iter().any(|value| value.trim().is_empty()) {
                errors.push(format!(
                    "{}:{} needs 2 nonempty English {field} entries",
                    contract.dir, lesson.id
                ));
            }
        }
    }
    if (core_lessons, checkpoints, capstones) != (9, 2, 1) {
        errors.push(format!(
            "{}: core must contain 9 lessons, 2 checkpoints, 1 capstone; got {core_lessons}/{checkpoints}/{capstones}",
            contract.dir
        ));
    }

    add_version_anchor_errors(&mut errors, contract, course);
    let mut frequencies = HashMap::<String, usize>::new();
    for copy in english.lessons.values() {
        for gram in normalized_8grams(copy) {
            *frequencies.entry(gram).or_default() += 1;
        }
    }
    let mut repeated = frequencies
        .into_iter()
        .filter(|(_, count)| count * 5 > english.lessons.len())
        .collect::<Vec<_>>();
    repeated.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    if !repeated.is_empty() {
        let examples = repeated
            .iter()
            .take(3)
            .map(|(gram, count)| format!("`{gram}` ({count})"))
            .collect::<Vec<_>>();
        errors.push(format!(
            "{}: {} normalized prose 8-grams appear in over 20% of the course; examples: {}",
            contract.dir,
            repeated.len(),
            examples.join(", ")
        ));
    }
    errors
}

fn fixture_contract_errors(
    contract: &LanguageContract,
    course: &Course,
    fixture: &QualityFixture,
) -> Vec<String> {
    let mut errors = Vec::new();
    if fixture.schema_version != 1 {
        errors.push(format!("{}: fixture schema must be 1", contract.dir));
    }
    let course_ids = course
        .lessons
        .iter()
        .map(|lesson| lesson.id.as_str())
        .collect::<BTreeSet<_>>();
    let fixture_ids = fixture
        .lessons
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if fixture_ids != course_ids {
        errors.push(format!(
            "{}: fixture IDs must exactly cover the language course",
            contract.dir
        ));
    }

    for lesson in &course.lessons {
        let Some(entry) = fixture.lessons.get(&lesson.id) else {
            continue;
        };
        if entry.reference_solution.trim().is_empty() {
            errors.push(format!("{}: reference_solution is empty", lesson.id));
        }
        if entry.reference_solution.len() > 20_000 {
            errors.push(format!("{}: reference_solution exceeds 20k", lesson.id));
        }
        if lesson.track == "core" && entry.mutants.len() != 1 {
            errors.push(format!("{}: core item needs exactly 1 mutant", lesson.id));
        }
        if entry.mutants.len() > 1 {
            errors.push(format!("{}: at most 1 mutant is allowed", lesson.id));
        }
        for mutant in &entry.mutants {
            if mutant.name.trim().is_empty() {
                errors.push(format!("{}: mutant name is empty", lesson.id));
            }
            if !(1..=3).contains(&mutant.replacements.len()) {
                errors.push(format!(
                    "{}:{} needs 1..=3 replacements",
                    lesson.id, mutant.name
                ));
            }
            if mutant
                .replacements
                .iter()
                .any(|replacement| replacement.from.is_empty())
            {
                errors.push(format!(
                    "{}:{} has an empty replacement source",
                    lesson.id, mutant.name
                ));
            }
        }
    }
    errors
}

#[test]
fn global_exact_110_catalog_and_fixture_coverage() {
    let expected_count = CONTRACTS
        .iter()
        .map(|contract| expected_ids(contract).len())
        .sum::<usize>();
    let expected = CONTRACTS
        .iter()
        .flat_map(expected_ids)
        .collect::<BTreeSet<_>>();
    assert_eq!(expected_count, 110, "test contract must list 110 IDs");
    assert_eq!(expected.len(), 110, "test contract IDs must be unique");

    let mut errors = Vec::new();
    let mut actual = BTreeSet::new();
    let mut actual_count = 0;
    for contract in CONTRACTS {
        let course: Course = read_json(course_path(contract));
        let fixture: QualityFixture = read_json(fixture_path(contract));
        let ids = course
            .lessons
            .iter()
            .map(|lesson| lesson.id.as_str())
            .collect::<Vec<_>>();
        if ids != expected_ids(contract) {
            errors.push(format!(
                "{}: course order differs from contract",
                contract.dir
            ));
        }
        actual_count += ids.len();
        for id in &ids {
            if !actual.insert((*id).to_string()) {
                errors.push(format!("duplicate course ID: {id}"));
            }
        }
        if fixture.schema_version != 1
            || fixture
                .lessons
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
                != ids.iter().copied().collect()
        {
            errors.push(format!(
                "{}: fixture coverage differs from course",
                contract.dir
            ));
        }
    }
    if actual_count != 110
        || actual
            != expected
                .into_iter()
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
    {
        errors.push("courses must contain the exact 110 IDs without duplicates".to_string());
    }
    assert!(
        errors.is_empty(),
        "global lesson contract failed:\n{}",
        errors.join("\n")
    );
}

fn solution_path(root: &Path, id: &str, runtime: &str) -> PathBuf {
    let dir = root.join("submissions").join(id);
    fs::create_dir_all(&dir).unwrap();
    dir.join(match runtime {
        "python" => "solution.py",
        "ts" => "solution.ts",
        "java" => "Solution.java",
        "rust" => "solution.rs",
        _ => unreachable!("unsupported fixture runtime"),
    })
}

fn run_source(root: &Path, id: &str, runtime: &str, source: &str, cases: &[IoCase]) -> JudgeResult {
    let path = solution_path(root, id, runtime);
    fs::write(&path, source).unwrap();
    judge_path(root, id, &path, runtime, cases)
}

fn assert_output_starter_uses_input(
    root: &Path,
    contract: &LanguageContract,
    lesson: &CourseLesson,
) {
    let path = solution_path(root, &lesson.id, contract.runtime);
    fs::write(&path, &lesson.starter).unwrap();
    let command = command_for(root, &path, contract.runtime)
        .unwrap_or_else(|error| panic!("{} starter compile failed: {error}", lesson.id))
        .unwrap_or_else(|| panic!("{} runtime is unavailable", contract.runtime));
    let run_dir = root.join("build").join(&lesson.id).join("run");
    fs::create_dir_all(&run_dir).unwrap();
    let mut outputs = HashSet::new();

    for case in &lesson.cases {
        let mut process = Command::new(&command.program);
        process.args(&command.args).current_dir(&run_dir);
        let run = run_capture(&mut process, &case.input, Duration::from_secs(5))
            .unwrap_or_else(|error| panic!("{} starter run failed: {error}", lesson.id));
        assert!(
            !run.timed_out && run.code == Some(0),
            "{} Output starter must run successfully for every input:\n{}",
            lesson.id,
            run.stderr
        );
        outputs.insert(normalize_judge_output(&run.stdout));
    }

    assert!(
        outputs.len() >= 2,
        "{} Output starter must produce input-dependent output",
        lesson.id
    );
}

fn java_string_literal(value: &str) -> String {
    let mut literal = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => literal.push_str("\\\\"),
            '"' => literal.push_str("\\\""),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            '\u{0008}' => literal.push_str("\\b"),
            '\u{000c}' => literal.push_str("\\f"),
            character if character.is_control() => {
                literal.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => literal.push(character),
        }
    }
    literal.push('"');
    literal
}

fn hardcoded_first_output(runtime: &str, output: &str) -> String {
    match runtime {
        "python" => format!(
            "import sys\nsys.stdout.write({})\n",
            serde_json::to_string(output).unwrap()
        ),
        "ts" => format!(
            "process.stdout.write({});\n",
            serde_json::to_string(output).unwrap()
        ),
        "java" => format!(
            "class Solution {{ public static void main(String[] args) {{ System.out.print({}); }} }}\n",
            java_string_literal(output)
        ),
        "rust" => format!("fn main() {{ print!(\"{{}}\", {output:?}); }}\n"),
        _ => unreachable!("unsupported fixture runtime"),
    }
}

fn judge_failure(expectation: FailureExpectation) -> JudgeFailureKind {
    match expectation {
        FailureExpectation::Output => JudgeFailureKind::Output,
        FailureExpectation::TypeCheck => JudgeFailureKind::TypeCheck,
        FailureExpectation::Compile => JudgeFailureKind::Compile,
    }
}

fn assert_runtime_contract(contract: &LanguageContract, course: &Course, fixture: &QualityFixture) {
    let root = tmp_root(&format!("lesson-quality-{}", contract.dir));
    for lesson in &course.lessons {
        let entry = &fixture.lessons[&lesson.id];
        let reference = run_source(
            &root,
            &lesson.id,
            contract.runtime,
            &entry.reference_solution,
            &lesson.cases,
        );
        assert!(
            reference.passed,
            "{} reference failed ({:?}):\n{}",
            lesson.id, reference.failure_kind, reference.output
        );

        let starter = run_source(
            &root,
            &lesson.id,
            contract.runtime,
            &lesson.starter,
            &lesson.cases,
        );
        assert_eq!(
            starter.failure_kind,
            Some(judge_failure(entry.starter_failure)),
            "{} starter failed incorrectly:\n{}",
            lesson.id,
            starter.output
        );
        if entry.starter_failure == FailureExpectation::Output {
            assert_output_starter_uses_input(&root, contract, lesson);
        }

        let hardcode = run_source(
            &root,
            &lesson.id,
            contract.runtime,
            &hardcoded_first_output(contract.runtime, &lesson.cases[0].output),
            &lesson.cases,
        );
        assert_eq!(
            hardcode.failure_kind,
            Some(JudgeFailureKind::Output),
            "{} first-output hardcode must compile and fail by output:\n{}",
            lesson.id,
            hardcode.output
        );

        for mutant in &entry.mutants {
            let mut source = entry.reference_solution.clone();
            for replacement in &mutant.replacements {
                assert_eq!(
                    source.match_indices(&replacement.from).count(),
                    1,
                    "{}:{} replacement source must match exactly once: {:?}",
                    lesson.id,
                    mutant.name,
                    replacement.from
                );
                source = source.replacen(&replacement.from, &replacement.to, 1);
            }
            let result = run_source(
                &root,
                &format!("{}-mutant", lesson.id),
                contract.runtime,
                &source,
                &lesson.cases,
            );
            assert_eq!(
                result.failure_kind,
                Some(judge_failure(mutant.expected_failure)),
                "{}:{} mutant failed incorrectly:\n{}",
                lesson.id,
                mutant.name,
                result.output
            );
        }
    }
}

fn assert_language_contract(contract: &LanguageContract) {
    let course: Course = read_json(course_path(contract));
    let english: EnglishCatalog = read_json(english_path(contract));
    let fixture: QualityFixture = read_json(fixture_path(contract));
    let mut errors = asset_contract_errors(contract, &course, &english);
    errors.extend(fixture_contract_errors(contract, &course, &fixture));
    assert!(
        errors.is_empty(),
        "{} lesson contract failed:\n{}",
        contract.dir,
        errors.join("\n")
    );
    assert_runtime_contract(contract, &course, &fixture);
}

#[test]
fn python_lesson_quality_contract() {
    assert_language_contract(&CONTRACTS[0]);
}

#[test]
fn typescript_lesson_quality_contract() {
    assert_language_contract(&CONTRACTS[1]);
}

#[test]
fn java_lesson_quality_contract() {
    assert_language_contract(&CONTRACTS[2]);
}

#[test]
fn rust_lesson_quality_contract() {
    assert_language_contract(&CONTRACTS[3]);
}
