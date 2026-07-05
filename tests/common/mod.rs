use practicode::core::{IoCase, Problem, map2, save_bank, starter_problem};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn tmp_root(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = env::temp_dir().join(format!("practicode-{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    root
}

#[allow(dead_code)]
pub fn two_problem_bank(root: &Path) -> Vec<Problem> {
    let mut first = starter_problem();
    first.id = "001-hello-world".to_string();
    let mut second = first.clone();
    second.id = "002-echo".to_string();
    second.slug = "echo".to_string();
    second.topics = vec!["io".to_string(), "string".to_string()];
    second.title = map2("ko", "그대로 출력", "en", "Echo");
    second.statement = map2(
        "ko",
        "입력을 그대로 출력하세요.",
        "en",
        "Print stdin unchanged.",
    );
    second.input = map2("ko", "문자열", "en", "A string");
    second.output = map2("ko", "입력과 같은 문자열", "en", "The same string");
    second.examples = vec![IoCase {
        input: "code\n".to_string(),
        output: "code\n".to_string(),
    }];
    second.cases = second.examples.clone();
    second.answers = HashMap::from([
        (
            "python".to_string(),
            "import sys\nprint(sys.stdin.read(), end='')\n".to_string(),
        ),
        (
            "ts".to_string(),
            "const fs = require('fs');\nprocess.stdout.write(fs.readFileSync(0, 'utf8'));\n"
                .to_string(),
        ),
        (
            "java".to_string(),
            "class Solution { public static void main(String[] args) throws Exception { System.out.print(new String(System.in.readAllBytes())); } }\n".to_string(),
        ),
        (
            "rust".to_string(),
            "use std::io::{self, Read};\nfn main() { let mut s = String::new(); io::stdin().read_to_string(&mut s).unwrap(); print!(\"{}\", s); }\n".to_string(),
        ),
    ]);
    let bank = vec![first, second];
    save_bank(root, &bank).unwrap();
    bank
}
