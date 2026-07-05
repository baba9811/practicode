use super::*;

pub struct SyntaxLesson {
    pub id: &'static str,
    pub titles: &'static [(&'static str, &'static str)],
    pub topics: &'static [&'static str],
    pub examples: &'static [SyntaxExample],
}

pub struct SyntaxExample {
    pub language: &'static str,
    pub code: &'static str,
}

const LESSONS: &[SyntaxLesson] = &[
    SyntaxLesson {
        id: "io",
        titles: &[
            ("en", "Standard input/output"),
            ("ko", "표준 입출력"),
            ("ja", "標準入出力"),
            ("zh", "标准输入输出"),
            ("es", "Entrada/salida estandar"),
        ],
        topics: &["io", "stdin", "stdout", "input", "output"],
        examples: &[
            SyntaxExample {
                language: "python",
                code: "import sys\ntext = sys.stdin.read()\nprint(text, end=\"\")",
            },
            SyntaxExample {
                language: "ts",
                code: "const fs = require(\"fs\");\nconst input = fs.readFileSync(0, \"utf8\");\nprocess.stdout.write(input);",
            },
            SyntaxExample {
                language: "java",
                code: "import java.io.*;\n\nclass Solution {\n    public static void main(String[] args) throws Exception {\n        String input = new String(System.in.readAllBytes());\n        System.out.print(input);\n    }\n}",
            },
            SyntaxExample {
                language: "rust",
                code: "use std::io::{self, Read};\n\nfn main() {\n    let mut input = String::new();\n    io::stdin().read_to_string(&mut input).unwrap();\n    print!(\"{input}\");\n}",
            },
        ],
    },
    SyntaxLesson {
        id: "strings",
        titles: &[
            ("en", "Strings"),
            ("ko", "문자열"),
            ("ja", "文字列"),
            ("zh", "字符串"),
            ("es", "Cadenas"),
        ],
        topics: &["string", "strings", "char", "chars", "text"],
        examples: &[
            SyntaxExample {
                language: "python",
                code: "s = \"hello\"\nprint(len(s))\nfor ch in s:\n    print(ch)",
            },
            SyntaxExample {
                language: "ts",
                code: "const s = \"hello\";\nconsole.log(s.length);\nfor (const ch of s) {\n  console.log(ch);\n}",
            },
            SyntaxExample {
                language: "java",
                code: "String s = \"hello\";\nSystem.out.println(s.length());\nfor (int i = 0; i < s.length(); i++) {\n    System.out.println(s.charAt(i));\n}",
            },
            SyntaxExample {
                language: "rust",
                code: "let s = \"hello\";\nprintln!(\"{}\", s.chars().count());\nfor ch in s.chars() {\n    println!(\"{ch}\");\n}",
            },
        ],
    },
    SyntaxLesson {
        id: "loops",
        titles: &[
            ("en", "Loops and conditions"),
            ("ko", "반복문과 조건문"),
            ("ja", "ループと条件分岐"),
            ("zh", "循环和条件"),
            ("es", "Bucles y condiciones"),
        ],
        topics: &["loop", "loops", "condition", "conditions", "control-flow"],
        examples: &[
            SyntaxExample {
                language: "python",
                code: "for n in range(5):\n    if n % 2 == 0:\n        print(n)",
            },
            SyntaxExample {
                language: "ts",
                code: "for (let n = 0; n < 5; n++) {\n  if (n % 2 === 0) console.log(n);\n}",
            },
            SyntaxExample {
                language: "java",
                code: "for (int n = 0; n < 5; n++) {\n    if (n % 2 == 0) System.out.println(n);\n}",
            },
            SyntaxExample {
                language: "rust",
                code: "for n in 0..5 {\n    if n % 2 == 0 {\n        println!(\"{n}\");\n    }\n}",
            },
        ],
    },
    SyntaxLesson {
        id: "arrays",
        titles: &[
            ("en", "Arrays and lists"),
            ("ko", "배열과 리스트"),
            ("ja", "配列とリスト"),
            ("zh", "数组和列表"),
            ("es", "Arreglos y listas"),
        ],
        topics: &["array", "arrays", "list", "lists", "vec", "vector"],
        examples: &[
            SyntaxExample {
                language: "python",
                code: "nums = [1, 2, 3]\nnums.append(4)\nprint(sum(nums))",
            },
            SyntaxExample {
                language: "ts",
                code: "const nums = [1, 2, 3];\nnums.push(4);\nconsole.log(nums.reduce((a, b) => a + b, 0));",
            },
            SyntaxExample {
                language: "java",
                code: "int[] nums = {1, 2, 3};\nint sum = 0;\nfor (int n : nums) sum += n;\nSystem.out.println(sum);",
            },
            SyntaxExample {
                language: "rust",
                code: "let nums = vec![1, 2, 3];\nlet sum: i32 = nums.iter().sum();\nprintln!(\"{sum}\");",
            },
        ],
    },
    SyntaxLesson {
        id: "functions",
        titles: &[
            ("en", "Functions"),
            ("ko", "함수"),
            ("ja", "関数"),
            ("zh", "函数"),
            ("es", "Funciones"),
        ],
        topics: &["function", "functions", "method", "methods"],
        examples: &[
            SyntaxExample {
                language: "python",
                code: "def add(a, b):\n    return a + b\n\nprint(add(2, 3))",
            },
            SyntaxExample {
                language: "ts",
                code: "function add(a: number, b: number): number {\n  return a + b;\n}\n\nconsole.log(add(2, 3));",
            },
            SyntaxExample {
                language: "java",
                code: "static int add(int a, int b) {\n    return a + b;\n}\n\nSystem.out.println(add(2, 3));",
            },
            SyntaxExample {
                language: "rust",
                code: "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\nprintln!(\"{}\", add(2, 3));",
            },
        ],
    },
];

pub fn syntax_lessons() -> &'static [SyntaxLesson] {
    LESSONS
}

pub fn syntax_lessons_for_problem(problem: &Problem) -> Vec<&'static SyntaxLesson> {
    LESSONS
        .iter()
        .filter(|lesson| {
            problem
                .topics
                .iter()
                .any(|topic| lesson.matches_topic(topic))
        })
        .collect()
}

pub fn syntax_code_for(lesson: &SyntaxLesson, language: &str) -> &'static str {
    let language = normalize_language(language);
    lesson
        .examples
        .iter()
        .find(|example| example.language == language)
        .or_else(|| lesson.examples.first())
        .map(|example| example.code)
        .unwrap_or("")
}

pub fn syntax_language_name(language: &str) -> &'static str {
    match normalize_language(language).as_str() {
        "python" => "Python",
        "ts" => "TypeScript",
        "java" => "Java",
        "rust" => "Rust",
        _ => "Python",
    }
}

pub fn syntax_lesson_title(lesson: &SyntaxLesson, ui_language: &str) -> &'static str {
    let lang = normalize_ui_language(ui_language);
    lesson
        .titles
        .iter()
        .find(|(key, _)| *key == lang)
        .or_else(|| lesson.titles.iter().find(|(key, _)| *key == "en"))
        .or_else(|| lesson.titles.first())
        .map(|(_, title)| *title)
        .unwrap_or("")
}

pub fn syntax_lesson_completed(state: &AppState, language: &str, lesson_id: &str) -> bool {
    let language = normalize_language(language);
    state
        .syntax_progress
        .get(&language)
        .is_some_and(|ids| ids.iter().any(|id| id == lesson_id))
}

pub fn syntax_progress_count(state: &AppState, language: &str) -> (usize, usize) {
    let language = normalize_language(language);
    let done = state
        .syntax_progress
        .get(&language)
        .map(|ids| ids.len())
        .unwrap_or_default();
    (done, LESSONS.len())
}

pub fn record_syntax_progress(state: &mut AppState, problem: &Problem) {
    let lesson_ids = syntax_lessons_for_problem(problem)
        .into_iter()
        .map(|lesson| lesson.id.to_string())
        .collect::<Vec<_>>();
    if lesson_ids.is_empty() {
        return;
    }
    let language = normalize_language(&state.settings.language);
    let mut ids = state.syntax_progress.remove(&language).unwrap_or_default();
    for id in lesson_ids {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    state
        .syntax_progress
        .insert(language, normalize_lesson_ids(&ids));
}

pub fn normalize_syntax_progress(
    progress: &HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    let mut normalized = HashMap::new();
    for (language, ids) in progress {
        let language = language.trim().to_lowercase();
        if LANGUAGES.contains(&language.as_str()) {
            let ids = normalize_lesson_ids(ids);
            if !ids.is_empty() {
                normalized.insert(language, ids);
            }
        }
    }
    normalized
}

pub fn syntax_lesson_text(
    problem: &Problem,
    language: &str,
    ui_language: &str,
    state: &AppState,
) -> String {
    let language = normalize_language(language);
    let lessons = syntax_lessons_for_problem(problem);
    let lang = normalize_ui_language(ui_language);
    if lessons.is_empty() {
        return ui_text(&lang, "syntax_no_lesson").to_string();
    }

    let name = syntax_language_name(&language);
    let mut lines = vec![format!("# {}: {name}", ui_text(&lang, "syntax"))];
    for lesson in lessons {
        let checked = if syntax_lesson_completed(state, &language, lesson.id) {
            "[x]"
        } else {
            "[ ]"
        };
        lines.extend([
            String::new(),
            format!("## {checked} {}", syntax_lesson_title(lesson, &lang)),
            String::new(),
            format!("```{language}"),
            syntax_code_for(lesson, &language).to_string(),
            "```".to_string(),
        ]);
    }
    lines.extend([
        String::new(),
        format!("{} ({})", ui_text(&lang, "syntax_practice"), name),
    ]);
    lines.join("\n")
}

impl SyntaxLesson {
    fn matches_topic(&self, topic: &str) -> bool {
        let topic = topic.trim().to_lowercase();
        self.topics.contains(&topic.as_str())
    }
}

fn normalize_lesson_ids(ids: &[String]) -> Vec<String> {
    LESSONS
        .iter()
        .filter(|lesson| ids.iter().any(|id| id == lesson.id))
        .map(|lesson| lesson.id.to_string())
        .collect()
}
