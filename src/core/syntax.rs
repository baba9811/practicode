use super::*;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
struct SyntaxLessonCopy {
    title: String,
    concept: String,
    worked_example: String,
    common_mistakes: Vec<String>,
    self_check: Vec<String>,
    exercise_prompt: String,
}

#[derive(Debug, Deserialize)]
struct SyntaxLessonCatalog {
    schema_version: u8,
    #[serde(rename = "programming_language")]
    _programming_language: String,
    #[serde(rename = "ui_language")]
    _ui_language: String,
    lessons: HashMap<String, SyntaxLessonCopy>,
}

type SyntaxLessonCopyMap = HashMap<String, SyntaxLessonCopy>;

static PY_EN_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static PY_KO_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static PY_JA_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static PY_ZH_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static PY_ES_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static TS_EN_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static TS_KO_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static TS_JA_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static TS_ZH_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static TS_ES_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static JAVA_EN_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static JAVA_KO_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static JAVA_JA_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static JAVA_ZH_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static JAVA_ES_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static RUST_EN_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static RUST_KO_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static RUST_JA_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static RUST_ZH_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();
static RUST_ES_LESSONS: OnceLock<SyntaxLessonCopyMap> = OnceLock::new();

#[derive(Clone, Copy, Debug)]
pub struct SyntaxCase {
    pub input: &'static str,
    pub output: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct SyntaxExercise {
    pub prompt: &'static str,
    pub starter: &'static str,
    pub cases: &'static [SyntaxCase],
}

#[derive(Clone, Copy, Debug)]
pub struct SyntaxLesson {
    pub id: &'static str,
    pub language: &'static str,
    pub level: &'static str,
    pub title: &'static str,
    pub body: &'static str,
    pub example: &'static str,
    pub exercise: SyntaxExercise,
    pub refs: &'static [&'static str],
}

macro_rules! lesson {
    ($id:expr, $language:expr, $level:expr, $title:expr, $body:expr, $example:expr, $starter:expr, $cases:expr, $refs:expr) => {
        SyntaxLesson {
            id: $id,
            language: $language,
            level: $level,
            title: $title,
            body: $body,
            example: $example,
            exercise: SyntaxExercise {
                prompt: "Before you run, predict the output. Then run the starter and edit it until the expected output matches.",
                starter: $starter,
                cases: $cases,
            },
            refs: $refs,
        }
    };
}

const PY_CORE_REFS: &[&str] = &[
    "https://docs.python.org/3/tutorial/index.html",
    "https://docs.python.org/3/reference/index.html",
    "https://docs.python.org/3/library/index.html",
    "https://peps.python.org/pep-0008/",
];
const TS_REFS: &[&str] = &[
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide",
    "https://www.typescriptlang.org/docs/handbook/intro.html",
];
const JAVA_REFS: &[&str] = &[
    "https://dev.java/learn/",
    "https://docs.oracle.com/javase/tutorial/",
];
const EMPTY_HELLO: &[SyntaxCase] = &[SyntaxCase {
    input: "",
    output: "ok\n",
}];
const ECHO_CASE: &[SyntaxCase] = &[SyntaxCase {
    input: "code\n",
    output: "code\n",
}];
const SUM_CASE: &[SyntaxCase] = &[SyntaxCase {
    input: "2 3\n",
    output: "5\n",
}];

const PYTHON_LESSONS: &[SyntaxLesson] = &[
    lesson!(
        "py-output",
        "python",
        "basic",
        "print and stdout",
        "print converts values to text, writes them to stdout, and adds a newline unless told otherwise.",
        "name = 'Ada'\nscore = 7\nprint(f'{name}:{score}')",
        "name = 'Ada'\nscore = 7\n# TODO: print exactly Ada:7 using the variables above\nprint('TODO')\n",
        &[SyntaxCase {
            input: "",
            output: "Ada:7\n",
        }],
        &[
            "https://docs.python.org/3/library/functions.html#print",
            "https://docs.python.org/3/tutorial/inputoutput.html",
            "https://peps.python.org/pep-0008/",
        ]
    ),
    lesson!(
        "py-variables",
        "python",
        "basic",
        "Variables",
        "Assignment binds a name to an object; rebinding changes what the name points at, not the old object.",
        "count = 1\ncount = count + 2\nprint(count)",
        "word = 'todo'\n# TODO: rebind word to the expected text\nprint(word)\n",
        EMPTY_HELLO,
        PY_CORE_REFS
    ),
    lesson!(
        "py-numbers",
        "python",
        "basic",
        "Numbers",
        "int and float cover most numeric work; //, %, and ** are common in problem solutions.",
        "total = 7\nsize = 2\nprint(f'{total // size}:{total % size}')",
        "total = 7\nsize = 2\n# TODO: use integer division and remainder so the output is 3:1\nprint(f'{total / size}:0')\n",
        &[SyntaxCase {
            input: "",
            output: "3:1\n",
        }],
        &[
            "https://docs.python.org/3/tutorial/introduction.html#numbers",
            "https://docs.python.org/3/library/stdtypes.html#numeric-types-int-float-complex",
        ]
    ),
    lesson!(
        "py-strings",
        "python",
        "basic",
        "Strings",
        "Strings are immutable sequences, so indexing and slicing read characters without changing the original text.",
        "text = 'python'\nprint(text[1:4])",
        "text = 'xokx'\n# TODO: use a slice to print ok\nprint(text)\n",
        EMPTY_HELLO,
        &[
            "https://docs.python.org/3/tutorial/introduction.html#text",
            "https://docs.python.org/3/library/stdtypes.html#text-sequence-type-str",
        ]
    ),
    lesson!(
        "py-control-flow",
        "python",
        "basic",
        "Control flow",
        "if chooses a block, for iterates over an iterable, and while repeats until its condition changes.",
        "total = 0\nfor n in range(1, 4):\n    if n % 2 == 1:\n        total += n\nprint(total)",
        "total = 0\nfor n in range(1, 4):\n    # TODO: add only odd numbers\n    total += 0\nprint(total)\n",
        &[SyntaxCase {
            input: "",
            output: "4\n",
        }],
        &[
            "https://docs.python.org/3/tutorial/controlflow.html",
            "https://docs.python.org/3/reference/compound_stmts.html",
        ]
    ),
    lesson!(
        "py-functions",
        "python",
        "basic",
        "Functions",
        "def creates a callable object; parameters receive arguments and return sends a value back to the caller.",
        "def area(width, height):\n    return width * height\n\nprint(area(3, 4))",
        "def area(width, height):\n    # TODO: return rectangle area, not perimeter\n    return width + height\n\nprint(area(3, 4))\n",
        &[SyntaxCase {
            input: "",
            output: "12\n",
        }],
        &[
            "https://docs.python.org/3/tutorial/controlflow.html#defining-functions",
            "https://docs.python.org/3/reference/compound_stmts.html#function-definitions",
        ]
    ),
    lesson!(
        "py-input",
        "python",
        "intermediate",
        "Input parsing",
        "stdin starts as text; read it once, split into tokens when structure matters, then convert tokens explicitly.",
        "import sys\nnums = [int(token) for token in sys.stdin.read().split()]\nprint(sum(nums))",
        "import sys\nnums = []\n# TODO: parse all integers from stdin and print their sum\nprint(sum(nums))\n",
        SUM_CASE,
        &[
            "https://docs.python.org/3/library/sys.html#sys.stdin",
            "https://docs.python.org/3/tutorial/inputoutput.html",
        ]
    ),
    lesson!(
        "py-lists-dicts",
        "python",
        "intermediate",
        "Lists and dicts",
        "Lists keep ordered values by position; dicts map keys to values for direct lookup and counting.",
        "scores = {'Ada': [2, 3], 'Lin': [4]}\nprint(sum(scores['Ada']))",
        "nums = [2, 3]\nscores = {'Ada': nums}\n# TODO: print the sum stored under Ada without hard-coding 5\nprint(len(scores['Ada']))\n",
        SUM_CASE,
        &[
            "https://docs.python.org/3/tutorial/datastructures.html#more-on-lists",
            "https://docs.python.org/3/tutorial/datastructures.html#dictionaries",
            "https://docs.python.org/3/library/stdtypes.html#mapping-types-dict",
        ]
    ),
    lesson!(
        "py-tuples-sets",
        "python",
        "basic",
        "Tuples and sets",
        "Tuples group a fixed sequence of values; sets keep unique members and make membership checks cheap.",
        "pair = ('o', 'k')\nseen = set(pair)\nprint(''.join(pair), len(seen))",
        "pair = ('o', 'k')\nseen = set()\n# TODO: build a set from the tuple and print ok 2\nprint(''.join(pair[:1]), len(seen))\n",
        &[SyntaxCase {
            input: "",
            output: "ok 2\n",
        }],
        &[
            "https://docs.python.org/3/tutorial/datastructures.html#tuples-and-sequences",
            "https://docs.python.org/3/tutorial/datastructures.html#sets",
            "https://docs.python.org/3/library/stdtypes.html#set-types-set-frozenset",
        ]
    ),
    lesson!(
        "py-comprehensions",
        "python",
        "intermediate",
        "Comprehensions",
        "A comprehension combines an output expression, a loop, and optional filters into one collection-building expression.",
        "nums = [1, 2, 3, 4]\nsquares = [n * n for n in nums if n % 2 == 0]\nprint(sum(squares))",
        "letters = ['o', 'x', 'k']\n# TODO: keep only the letters needed for ok with a comprehension\nword = ''.join([ch for ch in letters if ch != 'x' and ch != 'k'])\nprint(word)\n",
        EMPTY_HELLO,
        &[
            "https://docs.python.org/3/tutorial/datastructures.html#list-comprehensions",
            "https://docs.python.org/3/tutorial/datastructures.html#dictionaries",
        ]
    ),
    lesson!(
        "py-errors",
        "python",
        "intermediate",
        "Exceptions",
        "try isolates code that may fail; except handles a specific recoverable error without hiding unrelated bugs.",
        "try:\n    value = int('12')\nexcept ValueError:\n    value = 0\nprint(value)",
        "try:\n    value = int('bad')\nexcept ValueError:\n    # TODO: recover with the expected value\n    value = 0\nprint(value)\n",
        &[SyntaxCase {
            input: "",
            output: "12\n",
        }],
        &[
            "https://docs.python.org/3/tutorial/errors.html",
            "https://docs.python.org/3/reference/compound_stmts.html#the-try-statement",
        ]
    ),
    lesson!(
        "py-files-context",
        "python",
        "intermediate",
        "Files and context managers",
        "with enters a managed scope and calls cleanup automatically; file handles and contextlib helpers use this pattern.",
        "from io import StringIO\n\nwith StringIO('ok') as handle:\n    text = handle.read()\nprint(text)",
        "from io import StringIO\n\nwith StringIO('ok') as handle:\n    # TODO: read from the managed handle before it closes\n    text = ''\nprint(text)\n",
        EMPTY_HELLO,
        &[
            "https://docs.python.org/3/tutorial/inputoutput.html#reading-and-writing-files",
            "https://docs.python.org/3/reference/compound_stmts.html#the-with-statement",
            "https://docs.python.org/3/library/contextlib.html",
        ]
    ),
    lesson!(
        "py-modules-imports",
        "python",
        "basic",
        "Modules and imports",
        "import binds a module or object name so code can reuse standard-library behavior instead of rewriting it.",
        "import math\n\nprint(math.ceil(2.1))",
        "import math\n\n# TODO: use the imported module to round upward\nprint(math.floor(2.1))\n",
        &[SyntaxCase {
            input: "",
            output: "3\n",
        }],
        &[
            "https://docs.python.org/3/tutorial/modules.html",
            "https://docs.python.org/3/reference/import.html",
            "https://peps.python.org/pep-0008/#imports",
        ]
    ),
    lesson!(
        "py-dataclasses",
        "python",
        "intermediate",
        "Dataclasses",
        "dataclass generates the routine class methods for simple data containers while leaving behavior explicit.",
        "from dataclasses import dataclass\n\n@dataclass\nclass Point:\n    x: int\n    y: int\n\npoint = Point(2, 3)\nprint(point.x + point.y)",
        "from dataclasses import dataclass\n\n@dataclass\nclass Point:\n    x: int\n    y: int\n\npoint = Point(2, 3)\n# TODO: use both fields\nprint(point.x)\n",
        SUM_CASE,
        &["https://docs.python.org/3/library/dataclasses.html"]
    ),
    lesson!(
        "py-typing",
        "python",
        "intermediate",
        "Type hints",
        "Type hints document expected shapes for readers and tools; Python still executes values dynamically at runtime.",
        "from typing import Iterable\n\ndef total(values: Iterable[int]) -> int:\n    return sum(values)\n\nprint(total([2, 3]))",
        "from typing import Iterable\n\ndef total(values: Iterable[int]) -> int:\n    # TODO: return the sum of the iterable\n    return 0\n\nprint(total([2, 3]))\n",
        SUM_CASE,
        &[
            "https://docs.python.org/3/library/typing.html",
            "https://docs.python.org/3/tutorial/controlflow.html#function-annotations",
        ]
    ),
    lesson!(
        "py-generators",
        "python",
        "advanced",
        "Iterators and generators",
        "Iterators produce values one at a time; a generator function uses yield to pause and resume that production.",
        "def countdown(n):\n    while n > 0:\n        yield n\n        n -= 1\n\nprint(next(countdown(3)))",
        "def words():\n    # TODO: yield ok as the first generated value\n    yield ''\n\nprint(next(words()))\n",
        EMPTY_HELLO,
        &[
            "https://docs.python.org/3/tutorial/classes.html#iterators",
            "https://docs.python.org/3/tutorial/classes.html#generators",
            "https://docs.python.org/3/reference/simple_stmts.html#the-yield-statement",
        ]
    ),
    lesson!(
        "py-lambdas-closures",
        "python",
        "advanced",
        "Lambdas and closures",
        "lambda makes a small expression function; a closure remembers names from the surrounding scope.",
        "def make_adder(delta):\n    return lambda value: value + delta\n\nadd_two = make_adder(2)\nprint(add_two(3))",
        "def make_suffix(suffix):\n    # TODO: return a lambda that appends suffix to word\n    return lambda word: word\n\nadd_ok = make_suffix('ok')\nprint(add_ok(''))\n",
        EMPTY_HELLO,
        &[
            "https://docs.python.org/3/tutorial/controlflow.html#lambda-expressions",
            "https://docs.python.org/3/reference/expressions.html#lambda",
            "https://docs.python.org/3/tutorial/classes.html#python-scopes-and-namespaces",
        ]
    ),
    lesson!(
        "py-decorators",
        "python",
        "advanced",
        "Decorators",
        "A decorator receives a function at definition time and returns the function object that name should now refer to.",
        "def identity(fn):\n    return fn\n\n@identity\ndef word():\n    return 'ok'\n\nprint(word())",
        "def identity(fn):\n    # TODO: return the original function unchanged\n    return lambda: ''\n\n@identity\ndef word():\n    return 'ok'\n\nprint(word())\n",
        EMPTY_HELLO,
        &[
            "https://docs.python.org/3/reference/compound_stmts.html#function-definitions",
            "https://docs.python.org/3/glossary.html#term-decorator",
        ]
    ),
    lesson!(
        "py-sorting-keys",
        "python",
        "intermediate",
        "Sorting and key functions",
        "sorted returns a new ordered list; key functions choose the value used for each comparison.",
        "users = [('Ada', 3), ('Lin', 5), ('Bo', 4)]\nbest = sorted(users, key=lambda item: item[1], reverse=True)[0]\nprint(f'{best[0]}:{best[1]}')",
        "users = [('Ada', 3), ('Lin', 5), ('Bo', 4)]\n# TODO: sort by score descending, not by name\nbest = sorted(users)[0]\nprint(f'{best[0]}:{best[1]}')\n",
        &[SyntaxCase {
            input: "",
            output: "Lin:5\n",
        }],
        &[
            "https://docs.python.org/3/library/functions.html#sorted",
            "https://docs.python.org/3/howto/sorting.html",
        ]
    ),
    lesson!(
        "py-counter-defaultdict",
        "python",
        "intermediate",
        "Counter and defaultdict",
        "Counter counts hashable values directly; defaultdict creates missing collection values when grouping.",
        "from collections import Counter, defaultdict\n\nwords = ['red', 'blue', 'red']\ncounts = Counter(words)\ngroups = defaultdict(list)\nfor word in words:\n    groups[word[0]].append(word)\nprint(counts['red'], len(groups['r']))",
        "from collections import Counter, defaultdict\n\nwords = ['red', 'blue', 'red']\ncounts = Counter()\ngroups = defaultdict(list)\n# TODO: count words and group them by first letter\nprint(counts['red'], len(groups['r']))\n",
        &[SyntaxCase {
            input: "",
            output: "2 2\n",
        }],
        &["https://docs.python.org/3/library/collections.html"]
    ),
    lesson!(
        "py-deque",
        "python",
        "intermediate",
        "deque",
        "deque supports efficient appends and pops on both ends, which is why it is the usual queue type.",
        "from collections import deque\n\nqueue = deque(['middle'])\nqueue.appendleft('start')\nqueue.append('end')\nprint(queue.popleft(), queue.pop())",
        "from collections import deque\n\nqueue = deque(['middle'])\n# TODO: add start on the left and end on the right\nprint(queue.popleft(), 'missing')\n",
        &[SyntaxCase {
            input: "",
            output: "start end\n",
        }],
        &["https://docs.python.org/3/library/collections.html#collections.deque"]
    ),
    lesson!(
        "py-itertools",
        "python",
        "advanced",
        "itertools",
        "itertools provides lazy iterator building blocks for pairing, chaining, slicing, and combinatorics.",
        "import itertools\n\nparts = [['o'], ['k']]\nprint(''.join(itertools.chain.from_iterable(parts)))",
        "import itertools\n\nparts = [['o'], ['k']]\n# TODO: flatten both inner lists lazily\nprint(''.join(itertools.chain.from_iterable(parts[:1])))\n",
        EMPTY_HELLO,
        &["https://docs.python.org/3/library/itertools.html"]
    ),
    lesson!(
        "py-pathlib",
        "python",
        "intermediate",
        "pathlib",
        "pathlib represents paths as objects, so code can ask for names, suffixes, and parents without manual string splitting.",
        "from pathlib import PurePosixPath\n\npath = PurePosixPath('logs/app.txt')\nprint(f'{path.stem}:{path.suffix}')",
        "from pathlib import PurePosixPath\n\npath = PurePosixPath('logs/app.txt')\n# TODO: print the stem and suffix as app:.txt\nprint(path.name)\n",
        &[SyntaxCase {
            input: "",
            output: "app:.txt\n",
        }],
        &["https://docs.python.org/3/library/pathlib.html"]
    ),
    lesson!(
        "py-testing-assert",
        "python",
        "intermediate",
        "Testing and assert",
        "assert checks an invariant in small examples; test frameworks build on the same idea with repeatable test functions.",
        "def add_two(value):\n    return value + 2\n\nassert add_two(3) == 5\nprint('ok')",
        "def add_two(value):\n    # TODO: make the assertion describe the intended behavior\n    return value\n\nassert add_two(3) == 3\nprint('todo')\n",
        EMPTY_HELLO,
        &[
            "https://docs.python.org/3/reference/simple_stmts.html#the-assert-statement",
            "https://docs.python.org/3/library/unittest.html",
            "https://docs.python.org/3/tutorial/stdlib.html#quality-control",
        ]
    ),
    lesson!(
        "py-async",
        "python",
        "advanced",
        "Async concepts",
        "async def creates a coroutine; await pauses until the awaited operation completes, and asyncio.run drives the top-level coroutine.",
        "import asyncio\n\nasync def label():\n    return 'ok'\n\nasync def main():\n    print(await label())\n\nasyncio.run(main())",
        "import asyncio\n\nasync def label():\n    return 'ok'\n\nasync def main():\n    # TODO: await the coroutine and print its result\n    result = 'pending'\n    print(result)\n\nasyncio.run(main())\n",
        EMPTY_HELLO,
        &[
            "https://docs.python.org/3/library/asyncio.html",
            "https://docs.python.org/3/reference/datamodel.html#coroutines",
            "https://docs.python.org/3/reference/expressions.html#await",
        ]
    ),
];

const TS_LESSONS: &[SyntaxLesson] = &[
    lesson!(
        "ts-output",
        "ts",
        "basic",
        "Output",
        "console.log writes a line.",
        "console.log('ok');",
        "// TODO: write the expected line\n",
        EMPTY_HELLO,
        TS_REFS
    ),
    lesson!(
        "ts-variables",
        "ts",
        "basic",
        "Variables",
        "let changes; const does not reassign.",
        "const name: string = 'code';",
        "let word: string = '';\n// TODO: assign the expected word, then log it\nconsole.log(word);\n",
        EMPTY_HELLO,
        TS_REFS
    ),
    lesson!(
        "ts-strings",
        "ts",
        "basic",
        "Strings",
        "Strings expose length and iteration.",
        "for (const ch of 'ok') console.log(ch);",
        "const text = 'xokx';\n// TODO: use a slice to print the middle text\nconsole.log(text);\n",
        EMPTY_HELLO,
        TS_REFS
    ),
    lesson!(
        "ts-control-flow",
        "ts",
        "basic",
        "Control flow",
        "if and loops control execution.",
        "for (let i = 0; i < 3; i++) {}",
        "const ready = false;\n// TODO: print the expected word only when ready is true\nif (ready) console.log('ok');\n",
        EMPTY_HELLO,
        TS_REFS
    ),
    lesson!(
        "ts-functions",
        "ts",
        "basic",
        "Functions",
        "Parameter and return types make intent explicit.",
        "function add(a: number, b: number): number { return a + b; }",
        "function word(): string {\n  // TODO: return the expected word\n  return '';\n}\nconsole.log(word());\n",
        EMPTY_HELLO,
        TS_REFS
    ),
    lesson!(
        "ts-input",
        "ts",
        "intermediate",
        "Input parsing",
        "Node can read stdin for small exercises.",
        "const input = require('fs').readFileSync(0, 'utf8');",
        "const fs = require('fs');\nconst input = fs.readFileSync(0, 'utf8');\n// TODO: write input back unchanged\n",
        ECHO_CASE,
        TS_REFS
    ),
    lesson!(
        "ts-arrays-objects",
        "ts",
        "intermediate",
        "Arrays and objects",
        "Arrays hold sequences; object types describe shapes.",
        "type User = { name: string };",
        "const nums: number[] = [2, 3];\n// TODO: print the sum of nums without hard-coding 5\nconsole.log(nums.length);\n",
        SUM_CASE,
        TS_REFS
    ),
    lesson!(
        "ts-errors-async",
        "ts",
        "intermediate",
        "Errors and async",
        "try/catch handles thrown errors; async wraps promises.",
        "async function main() { return 1; }",
        "try {\n  throw new Error('x');\n} catch {\n  // TODO: handle the expected error by printing the expected word\n}\n",
        EMPTY_HELLO,
        &["https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await"]
    ),
    lesson!(
        "ts-narrowing",
        "ts",
        "advanced",
        "Narrowing",
        "Type guards refine union values.",
        "if (typeof value === 'string') value.toUpperCase();",
        "const value: string | number = 0;\n// TODO: narrow a string value before printing\nif (typeof value === 'string') console.log(value);\n",
        EMPTY_HELLO,
        TS_REFS
    ),
    lesson!(
        "ts-generics",
        "ts",
        "advanced",
        "Generics",
        "Generics preserve type information across reusable code.",
        "function first<T>(xs: T[]): T { return xs[0]; }",
        "function id<T>(value: T): T { return value; }\n// TODO: call id with the expected word and print it\nconsole.log(id(''));\n",
        EMPTY_HELLO,
        TS_REFS
    ),
    lesson!(
        "ts-mapped",
        "ts",
        "advanced",
        "Mapped types",
        "Mapped types transform object properties.",
        "type ReadonlyUser<T> = { readonly [K in keyof T]: T[K] };",
        "type Box<T> = { [K in keyof T]: T[K] };\nconst value: Box<{ word: string }> = { word: '' };\n// TODO: store the expected word, then print it\nconsole.log(value.word);\n",
        EMPTY_HELLO,
        &["https://www.typescriptlang.org/docs/handbook/utility-types.html"]
    ),
    lesson!(
        "ts-conditional",
        "ts",
        "advanced",
        "Conditional types",
        "Conditional types choose a type from another type.",
        "type Unwrap<T> = T extends Promise<infer U> ? U : T;",
        "type IsString<T> = T extends string ? string : never;\nlet word: IsString<'ok'> = '';\n// TODO: assign the expected word, then print it\nconsole.log(word);\n",
        EMPTY_HELLO,
        TS_REFS
    ),
];

const JAVA_LESSONS: &[SyntaxLesson] = &[
    lesson!(
        "java-output",
        "java",
        "basic",
        "Output",
        "System.out.println writes a line.",
        "System.out.println(\"ok\");",
        "class Solution { public static void main(String[] args) { /* TODO: print the expected line */ } }\n",
        EMPTY_HELLO,
        JAVA_REFS
    ),
    lesson!(
        "java-variables",
        "java",
        "basic",
        "Variables",
        "Java variables have declared types.",
        "String word = \"ok\";",
        "class Solution { public static void main(String[] args) { String word = \"\"; /* TODO: assign the expected word */ System.out.println(word); } }\n",
        EMPTY_HELLO,
        JAVA_REFS
    ),
    lesson!(
        "java-strings",
        "java",
        "basic",
        "Strings",
        "String methods expose length, chars, and substrings.",
        "\"code\".substring(0, 2)",
        "class Solution { public static void main(String[] args) { String text = \"xokx\"; /* TODO: print the middle text */ System.out.println(text); } }\n",
        EMPTY_HELLO,
        JAVA_REFS
    ),
    lesson!(
        "java-control-flow",
        "java",
        "basic",
        "Control flow",
        "if, for, and while control execution.",
        "for (int i = 0; i < 3; i++) {}",
        "class Solution { public static void main(String[] args) { boolean ready = false; /* TODO: print the expected word only when ready is true */ if (ready) System.out.println(\"ok\"); } }\n",
        EMPTY_HELLO,
        JAVA_REFS
    ),
    lesson!(
        "java-methods",
        "java",
        "basic",
        "Methods",
        "Methods group reusable behavior.",
        "static int add(int a, int b) { return a + b; }",
        "class Solution { static String word() { /* TODO: return the expected word */ return \"\"; } public static void main(String[] args) { System.out.println(word()); } }\n",
        EMPTY_HELLO,
        JAVA_REFS
    ),
    lesson!(
        "java-input",
        "java",
        "intermediate",
        "Input parsing",
        "System.in can be read directly for exercises.",
        "String input = new String(System.in.readAllBytes());",
        "import java.io.*;\nclass Solution { public static void main(String[] args) throws Exception { String input = new String(System.in.readAllBytes()); /* TODO: write input back unchanged */ } }\n",
        ECHO_CASE,
        JAVA_REFS
    ),
    lesson!(
        "java-arrays-collections",
        "java",
        "intermediate",
        "Arrays and collections",
        "Arrays are fixed size; collections add flexible containers.",
        "int[] nums = {1, 2};",
        "class Solution { public static void main(String[] args) { int[] nums = {2, 3}; /* TODO: print the sum without hard-coding 5 */ System.out.println(nums.length); } }\n",
        SUM_CASE,
        JAVA_REFS
    ),
    lesson!(
        "java-exceptions",
        "java",
        "intermediate",
        "Exceptions",
        "try/catch handles failures; checked exceptions are part of signatures.",
        "try { throw new RuntimeException(); } catch (RuntimeException e) {}",
        "class Solution { public static void main(String[] args) { try { throw new RuntimeException(); } catch (RuntimeException e) { /* TODO: print the expected word */ } } }\n",
        EMPTY_HELLO,
        JAVA_REFS
    ),
    lesson!(
        "java-classes-interfaces",
        "java",
        "advanced",
        "Classes and interfaces",
        "Classes hold state and behavior; interfaces describe behavior.",
        "interface Named { String name(); }",
        "interface Named { String name(); }\nclass Solution implements Named { public String name() { /* TODO: return the expected word */ return \"\"; } public static void main(String[] args) { System.out.println(new Solution().name()); } }\n",
        EMPTY_HELLO,
        JAVA_REFS
    ),
    lesson!(
        "java-generics",
        "java",
        "advanced",
        "Generics",
        "Generics reuse code with type parameters.",
        "class Box<T> { T value; }",
        "class Box<T> { T value; Box(T value) { this.value = value; } }\nclass Solution { public static void main(String[] args) { /* TODO: put the expected word inside the box */ System.out.println(new Box<String>(\"\").value); } }\n",
        EMPTY_HELLO,
        &["https://dev.java/learn/generics/"]
    ),
    lesson!(
        "java-lambda-streams",
        "java",
        "advanced",
        "Lambda and streams",
        "Lambdas pass behavior; streams process sequences.",
        "list.stream().map(x -> x + 1)",
        "import java.util.*;\nclass Solution { public static void main(String[] args) { List<String> xs = List.of(\"\"); /* TODO: stream the expected word */ xs.stream().forEach(System.out::println); } }\n",
        EMPTY_HELLO,
        &["https://docs.oracle.com/javase/tutorial/java/javaOO/lambdaexpressions.html"]
    ),
    lesson!(
        "java-records-sealed",
        "java",
        "advanced",
        "Records and sealed types",
        "Records reduce data boilerplate; sealed types bound inheritance.",
        "record Pair(int a, int b) {}",
        "record Word(String value) {}\nclass Solution { public static void main(String[] args) { /* TODO: store the expected word in the record */ System.out.println(new Word(\"\").value()); } }\n",
        EMPTY_HELLO,
        JAVA_REFS
    ),
];

const RUST_LESSONS: &[SyntaxLesson] = &[
    lesson!(
        "rust-output",
        "rust",
        "basic",
        "Output",
        "println! formats values and writes exactly one line to stdout.",
        "fn main() {\n    let score = 7;\n    println!(\"score={score}\");\n}",
        "fn main() {\n    let score = 7;\n    // TODO: print exactly score=7 using the value above\n    println!(\"TODO\");\n}\n",
        &[SyntaxCase {
            input: "",
            output: "score=7\n",
        }],
        &["https://doc.rust-lang.org/std/macro.println.html"]
    ),
    lesson!(
        "rust-variables",
        "rust",
        "basic",
        "Bindings and mutability",
        "let creates immutable bindings by default; mut makes rebinding through the same name explicit.",
        "fn main() {\n    let label = \"sum\";\n    let mut total = 1;\n    total += 2;\n    println!(\"{label}:{total}\");\n}",
        "fn main() {\n    let label = \"TODO\";\n    let mut total = 1;\n    // TODO: change total with mutation, then print sum:3\n    total += 0;\n    println!(\"{label}:{total}\");\n}\n",
        &[SyntaxCase {
            input: "",
            output: "sum:3\n",
        }],
        &["https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html"]
    ),
    lesson!(
        "rust-numbers-tuples",
        "rust",
        "basic",
        "Numbers and tuples",
        "Numeric types are explicit when inference is not enough; tuples group a fixed number of different values.",
        "fn main() {\n    let pair: (i32, i32) = (2, 3);\n    let sum = pair.0 + pair.1;\n    println!(\"{sum}\");\n}",
        "fn main() {\n    let pair: (i32, i32) = (2, 3);\n    // TODO: use both tuple fields so the output is 5\n    let sum = pair.0;\n    println!(\"{sum}\");\n}\n",
        SUM_CASE,
        &["https://doc.rust-lang.org/book/ch03-02-data-types.html"]
    ),
    lesson!(
        "rust-strings",
        "rust",
        "basic",
        "Strings",
        "String owns growable UTF-8 text; &str is a borrowed string slice into existing UTF-8 text.",
        "fn main() {\n    let mut name = String::from(\"rust\");\n    name.push_str(\"ace\");\n    let prefix: &str = &name[..4];\n    println!(\"{prefix}:{}\", name.len());\n}",
        "fn main() {\n    let mut name = String::from(\"rust\");\n    // TODO: extend the owned String, then print rust:7\n    name.push_str(\"\");\n    let prefix: &str = &name[..4];\n    println!(\"{prefix}:{}\", name.len());\n}\n",
        &[SyntaxCase {
            input: "",
            output: "rust:7\n",
        }],
        &["https://doc.rust-lang.org/book/ch04-03-slices.html"]
    ),
    lesson!(
        "rust-control-flow",
        "rust",
        "basic",
        "Control flow",
        "if can produce a value, and loop forms such as for let you turn ranges or collections into accumulated results.",
        "fn main() {\n    let n = 3;\n    let parity = if n % 2 == 0 { \"even\" } else { \"odd\" };\n    let mut total = 0;\n    for value in 1..=n {\n        total += value;\n    }\n    println!(\"{parity}:{total}\");\n}",
        "fn main() {\n    let n = 3;\n    let parity = if n % 2 == 0 { \"even\" } else { \"TODO\" };\n    let mut total = 0;\n    // TODO: include 1, 2, and 3 in the sum\n    for value in 1..n {\n        total += value;\n    }\n    println!(\"{parity}:{total}\");\n}\n",
        &[SyntaxCase {
            input: "",
            output: "odd:6\n",
        }],
        &["https://doc.rust-lang.org/book/ch03-05-control-flow.html"]
    ),
    lesson!(
        "rust-functions",
        "rust",
        "basic",
        "Functions",
        "Function signatures name parameter types and return types; the last expression can be the returned value.",
        "fn area(width: u32, height: u32) -> u32 {\n    width * height\n}\n\nfn main() {\n    println!(\"{}\", area(3, 4));\n}",
        "fn area(width: u32, height: u32) -> u32 {\n    // TODO: return rectangle area, not perimeter\n    width + height\n}\n\nfn main() {\n    println!(\"{}\", area(3, 4));\n}\n",
        &[SyntaxCase {
            input: "",
            output: "12\n",
        }],
        &["https://doc.rust-lang.org/book/ch03-03-how-functions-work.html"]
    ),
    lesson!(
        "rust-structs-impl",
        "rust",
        "basic",
        "Structs and impl",
        "A struct names related fields, and an impl block attaches methods and associated functions to that type.",
        "struct Rectangle {\n    width: u32,\n    height: u32,\n}\n\nimpl Rectangle {\n    fn area(&self) -> u32 {\n        self.width * self.height\n    }\n}\n\nfn main() {\n    let rect = Rectangle { width: 3, height: 4 };\n    println!(\"{}\", rect.area());\n}",
        "struct Rectangle {\n    width: u32,\n    height: u32,\n}\n\nimpl Rectangle {\n    fn area(&self) -> u32 {\n        // TODO: calculate from both fields\n        self.width + self.height\n    }\n}\n\nfn main() {\n    let rect = Rectangle { width: 3, height: 4 };\n    println!(\"{}\", rect.area());\n}\n",
        &[SyntaxCase {
            input: "",
            output: "12\n",
        }],
        &["https://doc.rust-lang.org/book/ch05-00-structs.html"]
    ),
    lesson!(
        "rust-enum-match",
        "rust",
        "basic",
        "Enums and match",
        "Enums model a closed set of variants, and match forces each variant to be handled deliberately.",
        "enum Command {\n    Add(i32, i32),\n    Quit,\n}\n\nfn run(command: Command) -> i32 {\n    match command {\n        Command::Add(a, b) => a + b,\n        Command::Quit => 0,\n    }\n}\n\nfn main() {\n    println!(\"{}\", run(Command::Add(2, 3)));\n}",
        "enum Command {\n    Add(i32, i32),\n    Quit,\n}\n\nfn run(command: Command) -> i32 {\n    match command {\n        // TODO: return the sum carried by Add\n        Command::Add(_a, _b) => 0,\n        Command::Quit => 0,\n    }\n}\n\nfn main() {\n    println!(\"{}\", run(Command::Add(2, 3)));\n}\n",
        SUM_CASE,
        &["https://doc.rust-lang.org/book/ch06-00-enums.html"]
    ),
    lesson!(
        "rust-option",
        "rust",
        "basic",
        "Option and if let",
        "Option<T> makes absence explicit, so code must handle Some(value) and None instead of assuming a value exists.",
        "fn first_char(text: &str) -> Option<char> {\n    text.chars().next()\n}\n\nfn main() {\n    if let Some(ch) = first_char(\"rust\") {\n        println!(\"{ch}\");\n    } else {\n        println!(\"empty\");\n    }\n}",
        "fn first_char(text: &str) -> Option<char> {\n    text.chars().next()\n}\n\nfn main() {\n    // TODO: choose input that makes Some('r') flow through if let\n    if let Some(ch) = first_char(\"\") {\n        println!(\"{ch}\");\n    } else {\n        println!(\"empty\");\n    }\n}\n",
        &[SyntaxCase {
            input: "",
            output: "r\n",
        }],
        &["https://doc.rust-lang.org/std/option/enum.Option.html"]
    ),
    lesson!(
        "rust-modules-use",
        "rust",
        "basic",
        "Modules and use",
        "mod creates a namespace, pub exposes selected items, and use brings a path into local scope without changing ownership.",
        "mod scoring {\n    pub fn label(score: u32) -> &'static str {\n        if score >= 80 { \"pass\" } else { \"retry\" }\n    }\n}\n\nuse scoring::label;\n\nfn main() {\n    println!(\"{}\", label(91));\n}",
        "mod scoring {\n    pub fn label(score: u32) -> &'static str {\n        if score >= 80 { \"pass\" } else { \"retry\" }\n    }\n}\n\nuse scoring::label;\n\nfn main() {\n    // TODO: pass a score that selects pass\n    println!(\"{}\", label(10));\n}\n",
        &[SyntaxCase {
            input: "",
            output: "pass\n",
        }],
        &[
            "https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html"
        ]
    ),
    lesson!(
        "rust-input",
        "rust",
        "intermediate",
        "Input parsing",
        "Coding-test Rust usually reads stdin as text, splits it, parses tokens once, and solves with typed values.",
        "use std::io::{self, Read};\n\nfn main() {\n    let mut input = String::new();\n    io::stdin().read_to_string(&mut input).unwrap();\n    let sum: i32 = input.split_whitespace()\n        .map(|token| token.parse::<i32>().unwrap())\n        .sum();\n    println!(\"{sum}\");\n}",
        "use std::io::{self, Read};\n\nfn main() {\n    let mut input = String::new();\n    io::stdin().read_to_string(&mut input).unwrap();\n    // TODO: parse all integers from stdin and print their sum\n    let sum = 0;\n    println!(\"{sum}\");\n}\n",
        SUM_CASE,
        &["https://doc.rust-lang.org/std/io/trait.Read.html"]
    ),
    lesson!(
        "rust-vec-hashmap",
        "rust",
        "intermediate",
        "Vec and HashMap",
        "Vec<T> stores ordered values, while HashMap<K, V> stores lookups by key; entry is the usual counting API.",
        "use std::collections::HashMap;\n\nfn main() {\n    let nums = vec![1, 2, 3];\n    let mut counts = HashMap::new();\n    for word in [\"red\", \"blue\", \"red\"] {\n        *counts.entry(word).or_insert(0) += 1;\n    }\n    println!(\"{} {}\", nums.iter().sum::<i32>(), counts[\"red\"]);\n}",
        "use std::collections::HashMap;\n\nfn main() {\n    let nums = vec![1, 2, 3];\n    let mut counts = HashMap::new();\n    for word in [\"red\", \"blue\", \"red\"] {\n        // TODO: count each word with entry(...).or_insert(...)\n        counts.insert(word, 1);\n    }\n    println!(\"{} {}\", nums.len(), counts[\"red\"]);\n}\n",
        &[SyntaxCase {
            input: "",
            output: "6 2\n",
        }],
        &[
            "https://doc.rust-lang.org/std/vec/struct.Vec.html",
            "https://doc.rust-lang.org/std/collections/struct.HashMap.html",
        ]
    ),
    lesson!(
        "rust-borrowing-slices",
        "rust",
        "intermediate",
        "Borrowing and slices",
        "Borrowed slices let functions read part of owned data without taking ownership of the whole value.",
        "fn first_word(text: &str) -> &str {\n    text.split_whitespace().next().unwrap_or(\"\")\n}\n\nfn main() {\n    let line = String::from(\"rust rules\");\n    println!(\"{}\", first_word(&line));\n}\n",
        "fn first_word(text: &str) -> &str {\n    text.split_whitespace().next().unwrap_or(\"\")\n}\n\nfn main() {\n    let line = String::from(\"rust rules\");\n    // TODO: borrow the String so first_word can read it\n    println!(\"{}\", first_word(\"\"));\n}\n",
        &[SyntaxCase {
            input: "",
            output: "rust\n",
        }],
        &["https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html"]
    ),
    lesson!(
        "rust-result",
        "rust",
        "intermediate",
        "Result and ?",
        "Result<T, E> represents recoverable failure; the ? operator unwraps Ok or returns the Err to the caller.",
        "fn parse_count(text: &str) -> Result<i32, std::num::ParseIntError> {\n    text.parse::<i32>()\n}\n\nfn main() -> Result<(), std::num::ParseIntError> {\n    let count = parse_count(\"3\")?;\n    println!(\"{}\", count + 2);\n    Ok(())\n}",
        "fn parse_count(text: &str) -> Result<i32, std::num::ParseIntError> {\n    text.parse::<i32>()\n}\n\nfn main() -> Result<(), std::num::ParseIntError> {\n    // TODO: parse 3 and use ? instead of unwrap\n    let count = parse_count(\"0\")?;\n    println!(\"{}\", count + 2);\n    Ok(())\n}\n",
        SUM_CASE,
        &["https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html"]
    ),
    lesson!(
        "rust-ownership",
        "rust",
        "advanced",
        "Ownership and borrowing",
        "Each owned value has one owner; moving transfers ownership, while borrowing lets code inspect data without taking it.",
        "fn describe(name: String) -> (String, usize) {\n    let len = name.len();\n    (name, len)\n}\n\nfn main() {\n    let name = String::from(\"rust\");\n    let (name, len) = describe(name);\n    println!(\"{name}:{len}\");\n}",
        "fn describe(name: String) -> (String, usize) {\n    let len = name.len();\n    (name, len)\n}\n\nfn main() {\n    let name = String::from(\"\");\n    // TODO: move the owned String into describe and use the returned owner\n    let (name, len) = describe(name);\n    println!(\"{name}:{len}\");\n}\n",
        &[SyntaxCase {
            input: "",
            output: "rust:4\n",
        }],
        &["https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html"]
    ),
    lesson!(
        "rust-iterators",
        "rust",
        "intermediate",
        "Iterators and closures",
        "Iterators are lazy until consumed; closures let map, filter, and fold express local transformations.",
        "fn main() {\n    let nums = [1, 2, 3, 4];\n    let total: i32 = nums.iter()\n        .filter(|n| **n % 2 == 0)\n        .map(|n| n * n)\n        .sum();\n    println!(\"{total}\");\n}",
        "fn main() {\n    let nums = [1, 2, 3, 4];\n    // TODO: square only the even numbers before summing\n    let total: i32 = nums.iter().map(|n| n).sum();\n    println!(\"{total}\");\n}\n",
        &[SyntaxCase {
            input: "",
            output: "20\n",
        }],
        &["https://doc.rust-lang.org/book/ch13-02-iterators.html"]
    ),
    lesson!(
        "rust-generics",
        "rust",
        "intermediate",
        "Generics",
        "Generics let one function or type work with many concrete types while preserving compile-time type checking.",
        "fn last_copy<T: Copy>(items: &[T]) -> Option<T> {\n    items.last().copied()\n}\n\nfn main() {\n    println!(\"{}\", last_copy(&[1, 2, 3]).unwrap());\n}",
        "fn last_copy<T: Copy>(items: &[T]) -> Option<T> {\n    // TODO: return the last copied item\n    let _ = items;\n    None\n}\n\nfn main() {\n    println!(\"{}\", last_copy(&[1, 2, 3]).unwrap_or(0));\n}\n",
        &[SyntaxCase {
            input: "",
            output: "3\n",
        }],
        &["https://doc.rust-lang.org/book/ch10-01-syntax.html"]
    ),
    lesson!(
        "rust-traits",
        "rust",
        "intermediate",
        "Traits and bounds",
        "Traits describe shared behavior, and bounds say which behavior a generic function is allowed to rely on.",
        "trait Summary {\n    fn summarize(&self) -> String;\n}\n\nstruct User {\n    name: String,\n    tasks: usize,\n}\n\nimpl Summary for User {\n    fn summarize(&self) -> String {\n        format!(\"{}: {}\", self.name, self.tasks)\n    }\n}\n\nfn print_summary<T: Summary>(item: &T) {\n    println!(\"{}\", item.summarize());\n}\n\nfn main() {\n    let user = User { name: String::from(\"Ada\"), tasks: 3 };\n    print_summary(&user);\n}",
        "trait Summary {\n    fn summarize(&self) -> String;\n}\n\nstruct User {\n    name: String,\n    tasks: usize,\n}\n\nimpl Summary for User {\n    fn summarize(&self) -> String {\n        // TODO: include both fields as Ada: 3\n        self.name.clone()\n    }\n}\n\nfn print_summary<T: Summary>(item: &T) {\n    println!(\"{}\", item.summarize());\n}\n\nfn main() {\n    let user = User { name: String::from(\"Ada\"), tasks: 3 };\n    print_summary(&user);\n}\n",
        &[SyntaxCase {
            input: "",
            output: "Ada: 3\n",
        }],
        &["https://doc.rust-lang.org/book/ch10-02-traits.html"]
    ),
    lesson!(
        "rust-lifetimes",
        "rust",
        "intermediate",
        "Lifetimes",
        "Lifetime annotations describe relationships between borrowed values; they do not make any value live longer.",
        "fn longer<'a>(left: &'a str, right: &'a str) -> &'a str {\n    if left.len() >= right.len() { left } else { right }\n}\n\nfn main() {\n    println!(\"{}\", longer(\"borrow\", \"rs\"));\n}",
        "fn longer<'a>(left: &'a str, right: &'a str) -> &'a str {\n    // TODO: return the longer borrowed string\n    let _ = left;\n    right\n}\n\nfn main() {\n    println!(\"{}\", longer(\"borrow\", \"rs\"));\n}\n",
        &[SyntaxCase {
            input: "",
            output: "borrow\n",
        }],
        &["https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html"]
    ),
    lesson!(
        "rust-traits-lifetimes",
        "rust",
        "advanced",
        "Trait objects and dyn dispatch",
        "Trait objects such as &dyn Trait allow values of different concrete types to be used through shared behavior.",
        "trait Draw {\n    fn draw(&self) -> &'static str;\n}\n\nstruct Button;\n\nimpl Draw for Button {\n    fn draw(&self) -> &'static str {\n        \"button\"\n    }\n}\n\nfn render(item: &dyn Draw) -> &'static str {\n    item.draw()\n}\n\nfn main() {\n    let button = Button;\n    println!(\"{}\", render(&button));\n}",
        "trait Draw {\n    fn draw(&self) -> &'static str;\n}\n\nstruct Button;\n\nimpl Draw for Button {\n    fn draw(&self) -> &'static str {\n        // TODO: return the label used by render\n        \"TODO\"\n    }\n}\n\nfn render(item: &dyn Draw) -> &'static str {\n    item.draw()\n}\n\nfn main() {\n    let button = Button;\n    println!(\"{}\", render(&button));\n}\n",
        &[SyntaxCase {
            input: "",
            output: "button\n",
        }],
        &["https://doc.rust-lang.org/book/ch18-02-trait-objects.html"]
    ),
    lesson!(
        "rust-testing",
        "rust",
        "intermediate",
        "Tests and assertions",
        "Rust test functions use #[test] and assertion macros; normal code still needs small pure functions that tests can call.",
        "fn add_two(n: i32) -> i32 {\n    n + 2\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn adds_two() {\n        assert_eq!(add_two(3), 5);\n    }\n}\n\nfn main() {\n    println!(\"{}\", add_two(3));\n}",
        "fn add_two(n: i32) -> i32 {\n    // TODO: make the function satisfy the test expectation\n    n\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn adds_two() {\n        assert_eq!(add_two(3), 5);\n    }\n}\n\nfn main() {\n    println!(\"{}\", add_two(3));\n}\n",
        SUM_CASE,
        &["https://doc.rust-lang.org/book/ch11-00-testing.html"]
    ),
    lesson!(
        "rust-smart-pointers",
        "rust",
        "advanced",
        "Smart pointers",
        "Smart pointers such as Box<T> own data with pointer-like behavior and can place values on the heap.",
        "fn main() {\n    let boxed = Box::new(String::from(\"heap\"));\n    println!(\"{}\", boxed.len());\n}",
        "fn main() {\n    // TODO: put heap inside Box<String> and print its length\n    let boxed = Box::new(String::from(\"\"));\n    println!(\"{}\", boxed.len());\n}\n",
        &[SyntaxCase {
            input: "",
            output: "4\n",
        }],
        &["https://doc.rust-lang.org/book/ch15-00-smart-pointers.html"]
    ),
    lesson!(
        "rust-interior-mutability",
        "rust",
        "advanced",
        "Interior mutability",
        "RefCell<T> checks borrow rules at runtime, allowing mutation through an immutable owner when the design requires it.",
        "use std::cell::RefCell;\n\nfn main() {\n    let log = RefCell::new(Vec::new());\n    log.borrow_mut().push(\"event\");\n    println!(\"{}\", log.borrow().len());\n}",
        "use std::cell::RefCell;\n\nfn main() {\n    let log: RefCell<Vec<&str>> = RefCell::new(Vec::new());\n    // TODO: borrow mutably and push one event\n    println!(\"{}\", log.borrow().len());\n}\n",
        &[SyntaxCase {
            input: "",
            output: "1\n",
        }],
        &["https://doc.rust-lang.org/book/ch15-05-interior-mutability.html"]
    ),
    lesson!(
        "rust-concurrency",
        "rust",
        "advanced",
        "Threads and join",
        "thread::spawn moves work to another OS thread, and join waits for that thread's result.",
        "use std::thread;\n\nfn main() {\n    let handle = thread::spawn(|| \"worker\");\n    println!(\"{}\", handle.join().unwrap());\n}",
        "use std::thread;\n\nfn main() {\n    let handle = thread::spawn(|| \"worker\");\n    // TODO: print the joined worker result\n    let _ = handle;\n    println!(\"main\");\n}\n",
        &[SyntaxCase {
            input: "",
            output: "worker\n",
        }],
        &["https://doc.rust-lang.org/book/ch16-01-threads.html"]
    ),
    lesson!(
        "rust-shared-state",
        "rust",
        "advanced",
        "Shared state with Arc and Mutex",
        "Arc<T> shares ownership across threads, and Mutex<T> protects mutation so only one thread edits at a time.",
        "use std::sync::{Arc, Mutex};\nuse std::thread;\n\nfn main() {\n    let count = Arc::new(Mutex::new(1));\n    let worker_count = Arc::clone(&count);\n    let handle = thread::spawn(move || {\n        *worker_count.lock().unwrap() += 1;\n    });\n    handle.join().unwrap();\n    println!(\"{}\", *count.lock().unwrap());\n}",
        "use std::sync::{Arc, Mutex};\nuse std::thread;\n\nfn main() {\n    let count = Arc::new(Mutex::new(1));\n    let worker_count = Arc::clone(&count);\n    let handle = thread::spawn(move || {\n        // TODO: lock and increment the shared count\n        let _ = worker_count;\n    });\n    handle.join().unwrap();\n    println!(\"{}\", *count.lock().unwrap());\n}\n",
        &[SyntaxCase {
            input: "",
            output: "2\n",
        }],
        &["https://doc.rust-lang.org/book/ch16-03-shared-state.html"]
    ),
    lesson!(
        "rust-async-await",
        "rust",
        "advanced",
        "Async and await",
        "async creates a Future that can pause at await points; executing futures needs a runtime or executor.",
        "async fn label() -> &'static str {\n    \"ready\"\n}\n\nfn main() {\n    let future = label();\n    drop(future);\n    println!(\"future-created\");\n}",
        "async fn label() -> &'static str {\n    \"ready\"\n}\n\nfn main() {\n    let future = label();\n    drop(future);\n    // TODO: this single-file exercise creates a Future but does not run an async runtime\n    println!(\"pending\");\n}\n",
        &[SyntaxCase {
            input: "",
            output: "future-created\n",
        }],
        &["https://doc.rust-lang.org/book/ch17-00-async-await.html"]
    ),
    lesson!(
        "rust-macros",
        "rust",
        "advanced",
        "macro_rules!",
        "macro_rules! matches token patterns at compile time and expands them into Rust code before type checking.",
        "macro_rules! greet {\n    ($name:expr) => {\n        format!(\"hi {}\", $name)\n    };\n}\n\nfn main() {\n    println!(\"{}\", greet!(\"Rust\"));\n}",
        "macro_rules! greet {\n    ($name:expr) => {\n        // TODO: expand to hi <name>\n        format!(\"TODO {}\", $name)\n    };\n}\n\nfn main() {\n    println!(\"{}\", greet!(\"Rust\"));\n}\n",
        &[SyntaxCase {
            input: "",
            output: "hi Rust\n",
        }],
        &["https://doc.rust-lang.org/book/ch20-05-macros.html"]
    ),
    lesson!(
        "rust-unsafe",
        "rust",
        "advanced",
        "Unsafe Rust",
        "unsafe enables operations the compiler cannot fully verify, but the programmer must still uphold Rust's safety rules.",
        "fn main() {\n    let value = 7;\n    let pointer = &value as *const i32;\n    let read = unsafe { *pointer };\n    println!(\"{read}\");\n}",
        "fn main() {\n    let value = 7;\n    let pointer = &value as *const i32;\n    // TODO: read the raw pointer inside an unsafe block\n    let read = 0;\n    let _ = pointer;\n    println!(\"{read}\");\n}\n",
        &[SyntaxCase {
            input: "",
            output: "7\n",
        }],
        &["https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html"]
    ),
    lesson!(
        "rust-cargo-workspaces",
        "rust",
        "advanced",
        "Cargo packages and workspaces",
        "Cargo manages packages, dependencies, tests, and workspaces; workspace commands let related crates build together.",
        "const CHECK_ALL: &str = \"cargo check --workspace\";\n\nfn main() {\n    println!(\"{CHECK_ALL}\");\n}",
        "const CHECK_ALL: &str = \"cargo check\";\n\nfn main() {\n    // TODO: print the command that checks every workspace member\n    println!(\"{CHECK_ALL}\");\n}\n",
        &[SyntaxCase {
            input: "",
            output: "cargo check --workspace\n",
        }],
        &["https://doc.rust-lang.org/cargo/reference/workspaces.html"]
    ),
];

pub fn syntax_lessons_for(language: &str) -> Vec<&'static SyntaxLesson> {
    let lessons = match normalize_language(language).as_str() {
        "ts" => TS_LESSONS,
        "java" => JAVA_LESSONS,
        "rust" => RUST_LESSONS,
        _ => PYTHON_LESSONS,
    };
    lessons.iter().collect()
}

pub fn current_syntax_lesson(state: &AppState, language: &str) -> &'static SyntaxLesson {
    let language = normalize_language(language);
    let lessons = syntax_lessons_for(&language);
    if let Some(id) = state.current_syntax_lesson.get(&language)
        && let Some(lesson) = lessons.iter().find(|lesson| lesson.id == id)
    {
        return lesson;
    }
    lessons
        .iter()
        .find(|lesson| !syntax_lesson_completed(state, &language, lesson.id))
        .copied()
        .unwrap_or(lessons[0])
}

pub fn syntax_progress_count(state: &AppState, language: &str) -> (usize, usize) {
    let language = normalize_language(language);
    (
        state
            .syntax_progress
            .get(&language)
            .map_or(0, |ids| ids.len()),
        syntax_lessons_for(&language).len(),
    )
}

pub fn syntax_lesson_completed(state: &AppState, language: &str, lesson_id: &str) -> bool {
    let language = normalize_language(language);
    state
        .syntax_progress
        .get(&language)
        .is_some_and(|ids| ids.iter().any(|id| id == lesson_id))
}

pub fn record_syntax_pass(state: &mut AppState, language: &str, lesson_id: &str) {
    let language = normalize_language(language);
    if !syntax_lessons_for(&language)
        .iter()
        .any(|lesson| lesson.id == lesson_id)
    {
        return;
    }
    let mut ids = state.syntax_progress.remove(&language).unwrap_or_default();
    if !ids.iter().any(|id| id == lesson_id) {
        ids.push(lesson_id.to_string());
    }
    state
        .syntax_progress
        .insert(language.clone(), normalize_syntax_ids_for(&language, &ids));
}

pub fn set_current_syntax_lesson(state: &mut AppState, language: &str, lesson_id: &str) {
    let language = normalize_language(language);
    if syntax_lessons_for(&language)
        .iter()
        .any(|lesson| lesson.id == lesson_id)
    {
        state
            .current_syntax_lesson
            .insert(language, lesson_id.to_string());
    }
}

pub fn next_syntax_lesson(state: &mut AppState, language: &str, direction: isize) {
    let language = normalize_language(language);
    let lessons = syntax_lessons_for(&language);
    let current = current_syntax_lesson(state, &language).id;
    let index = lessons
        .iter()
        .position(|lesson| lesson.id == current)
        .unwrap_or(0);
    let next = (index as isize + direction).clamp(0, lessons.len() as isize - 1) as usize;
    state
        .current_syntax_lesson
        .insert(language, lessons[next].id.to_string());
}

pub fn normalize_syntax_progress(
    progress: &HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    let mut normalized = HashMap::new();
    for language in LANGUAGES {
        if let Some(ids) = progress.get(*language) {
            let ids = normalize_syntax_ids_for(language, ids);
            if !ids.is_empty() {
                normalized.insert((*language).to_string(), ids);
            }
        }
    }
    normalized
}

pub fn normalize_current_syntax_lessons(
    current: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut normalized = HashMap::new();
    for language in LANGUAGES {
        if let Some(id) = current.get(*language)
            && syntax_lessons_for(language)
                .iter()
                .any(|lesson| lesson.id == id)
        {
            normalized.insert((*language).to_string(), id.clone());
        }
    }
    normalized
}

pub fn ensure_syntax_submission(root: &Path, lesson: &SyntaxLesson) -> Result<PathBuf> {
    let path = root
        .join("submissions")
        .join(".syntax")
        .join(lesson.language)
        .join(lesson.id)
        .join(format!("exercise.{}", ext_for(lesson.language)));
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, lesson.exercise.starter)?;
    }
    Ok(path)
}

pub fn syntax_cases(lesson: &SyntaxLesson) -> Vec<IoCase> {
    lesson
        .exercise
        .cases
        .iter()
        .map(|case| IoCase {
            input: case.input.to_string(),
            output: case.output.to_string(),
        })
        .collect()
}

pub fn render_syntax_lesson(lesson: &SyntaxLesson, state: &AppState) -> String {
    let ui_language = &state.settings.ui_language;
    let (done, total) = syntax_progress_count(state, lesson.language);
    let completed = if syntax_lesson_completed(state, lesson.language, lesson.id) {
        ui_text(ui_language, "syntax_complete")
    } else {
        ui_text(ui_language, "syntax_open")
    };
    let refs = lesson.refs.join("\n");
    let concept = localized_syntax_body(lesson, ui_language);
    let worked_example = localized_syntax_worked_example(lesson, ui_language);
    let common_mistakes =
        localized_syntax_list_section(lesson, ui_language, "syntax_common_mistakes", |copy| {
            &copy.common_mistakes
        });
    let self_check =
        localized_syntax_list_section(lesson, ui_language, "syntax_self_check", |copy| {
            &copy.self_check
        });
    let extra_sections = [common_mistakes, self_check]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n\n");
    let extra_sections = if extra_sections.is_empty() {
        String::new()
    } else {
        format!("\n\n{extra_sections}")
    };
    format!(
        "# {}: {}\n\n{}: {}\n{}: {}\n{}: {done}/{total} ({completed})\n\n## {}\n\n{}\n\n## {}\n\n{}\n{}\n\n## {}\n\n{}\n\n## {}\n\n{}",
        ui_text(ui_language, "syntax"),
        localized_syntax_title(lesson, ui_language),
        ui_text(ui_language, "syntax_language"),
        syntax_language_name(lesson.language),
        ui_text(ui_language, "syntax_level"),
        localized_syntax_level(lesson.level, ui_language),
        ui_text(ui_language, "syntax_progress"),
        ui_text(ui_language, "syntax_concept"),
        concept,
        ui_text(ui_language, "syntax_worked_example"),
        worked_example,
        extra_sections,
        ui_text(ui_language, "syntax_exercise"),
        localized_syntax_exercise_prompt(lesson, ui_language),
        ui_text(ui_language, "syntax_references"),
        refs
    )
}

pub fn syntax_lesson_study_context(lesson: &SyntaxLesson, ui_language: &str) -> String {
    let common_mistakes =
        localized_syntax_list_section(lesson, ui_language, "syntax_common_mistakes", |copy| {
            &copy.common_mistakes
        });
    let self_check =
        localized_syntax_list_section(lesson, ui_language, "syntax_self_check", |copy| {
            &copy.self_check
        });
    [
        format!(
            "Lesson: {} ({})",
            localized_syntax_title(lesson, ui_language),
            lesson.id
        ),
        format!("Concept:\n{}", localized_syntax_body(lesson, ui_language)),
        format!(
            "Worked example:\n{}",
            localized_syntax_worked_example(lesson, ui_language)
        ),
        common_mistakes.unwrap_or_default(),
        self_check.unwrap_or_default(),
        format!(
            "Exercise prompt:\n{}",
            localized_syntax_exercise_prompt(lesson, ui_language)
        ),
        format!("References:\n{}", lesson.refs.join("\n")),
    ]
    .into_iter()
    .filter(|section| !section.trim().is_empty())
    .collect::<Vec<_>>()
    .join("\n\n")
}

pub fn syntax_language_name(language: &str) -> &'static str {
    match normalize_language(language).as_str() {
        "ts" => "TypeScript",
        "java" => "Java",
        "rust" => "Rust",
        _ => "Python",
    }
}

fn localized_syntax_level(level: &'static str, ui_language: &str) -> &'static str {
    match level {
        "basic" => ui_text(ui_language, "syntax_basic"),
        "intermediate" => ui_text(ui_language, "syntax_intermediate"),
        "advanced" => ui_text(ui_language, "syntax_advanced"),
        _ => level,
    }
}

fn localized_syntax_exercise_prompt(lesson: &SyntaxLesson, ui_language: &str) -> String {
    required_lesson_copy_for(lesson, ui_language)
        .exercise_prompt
        .clone()
}

fn localized_syntax_title(lesson: &SyntaxLesson, ui_language: &str) -> String {
    required_lesson_copy_for(lesson, ui_language).title.clone()
}

fn localized_syntax_body(lesson: &SyntaxLesson, ui_language: &str) -> String {
    required_lesson_copy_for(lesson, ui_language)
        .concept
        .clone()
}

fn localized_syntax_worked_example(lesson: &SyntaxLesson, ui_language: &str) -> String {
    let mut text = String::new();
    text.push_str(&required_lesson_copy_for(lesson, ui_language).worked_example);
    text.push_str("\n\n");
    text.push_str(&format!("```{}\n{}\n```", lesson.language, lesson.example));
    text
}

fn localized_syntax_list_section(
    lesson: &SyntaxLesson,
    ui_language: &str,
    title_key: &str,
    items: fn(&SyntaxLessonCopy) -> &Vec<String>,
) -> Option<String> {
    let copy = required_lesson_copy_for(lesson, ui_language);
    let items = items(copy);
    if items.is_empty() {
        return None;
    }
    let body = items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n");
    Some(format!("## {}\n\n{body}", ui_text(ui_language, title_key)))
}

fn required_lesson_copy_for(lesson: &SyntaxLesson, ui_language: &str) -> &'static SyntaxLessonCopy {
    let language = normalize_language(lesson.language);
    let ui_language = normalize_ui_language(ui_language);
    let catalog = match (language.as_str(), ui_language.as_str()) {
        ("python", "ko") => PY_KO_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/python/ko.json"))),
        ("python", "ja") => PY_JA_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/python/ja.json"))),
        ("python", "zh") => PY_ZH_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/python/zh.json"))),
        ("python", "es") => PY_ES_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/python/es.json"))),
        ("python", _) => PY_EN_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/python/en.json"))),
        ("ts", "ko") => TS_KO_LESSONS.get_or_init(|| {
            load_lesson_copy(include_str!("../../assets/lessons/typescript/ko.json"))
        }),
        ("ts", "ja") => TS_JA_LESSONS.get_or_init(|| {
            load_lesson_copy(include_str!("../../assets/lessons/typescript/ja.json"))
        }),
        ("ts", "zh") => TS_ZH_LESSONS.get_or_init(|| {
            load_lesson_copy(include_str!("../../assets/lessons/typescript/zh.json"))
        }),
        ("ts", "es") => TS_ES_LESSONS.get_or_init(|| {
            load_lesson_copy(include_str!("../../assets/lessons/typescript/es.json"))
        }),
        ("ts", _) => TS_EN_LESSONS.get_or_init(|| {
            load_lesson_copy(include_str!("../../assets/lessons/typescript/en.json"))
        }),
        ("java", "ko") => JAVA_KO_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/java/ko.json"))),
        ("java", "ja") => JAVA_JA_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/java/ja.json"))),
        ("java", "zh") => JAVA_ZH_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/java/zh.json"))),
        ("java", "es") => JAVA_ES_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/java/es.json"))),
        ("java", _) => JAVA_EN_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/java/en.json"))),
        ("rust", "ko") => RUST_KO_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/rust/ko.json"))),
        ("rust", "ja") => RUST_JA_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/rust/ja.json"))),
        ("rust", "zh") => RUST_ZH_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/rust/zh.json"))),
        ("rust", "es") => RUST_ES_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/rust/es.json"))),
        ("rust", _) => RUST_EN_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/rust/en.json"))),
        _ => PY_EN_LESSONS
            .get_or_init(|| load_lesson_copy(include_str!("../../assets/lessons/python/en.json"))),
    };
    catalog.get(lesson.id).unwrap_or_else(|| {
        panic!(
            "missing lesson copy: {language}:{ui_language}:{}",
            lesson.id
        )
    })
}

fn load_lesson_copy(text: &str) -> SyntaxLessonCopyMap {
    let catalog: SyntaxLessonCatalog =
        serde_json::from_str(text).expect("valid syntax lesson copy");
    assert_eq!(
        catalog.schema_version, 1,
        "unsupported syntax lesson schema"
    );
    catalog.lessons
}

fn normalize_syntax_ids_for(language: &str, ids: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for lesson in syntax_lessons_for(language) {
        if ids.iter().any(|id| id == lesson.id) && !normalized.iter().any(|id| id == lesson.id) {
            normalized.push(lesson.id.to_string());
        }
    }
    normalized
}
