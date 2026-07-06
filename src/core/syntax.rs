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
    "https://www.typescriptlang.org/docs/handbook/2/everyday-types.html",
    "https://www.typescriptlang.org/docs/handbook/2/narrowing.html",
    "https://nodejs.org/api/typescript.html",
];
const TS_NODE_REFS: &[&str] = &[
    "https://nodejs.org/api/typescript.html",
    "https://nodejs.org/api/fs.html#fsreadfilesyncpath-options",
    "https://nodejs.org/api/process.html#processstdout",
];
const TS_ARRAY_REFS: &[&str] = &[
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/map",
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/filter",
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/reduce",
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Loops_and_iteration",
];
const TS_TYPE_REFS: &[&str] = &[
    "https://www.typescriptlang.org/docs/handbook/2/generics.html",
    "https://www.typescriptlang.org/docs/handbook/2/keyof-types.html",
    "https://www.typescriptlang.org/docs/handbook/2/typeof-types.html",
    "https://www.typescriptlang.org/docs/handbook/2/indexed-access-types.html",
    "https://www.typescriptlang.org/docs/handbook/2/mapped-types.html",
    "https://www.typescriptlang.org/docs/handbook/2/conditional-types.html",
    "https://www.typescriptlang.org/docs/handbook/utility-types.html",
    "https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-0.html",
    "https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-4.html",
    "https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-6.html",
    "https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-9.html",
];
const JAVA_CORE_REFS: &[&str] = &[
    "https://dev.java/learn/",
    "https://docs.oracle.com/javase/tutorial/",
    "https://docs.oracle.com/javase/specs/jls/se21/html/index.html",
];
const JAVA_LANGUAGE_REFS: &[&str] = &[
    "https://dev.java/learn/",
    "https://docs.oracle.com/javase/tutorial/java/nutsandbolts/index.html",
    "https://docs.oracle.com/javase/specs/jls/se21/html/index.html",
];
const JAVA_CLASS_REFS: &[&str] = &[
    "https://dev.java/learn/",
    "https://docs.oracle.com/javase/tutorial/java/javaOO/classes.html",
    "https://docs.oracle.com/javase/specs/jls/se21/html/jls-8.html",
];
const JAVA_COLLECTION_REFS: &[&str] = &[
    "https://dev.java/learn/",
    "https://docs.oracle.com/javase/tutorial/collections/index.html",
    "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/List.html",
    "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Map.html",
    "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Set.html",
];
const JAVA_EXCEPTION_REFS: &[&str] = &[
    "https://dev.java/learn/",
    "https://docs.oracle.com/javase/tutorial/essential/exceptions/",
    "https://docs.oracle.com/javase/specs/jls/se21/html/jls-11.html",
    "https://docs.oracle.com/javase/specs/jls/se21/html/jls-14.html",
];
const JAVA_STREAM_REFS: &[&str] = &[
    "https://dev.java/learn/",
    "https://docs.oracle.com/javase/tutorial/java/javaOO/lambdaexpressions.html",
    "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/stream/Stream.html",
];
const EMPTY_HELLO: &[SyntaxCase] = &[SyntaxCase {
    input: "",
    output: "ok\n",
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
        "Console and stdout",
        "console.log appends a newline, while process.stdout.write writes exactly the bytes you give it.",
        "const score: number = 7;\nprocess.stdout.write(`score=${score}\\n`);",
        "const score: number = 7;\n// TODO: print exactly score=7 with one trailing newline\nprocess.stdout.write('TODO\\n');\n",
        &[SyntaxCase {
            input: "",
            output: "score=7\n",
        }],
        TS_NODE_REFS
    ),
    lesson!(
        "ts-let-const",
        "ts",
        "basic",
        "let and const",
        "const protects a binding from reassignment; let marks the few local values that intentionally change.",
        "const label = 'sum';\nlet total = 1;\ntotal += 2;\nconsole.log(`${label}:${total}`);",
        "const label = 'TODO';\nlet total = 1;\n// TODO: keep label stable and mutate total so the output is sum:3\ntotal += 0;\nconsole.log(`${label}:${total}`);\n",
        &[SyntaxCase {
            input: "",
            output: "sum:3\n",
        }],
        &[
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/let",
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/const",
            "https://www.typescriptlang.org/docs/handbook/2/everyday-types.html",
        ]
    ),
    lesson!(
        "ts-primitives",
        "ts",
        "basic",
        "Primitive types",
        "string, number, and boolean describe the common scalar values that most stdin parsing produces.",
        "function report(name: string, score: number, passed: boolean): string {\n  return `${name}:${score}:${passed ? 'pass' : 'retry'}`;\n}\n\nconsole.log(report('Ada', 7, true));",
        "function report(name: string, score: number, passed: boolean): string {\n  return `${name}:${score}:${passed ? 'pass' : 'retry'}`;\n}\n\n// TODO: pass the primitive values that produce Ada:7:pass\nconsole.log(report('Ada', 0, false));\n",
        &[SyntaxCase {
            input: "",
            output: "Ada:7:pass\n",
        }],
        TS_REFS
    ),
    lesson!(
        "ts-strings-templates",
        "ts",
        "basic",
        "Strings and templates",
        "Template literals keep formatting close to the values, and string methods return new strings instead of mutating text.",
        "const raw = ' Ada ';\nconst score = 7;\nconsole.log(`${raw.trim()}:${score}`);",
        "const raw = ' Ada ';\nconst score = 7;\n// TODO: trim the name and interpolate score without changing either value\nconsole.log(`${raw}:${score + 1}`);\n",
        &[SyntaxCase {
            input: "",
            output: "Ada:7\n",
        }],
        &[
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals",
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String",
            "https://www.typescriptlang.org/docs/handbook/2/everyday-types.html",
        ]
    ),
    lesson!(
        "ts-arrays-tuples",
        "ts",
        "basic",
        "Arrays and tuples",
        "number[] models a sequence of same-shaped values; a tuple fixes both position and type for small records.",
        "const scores: number[] = [2, 3];\nconst result: [string, number] = ['Ada', scores[0] + scores[1]];\nconsole.log(`${result[0]}:${result[1]}`);",
        "const scores: number[] = [2, 3];\n// TODO: put the summed score in the tuple, not the array length\nconst result: [string, number] = ['Ada', scores.length];\nconsole.log(`${result[0]}:${result[1]}`);\n",
        &[SyntaxCase {
            input: "",
            output: "Ada:5\n",
        }],
        &[
            "https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#arrays",
            "https://www.typescriptlang.org/docs/handbook/2/objects.html#tuple-types",
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array",
        ]
    ),
    lesson!(
        "ts-objects",
        "ts",
        "basic",
        "Object types",
        "Object type annotations name required fields so calculations cannot silently ignore missing properties.",
        "type Rectangle = { width: number; height: number };\nconst rect: Rectangle = { width: 3, height: 4 };\nconsole.log(rect.width * rect.height);",
        "type Rectangle = { width: number; height: number };\nconst rect: Rectangle = { width: 3, height: 4 };\n// TODO: calculate area from both required fields\nconsole.log(rect.width + rect.height);\n",
        &[SyntaxCase {
            input: "",
            output: "12\n",
        }],
        &[
            "https://www.typescriptlang.org/docs/handbook/2/objects.html",
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Working_with_objects",
        ]
    ),
    lesson!(
        "ts-functions",
        "ts",
        "basic",
        "Functions",
        "Parameter and return annotations make the input and output contract visible at the call site.",
        "function area(width: number, height: number): number {\n  return width * height;\n}\n\nconsole.log(area(3, 4));",
        "function area(width: number, height: number): number {\n  // TODO: return rectangle area, not perimeter\n  return width + height;\n}\n\nconsole.log(area(3, 4));\n",
        &[SyntaxCase {
            input: "",
            output: "12\n",
        }],
        TS_REFS
    ),
    lesson!(
        "ts-input",
        "ts",
        "intermediate",
        "Node stdin parsing",
        "In coding tests, read fd 0 once, split the text into tokens, and convert tokens before doing numeric work.",
        "const fs = require('node:fs');\nconst input: string = fs.readFileSync(0, 'utf8');\nconst nums = input.trim().split(/\\s+/).filter(Boolean).map(Number);\nconsole.log(nums.reduce((sum, n) => sum + n, 0));",
        "const fs = require('node:fs');\nconst input: string = fs.readFileSync(0, 'utf8');\n// TODO: parse all integers from stdin and print their sum\nconst nums: number[] = [];\nconsole.log(nums.reduce((sum, n) => sum + n, 0));\n",
        SUM_CASE,
        TS_NODE_REFS
    ),
    lesson!(
        "ts-control-flow",
        "ts",
        "basic",
        "Control flow",
        "if, for, while, and switch all narrow the path values can take before they reach stdout.",
        "let total = 0;\nfor (let n = 1; n <= 3; n++) {\n  if (n % 2 === 1) total += n;\n}\nconsole.log(total);",
        "let total = 0;\nfor (let n = 1; n <= 3; n++) {\n  // TODO: add only odd numbers\n  total += n;\n}\nconsole.log(total);\n",
        &[SyntaxCase {
            input: "",
            output: "4\n",
        }],
        &[
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Control_flow_and_error_handling",
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Loops_and_iteration",
            "https://www.typescriptlang.org/docs/handbook/2/narrowing.html",
        ]
    ),
    lesson!(
        "ts-union-narrowing",
        "ts",
        "intermediate",
        "Union and narrowing",
        "A union accepts several shapes, but TypeScript only allows member-specific operations after a runtime check narrows the value.",
        "function label(value: string | number): string {\n  if (typeof value === 'string') return value.toUpperCase();\n  return value.toFixed(0);\n}\n\nconsole.log(label('ok'));",
        "function label(value: string | number): string {\n  if (typeof value === 'string') return value;\n  return value.toFixed(0);\n}\n\n// TODO: preserve the union but narrow the string branch to uppercase\nconsole.log(label('ok'));\n",
        &[SyntaxCase {
            input: "",
            output: "OK\n",
        }],
        &[
            "https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#union-types",
            "https://www.typescriptlang.org/docs/handbook/2/narrowing.html",
            "https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-4.html",
        ]
    ),
    lesson!(
        "ts-literal-types",
        "ts",
        "intermediate",
        "Literal types",
        "Literal unions restrict values to exact strings or numbers, which is useful for modes, commands, and states.",
        "type Direction = 'left' | 'right';\nfunction turn(direction: Direction): string {\n  return direction === 'left' ? 'L' : 'R';\n}\n\nconsole.log(turn('left'));",
        "type Direction = 'left' | 'right';\nfunction turn(direction: Direction): string {\n  return direction === 'left' ? 'L' : 'R';\n}\n\n// TODO: choose the literal that produces L\nconsole.log(turn('right'));\n",
        &[SyntaxCase {
            input: "",
            output: "L\n",
        }],
        TS_REFS
    ),
    lesson!(
        "ts-optional-nullish",
        "ts",
        "intermediate",
        "Optional and nullish",
        "Optional properties read as possibly undefined, and ?? keeps valid falsey values such as 0 or an empty string.",
        "type User = { name: string; score?: number | null };\nconst user: User = { name: 'Ada', score: 0 };\nconsole.log(`${user.name}:${user.score ?? 10}`);",
        "type User = { name: string; score?: number | null };\nconst user: User = { name: 'Ada', score: 0 };\n// TODO: keep score 0 instead of replacing it with the fallback\nconsole.log(`${user.name}:${user.score || 10}`);\n",
        &[SyntaxCase {
            input: "",
            output: "Ada:0\n",
        }],
        &[
            "https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#optional-properties",
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Nullish_coalescing",
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Optional_chaining",
        ]
    ),
    lesson!(
        "ts-interfaces-aliases",
        "ts",
        "intermediate",
        "Interfaces and type aliases",
        "Interfaces and aliases both name object contracts; aliases also name unions, tuples, and type expressions.",
        "interface Named {\n  name: string;\n}\ntype Score = { points: number };\nfunction summary(user: Named & Score): string {\n  return `${user.name}:${user.points}`;\n}\n\nconsole.log(summary({ name: 'Ada', points: 5 }));",
        "interface Named {\n  name: string;\n}\ntype Score = { points: number };\nfunction summary(user: Named & Score): string {\n  // TODO: include both the interface field and the alias field\n  return user.name;\n}\n\nconsole.log(summary({ name: 'Ada', points: 5 }));\n",
        &[SyntaxCase {
            input: "",
            output: "Ada:5\n",
        }],
        &[
            "https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#interfaces",
            "https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#type-aliases",
            "https://www.typescriptlang.org/docs/handbook/2/objects.html",
        ]
    ),
    lesson!(
        "ts-generics",
        "ts",
        "intermediate",
        "Generics",
        "Generics let reusable functions preserve the caller's type instead of collapsing values to any.",
        "function first<T>(items: readonly T[]): T | undefined {\n  return items[0];\n}\n\nconsole.log(first(['ok', 'skip']) ?? 'none');",
        "function first<T>(items: readonly T[]): T | undefined {\n  // TODO: return the first item while preserving T\n  return items[1];\n}\n\nconsole.log(first(['ok', 'skip']) ?? 'none');\n",
        EMPTY_HELLO,
        TS_TYPE_REFS
    ),
    lesson!(
        "ts-keyof-typeof",
        "ts",
        "advanced",
        "keyof and typeof",
        "typeof captures the static type of a value, and keyof turns that object type into a union of valid keys.",
        "const limits = { small: 2, large: 5 } as const;\ntype Size = keyof typeof limits;\nfunction limitFor(size: Size): number {\n  return limits[size];\n}\n\nconsole.log(limitFor('large'));",
        "const limits = { small: 2, large: 5 } as const;\ntype Size = keyof typeof limits;\nfunction limitFor(size: Size): number {\n  return limits[size];\n}\n\n// TODO: use the key whose value is 5\nconsole.log(limitFor('small'));\n",
        SUM_CASE,
        TS_TYPE_REFS
    ),
    lesson!(
        "ts-indexed-access",
        "ts",
        "advanced",
        "Indexed access types",
        "Indexed access types read a property type from another type so value code and type code stay in sync.",
        "type User = { name: string; scores: number[] };\ntype Score = User['scores'][number];\nconst score: Score = 5;\nconsole.log(score);",
        "type User = { name: string; scores: number[] };\ntype Score = User['scores'][number];\n// TODO: assign a valid Score value that prints 5\nconst score: Score = 0;\nconsole.log(score);\n",
        SUM_CASE,
        TS_TYPE_REFS
    ),
    lesson!(
        "ts-mapped-types",
        "ts",
        "advanced",
        "Mapped types",
        "Mapped types loop over keys at the type level, often to turn one object shape into another related shape.",
        "type Flags<T> = { [K in keyof T]: boolean };\ntype Features = { search: () => void; share: () => void };\nconst enabled: Flags<Features> = { search: true, share: false };\nconsole.log(Object.entries(enabled).find(([, on]) => on)?.[0] ?? 'none');",
        "type Flags<T> = { [K in keyof T]: boolean };\ntype Features = { search: () => void; share: () => void };\nconst enabled: Flags<Features> = { search: false, share: false };\n// TODO: enable search while keeping the mapped shape\nconsole.log(Object.entries(enabled).find(([, on]) => on)?.[0] ?? 'none');\n",
        EMPTY_HELLO,
        TS_TYPE_REFS
    ),
    lesson!(
        "ts-conditional-types",
        "ts",
        "advanced",
        "Conditional types",
        "Conditional types choose one type branch from another type, and infer can capture part of a matched shape.",
        "type ElementType<T> = T extends readonly (infer Item)[] ? Item : T;\nconst word: ElementType<string[]> = 'ok';\nconsole.log(word);",
        "type ElementType<T> = T extends readonly (infer Item)[] ? Item : T;\n// TODO: assign the element type carried by string[]\nconst word: ElementType<string[]> = '';\nconsole.log(word);\n",
        EMPTY_HELLO,
        TS_TYPE_REFS
    ),
    lesson!(
        "ts-utility-types",
        "ts",
        "advanced",
        "Utility types",
        "Utility types such as Pick, Partial, Required, and Awaited express common transformations without custom aliases.",
        "type User = { id: number; name: string; score: number };\ntype UserPatch = Partial<Pick<User, 'name' | 'score'>>;\nconst patch: UserPatch = { name: 'Ada', score: 5 };\nconsole.log(`${patch.name}:${patch.score}`);",
        "type User = { id: number; name: string; score: number };\ntype UserPatch = Partial<Pick<User, 'name' | 'score'>>;\n// TODO: fill the patch fields allowed by Pick and Partial\nconst patch: UserPatch = { name: 'Ada' };\nconsole.log(`${patch.name}:${patch.score ?? 0}`);\n",
        &[SyntaxCase {
            input: "",
            output: "Ada:5\n",
        }],
        TS_TYPE_REFS
    ),
    lesson!(
        "ts-discriminated-unions",
        "ts",
        "advanced",
        "Discriminated unions",
        "A shared literal field lets switch narrow each variant and makes missing cases visible during type checking.",
        "type Shape = { kind: 'rect'; width: number; height: number } | { kind: 'circle'; radius: number };\nfunction measure(shape: Shape): number {\n  switch (shape.kind) {\n    case 'rect':\n      return shape.width * shape.height;\n    case 'circle':\n      return shape.radius * 2;\n  }\n}\n\nconsole.log(measure({ kind: 'rect', width: 3, height: 4 }));",
        "type Shape = { kind: 'rect'; width: number; height: number } | { kind: 'circle'; radius: number };\nfunction measure(shape: Shape): number {\n  switch (shape.kind) {\n    case 'rect':\n      // TODO: use the fields that only exist on the rect variant\n      return shape.width + shape.height;\n    case 'circle':\n      return shape.radius * 2;\n  }\n}\n\nconsole.log(measure({ kind: 'rect', width: 3, height: 4 }));\n",
        &[SyntaxCase {
            input: "",
            output: "12\n",
        }],
        &[
            "https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions",
            "https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-4.html",
        ]
    ),
    lesson!(
        "ts-async-promise",
        "ts",
        "intermediate",
        "Async and Promise",
        "async functions return Promise values; await unwraps the fulfilled value before later code uses it.",
        "async function double(value: number): Promise<number> {\n  return value * 2;\n}\n\nasync function main(): Promise<void> {\n  console.log(await double(2));\n}\n\nmain();",
        "async function double(value: number): Promise<number> {\n  return value * 2;\n}\n\nasync function main(): Promise<void> {\n  // TODO: await the Promise before printing its number\n  console.log(String(double(2)));\n}\n\nmain();\n",
        &[SyntaxCase {
            input: "",
            output: "4\n",
        }],
        &[
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise",
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await",
            "https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#functions-which-return-promises",
        ]
    ),
    lesson!(
        "ts-error-handling",
        "ts",
        "intermediate",
        "Error handling",
        "catch receives an unknown failure; narrow it before reading Error-specific fields or choosing a fallback.",
        "function parseCount(text: string): number {\n  try {\n    return Number.parseInt(text, 10);\n  } catch (error: unknown) {\n    return error instanceof Error ? 0 : -1;\n  }\n}\n\nconsole.log(parseCount('12'));",
        "function parseCount(text: string): number {\n  try {\n    const value = Number.parseInt(text, 10);\n    if (Number.isNaN(value)) throw new Error('bad number');\n    return value;\n  } catch (error: unknown) {\n    // TODO: narrow the caught value and recover with 12\n    return error instanceof Error ? 0 : -1;\n  }\n}\n\nconsole.log(parseCount('bad'));\n",
        &[SyntaxCase {
            input: "",
            output: "12\n",
        }],
        &[
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch",
            "https://www.typescriptlang.org/docs/handbook/2/narrowing.html",
        ]
    ),
    lesson!(
        "ts-modules",
        "ts",
        "intermediate",
        "Modules and exports",
        "import and export make file boundaries explicit; in Node type stripping, module syntax still follows Node's module rules.",
        "export function label(value: string): string {\n  return value.toUpperCase();\n}\n\nconsole.log(label('ok'));",
        "export function label(value: string): string {\n  // TODO: export behavior that callers can trust\n  return value;\n}\n\nconsole.log(label('ok'));\n",
        &[SyntaxCase {
            input: "",
            output: "OK\n",
        }],
        &[
            "https://www.typescriptlang.org/docs/handbook/2/modules.html",
            "https://nodejs.org/api/typescript.html#determining-module-system",
        ]
    ),
    lesson!(
        "ts-classes",
        "ts",
        "intermediate",
        "Classes and access modifiers",
        "Classes combine state with methods; TypeScript access modifiers describe the intended boundary for that state.",
        "class Counter {\n  private value: number;\n\n  constructor(start: number) {\n    this.value = start;\n  }\n\n  increment(): number {\n    this.value += 1;\n    return this.value;\n  }\n}\n\nconsole.log(new Counter(1).increment());",
        "class Counter {\n  private value: number;\n\n  constructor(start: number) {\n    this.value = start;\n  }\n\n  increment(): number {\n    // TODO: update private state before returning it\n    return this.value;\n  }\n}\n\nconsole.log(new Counter(1).increment());\n",
        &[SyntaxCase {
            input: "",
            output: "2\n",
        }],
        &[
            "https://www.typescriptlang.org/docs/handbook/2/classes.html",
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes",
            "https://nodejs.org/api/typescript.html#typescript-features",
        ]
    ),
    lesson!(
        "ts-readonly",
        "ts",
        "intermediate",
        "readonly",
        "readonly documents that callers may read a property or array but should not replace or mutate it through that type.",
        "type Config = { readonly name: string; readonly scores: readonly number[] };\nconst config: Config = { name: 'Ada', scores: [2, 3] };\nconsole.log(`${config.name}:${config.scores.reduce((sum, n) => sum + n, 0)}`);",
        "type Config = { readonly name: string; readonly scores: readonly number[] };\nconst config: Config = { name: 'Ada', scores: [2, 3] };\n// TODO: read from readonly data without replacing it\nconsole.log(`${config.name}:${config.scores.length}`);\n",
        &[SyntaxCase {
            input: "",
            output: "Ada:5\n",
        }],
        &[
            "https://www.typescriptlang.org/docs/handbook/2/objects.html#readonly-properties",
            "https://www.typescriptlang.org/docs/handbook/2/objects.html#the-readonlyarray-type",
        ]
    ),
    lesson!(
        "ts-satisfies-as-const",
        "ts",
        "advanced",
        "satisfies and as const",
        "as const preserves literal values, and satisfies checks a wider contract without widening the value's own type.",
        "const routes = {\n  home: '/',\n  user: '/users',\n} as const satisfies Record<string, `/${string}`>;\ntype RouteName = keyof typeof routes;\nconst selected: RouteName = 'user';\nconsole.log(routes[selected]);",
        "const routes = {\n  home: '/',\n  user: '/users',\n} as const satisfies Record<string, `/${string}`>;\ntype RouteName = keyof typeof routes;\n// TODO: choose the literal key for the users route\nconst selected: RouteName = 'home';\nconsole.log(routes[selected]);\n",
        &[SyntaxCase {
            input: "",
            output: "/users\n",
        }],
        &[
            "https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-9.html",
            "https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-0.html",
            "https://www.typescriptlang.org/docs/handbook/2/typeof-types.html",
        ]
    ),
    lesson!(
        "ts-iterables",
        "ts",
        "intermediate",
        "Iterables",
        "for...of consumes any iterable, so arrays, strings, sets, and many Node values can share loop code.",
        "const chars: Iterable<string> = ['o', 'k'];\nlet word = '';\nfor (const ch of chars) {\n  word += ch;\n}\nconsole.log(word);",
        "const chars: Iterable<string> = ['o', 'k'];\nlet word = '';\nfor (const ch of chars) {\n  // TODO: collect every yielded character\n  word = ch;\n}\nconsole.log(word);\n",
        EMPTY_HELLO,
        &[
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Iteration_protocols",
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for...of",
            "https://www.typescriptlang.org/docs/handbook/iterators-and-generators.html",
        ]
    ),
    lesson!(
        "ts-array-methods",
        "ts",
        "intermediate",
        "map, filter, and reduce",
        "map transforms each item, filter keeps selected items, and reduce folds a sequence into one accumulated value.",
        "const nums = [1, 2, 3, 4];\nconst total = nums\n  .filter((n) => n % 2 === 0)\n  .map((n) => n * n)\n  .reduce((sum, n) => sum + n, 0);\nconsole.log(total);",
        "const nums = [1, 2, 3, 4];\n// TODO: square only the even numbers before summing\nconst total = nums.reduce((sum, n) => sum + n, 0);\nconsole.log(total);\n",
        &[SyntaxCase {
            input: "",
            output: "20\n",
        }],
        TS_ARRAY_REFS
    ),
];

const JAVA_LESSONS: &[SyntaxLesson] = &[
    lesson!(
        "java-output",
        "java",
        "basic",
        "Stdout",
        "System.out.print writes text as-is, while System.out.println appends a line break that judges usually compare exactly.",
        r#"class Solution {
    public static void main(String[] args) {
        int score = 7;
        System.out.println("score=" + score);
    }
}
"#,
        r#"class Solution {
    public static void main(String[] args) {
        int score = 7;
        // TODO: print exactly score=7 with one trailing newline.
        System.out.println("TODO");
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "score=7\n",
        }],
        JAVA_CORE_REFS
    ),
    lesson!(
        "java-variables-types",
        "java",
        "basic",
        "Variables and types",
        "Local variables have declared types; primitives hold values directly and references point at objects such as String.",
        r#"class Solution {
    public static void main(String[] args) {
        String name = "Ada";
        int score = 7;
        boolean passed = score >= 5;
        System.out.println(name + ":" + score + ":" + passed);
    }
}
"#,
        r#"class Solution {
    public static void main(String[] args) {
        String name = "Ada";
        int score = 0;
        boolean passed = false;
        // TODO: update the typed values so the report matches the expected output.
        System.out.println(name + ":" + score + ":" + passed);
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "Ada:7:true\n",
        }],
        JAVA_LANGUAGE_REFS
    ),
    lesson!(
        "java-numbers-operators",
        "java",
        "basic",
        "Numbers and operators",
        "Integer division, remainder, casts, and numeric promotion decide the value before it ever reaches stdout.",
        r#"class Solution {
    public static void main(String[] args) {
        int total = 17;
        int size = 5;
        System.out.println((total / size) + ":" + (total % size));
    }
}
"#,
        r#"class Solution {
    public static void main(String[] args) {
        int total = 17;
        int size = 5;
        // TODO: use integer division and remainder so the output is 3:2.
        System.out.println((total / 2) + ":" + (total - size));
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "3:2\n",
        }],
        JAVA_LANGUAGE_REFS
    ),
    lesson!(
        "java-strings",
        "java",
        "basic",
        "Strings",
        "String is immutable; methods such as trim, substring, charAt, and equals return or compare values without changing the original.",
        r#"class Solution {
    public static void main(String[] args) {
        String raw = "  ok!  ";
        String cleaned = raw.trim().substring(0, 2);
        System.out.println(cleaned);
    }
}
"#,
        r#"class Solution {
    public static void main(String[] args) {
        String raw = "  ok!  ";
        // TODO: trim the text and keep only ok.
        String cleaned = raw.substring(0, 2);
        System.out.println(cleaned);
    }
}
"#,
        EMPTY_HELLO,
        JAVA_LANGUAGE_REFS
    ),
    lesson!(
        "java-control-flow",
        "java",
        "basic",
        "Control flow",
        "if selects a branch, loops repeat work, and each block must move the same typed values toward the final answer.",
        r#"class Solution {
    public static void main(String[] args) {
        int total = 0;
        for (int n = 1; n <= 3; n++) {
            if (n % 2 == 1) {
                total += n;
            }
        }
        System.out.println(total);
    }
}
"#,
        r#"class Solution {
    public static void main(String[] args) {
        int total = 0;
        for (int n = 1; n <= 3; n++) {
            // TODO: add only odd numbers.
            total += n;
        }
        System.out.println(total);
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "4\n",
        }],
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/javase/tutorial/java/nutsandbolts/flow.html",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-14.html",
        ]
    ),
    lesson!(
        "java-methods",
        "java",
        "basic",
        "Methods",
        "A method signature declares parameter types and a return type; return sends the computed value back to the caller.",
        r#"class Solution {
    static int area(int width, int height) {
        return width * height;
    }

    public static void main(String[] args) {
        System.out.println(area(3, 4));
    }
}
"#,
        r#"class Solution {
    static int area(int width, int height) {
        // TODO: return rectangle area, not perimeter.
        return width + height;
    }

    public static void main(String[] args) {
        System.out.println(area(3, 4));
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "12\n",
        }],
        JAVA_LANGUAGE_REFS
    ),
    lesson!(
        "java-input",
        "java",
        "intermediate",
        "Input parsing",
        "Coding-test Java usually reads System.in once, splits whitespace, then converts tokens before doing numeric work.",
        r#"import java.io.IOException;

class Solution {
    public static void main(String[] args) throws IOException {
        String input = new String(System.in.readAllBytes());
        int sum = 0;
        for (String token : input.trim().split("\\s+")) {
            if (!token.isEmpty()) {
                sum += Integer.parseInt(token);
            }
        }
        System.out.println(sum);
    }
}
"#,
        r#"import java.io.IOException;

class Solution {
    public static void main(String[] args) throws IOException {
        String input = new String(System.in.readAllBytes());
        int sum = 0;
        // TODO: split input into tokens and add every parsed integer.
        if (input.isEmpty()) {
            sum = 0;
        }
        System.out.println(sum);
    }
}
"#,
        SUM_CASE,
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/System.html#in",
            "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Scanner.html",
        ]
    ),
    lesson!(
        "java-arrays-collections",
        "java",
        "intermediate",
        "Arrays and collections",
        "Arrays keep a fixed length, while List, Map, and Set cover growable order, keyed lookup, and uniqueness.",
        r#"import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

class Solution {
    public static void main(String[] args) {
        int[] nums = {2, 3};
        List<Integer> list = new ArrayList<>();
        Map<String, Integer> totals = new HashMap<>();
        Set<Integer> seen = new HashSet<>();
        for (int n : nums) {
            list.add(n);
            totals.merge("sum", n, Integer::sum);
            seen.add(n);
        }
        System.out.println(totals.get("sum") + ":" + list.size() + ":" + seen.size());
    }
}
"#,
        r#"import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

class Solution {
    public static void main(String[] args) {
        int[] nums = {2, 3};
        List<Integer> list = new ArrayList<>();
        Map<String, Integer> totals = new HashMap<>();
        Set<Integer> seen = new HashSet<>();
        for (int n : nums) {
            list.add(n);
            // TODO: update both the Map total and Set of seen values.
        }
        System.out.println(nums.length + ":" + list.size() + ":" + seen.size());
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "5:2:2\n",
        }],
        JAVA_COLLECTION_REFS
    ),
    lesson!(
        "java-classes-objects",
        "java",
        "basic",
        "Classes and objects",
        "A class defines fields and behavior; new creates an object whose instance methods read or change that state.",
        r#"class Counter {
    int value = 3;

    int add(int delta) {
        value += delta;
        return value;
    }
}

class Solution {
    public static void main(String[] args) {
        Counter counter = new Counter();
        System.out.println(counter.add(2));
    }
}
"#,
        r#"class Counter {
    int value = 3;

    int add(int delta) {
        // TODO: change this object's state by delta before returning it.
        return value;
    }
}

class Solution {
    public static void main(String[] args) {
        Counter counter = new Counter();
        System.out.println(counter.add(2));
    }
}
"#,
        SUM_CASE,
        JAVA_CLASS_REFS
    ),
    lesson!(
        "java-constructors",
        "java",
        "basic",
        "Constructors",
        "A constructor initializes each new object before methods run; overloaded constructors provide different entry points.",
        r#"class Rectangle {
    private final int width;
    private final int height;

    Rectangle(int width, int height) {
        this.width = width;
        this.height = height;
    }

    int area() {
        return width * height;
    }
}

class Solution {
    public static void main(String[] args) {
        System.out.println(new Rectangle(3, 4).area());
    }
}
"#,
        r#"class Rectangle {
    private final int width;
    private final int height;

    Rectangle(int width, int height) {
        this.width = width;
        // TODO: store the height parameter in the field.
        this.height = 0;
    }

    int area() {
        return width * height;
    }
}

class Solution {
    public static void main(String[] args) {
        System.out.println(new Rectangle(3, 4).area());
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "12\n",
        }],
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/javase/tutorial/java/javaOO/constructors.html",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-8.html",
        ]
    ),
    lesson!(
        "java-encapsulation",
        "java",
        "basic",
        "Encapsulation",
        "private fields protect representation; public methods expose the operations callers are allowed to perform.",
        r#"class Score {
    private int points;

    void add(int delta) {
        if (delta > 0) {
            points += delta;
        }
    }

    int points() {
        return points;
    }
}

class Solution {
    public static void main(String[] args) {
        Score score = new Score();
        score.add(5);
        System.out.println(score.points());
    }
}
"#,
        r#"class Score {
    private int points;

    void add(int delta) {
        if (delta > 0) {
            // TODO: update the private field through this method.
        }
    }

    int points() {
        return points;
    }
}

class Solution {
    public static void main(String[] args) {
        Score score = new Score();
        score.add(5);
        System.out.println(score.points());
    }
}
"#,
        SUM_CASE,
        JAVA_CLASS_REFS
    ),
    lesson!(
        "java-static-members",
        "java",
        "basic",
        "Static members",
        "static fields and methods belong to the class, not one object, so they are shared through the class name.",
        r#"class Scale {
    static final int FACTOR = 3;

    static int apply(int value) {
        return value * FACTOR;
    }
}

class Solution {
    public static void main(String[] args) {
        System.out.println(Scale.apply(2));
    }
}
"#,
        r#"class Scale {
    static final int FACTOR = 3;

    static int apply(int value) {
        // TODO: use the shared FACTOR constant.
        return value;
    }
}

class Solution {
    public static void main(String[] args) {
        System.out.println(Scale.apply(2));
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "6\n",
        }],
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/javase/tutorial/java/javaOO/classvars.html",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-8.html",
        ]
    ),
    lesson!(
        "java-enum-switch",
        "java",
        "intermediate",
        "Enum and switch",
        "enum names a fixed set of constants, and switch expressions turn those constants into explicit result branches.",
        r#"enum Status {
    TODO, DONE
}

class Solution {
    static String label(Status status) {
        return switch (status) {
            case TODO -> "work";
            case DONE -> "ok";
        };
    }

    public static void main(String[] args) {
        System.out.println(label(Status.DONE));
    }
}
"#,
        r#"enum Status {
    TODO, DONE
}

class Solution {
    static String label(Status status) {
        return switch (status) {
            case TODO -> "work";
            // TODO: return ok for the DONE branch.
            case DONE -> "done";
        };
    }

    public static void main(String[] args) {
        System.out.println(label(Status.DONE));
    }
}
"#,
        EMPTY_HELLO,
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-8.html",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-14.html",
        ]
    ),
    lesson!(
        "java-exceptions",
        "java",
        "intermediate",
        "Exceptions",
        "try/catch handles recoverable failures; checked exceptions must be caught or declared in a method signature.",
        r#"class Solution {
    static int parseOrDefault(String text) {
        try {
            return Integer.parseInt(text);
        } catch (NumberFormatException error) {
            return 12;
        }
    }

    public static void main(String[] args) {
        System.out.println(parseOrDefault("bad"));
    }
}
"#,
        r#"class Solution {
    static int parseOrDefault(String text) {
        try {
            return Integer.parseInt(text);
        } catch (NumberFormatException error) {
            // TODO: recover with the expected fallback value.
            return 0;
        }
    }

    public static void main(String[] args) {
        System.out.println(parseOrDefault("bad"));
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "12\n",
        }],
        JAVA_EXCEPTION_REFS
    ),
    lesson!(
        "java-generics",
        "java",
        "intermediate",
        "Generics",
        "Generics let one class or method preserve the caller's element type instead of falling back to Object casts.",
        r#"import java.util.List;

class Solution {
    static <T> T last(List<T> items) {
        return items.get(items.size() - 1);
    }

    public static void main(String[] args) {
        System.out.println(last(List.of("skip", "ok")));
    }
}
"#,
        r#"import java.util.List;

class Solution {
    static <T> T last(List<T> items) {
        // TODO: return the last element while preserving T.
        return items.get(0);
    }

    public static void main(String[] args) {
        System.out.println(last(List.of("skip", "ok")));
    }
}
"#,
        EMPTY_HELLO,
        &[
            "https://dev.java/learn/generics/",
            "https://docs.oracle.com/javase/tutorial/java/generics/index.html",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-8.html",
        ]
    ),
    lesson!(
        "java-interfaces",
        "java",
        "intermediate",
        "Interfaces",
        "An interface names behavior a class promises to implement; callers can depend on that contract instead of the concrete class.",
        r#"interface Named {
    String name();
}

class User implements Named {
    public String name() {
        return "ok";
    }
}

class Solution {
    static String describe(Named named) {
        return named.name();
    }

    public static void main(String[] args) {
        System.out.println(describe(new User()));
    }
}
"#,
        r#"interface Named {
    String name();
}

class User implements Named {
    public String name() {
        // TODO: satisfy the interface with the expected name.
        return "";
    }
}

class Solution {
    static String describe(Named named) {
        return named.name();
    }

    public static void main(String[] args) {
        System.out.println(describe(new User()));
    }
}
"#,
        EMPTY_HELLO,
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/javase/tutorial/java/IandI/createinterface.html",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-9.html",
        ]
    ),
    lesson!(
        "java-inheritance-composition",
        "java",
        "intermediate",
        "Inheritance and composition",
        "Inheritance reuses an is-a relationship; composition keeps a helper object as a field when behavior is only a has-a dependency.",
        r#"class Bonus {
    int apply(int base) {
        return base + 2;
    }
}

class User {
    int baseScore() {
        return 3;
    }
}

class PremiumUser extends User {
    private final Bonus bonus = new Bonus();

    int score() {
        return bonus.apply(baseScore());
    }
}

class Solution {
    public static void main(String[] args) {
        System.out.println(new PremiumUser().score());
    }
}
"#,
        r#"class Bonus {
    int apply(int base) {
        return base + 2;
    }
}

class User {
    int baseScore() {
        return 3;
    }
}

class PremiumUser extends User {
    private final Bonus bonus = new Bonus();

    int score() {
        // TODO: compose Bonus with the inherited baseScore.
        return baseScore();
    }
}

class Solution {
    public static void main(String[] args) {
        System.out.println(new PremiumUser().score());
    }
}
"#,
        SUM_CASE,
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/javase/tutorial/java/IandI/subclasses.html",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-8.html",
        ]
    ),
    lesson!(
        "java-records",
        "java",
        "advanced",
        "Records",
        "A record declares an immutable data carrier and gives you a constructor, accessors, equals, hashCode, and toString.",
        r#"record Point(int x, int y) {
    int sum() {
        return x + y;
    }
}

class Solution {
    public static void main(String[] args) {
        Point point = new Point(2, 3);
        System.out.println(point.sum());
    }
}
"#,
        r#"record Point(int x, int y) {
    int sum() {
        // TODO: use both generated accessors.
        return x();
    }
}

class Solution {
    public static void main(String[] args) {
        Point point = new Point(2, 3);
        System.out.println(point.sum());
    }
}
"#,
        SUM_CASE,
        &[
            "https://dev.java/learn/records/",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-8.html",
        ]
    ),
    lesson!(
        "java-optional",
        "java",
        "intermediate",
        "Optional",
        "Optional<T> makes a maybe-present value explicit and asks the caller to map, filter, or provide a fallback.",
        r#"import java.util.Optional;

class Solution {
    public static void main(String[] args) {
        Optional<String> value = Optional.of("ok");
        String label = value.filter(text -> text.length() == 2).orElse("missing");
        System.out.println(label);
    }
}
"#,
        r#"import java.util.Optional;

class Solution {
    public static void main(String[] args) {
        Optional<String> value = Optional.empty();
        // TODO: keep ok present and filter it before the fallback.
        String label = value.filter(text -> text.length() == 2).orElse("missing");
        System.out.println(label);
    }
}
"#,
        EMPTY_HELLO,
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Optional.html",
        ]
    ),
    lesson!(
        "java-streams-lambdas",
        "java",
        "advanced",
        "Streams and lambdas",
        "A lambda supplies behavior to a pipeline, and a stream processes elements only when a terminal operation consumes it.",
        r#"import java.util.List;

class Solution {
    public static void main(String[] args) {
        int total = List.of(1, 2, 3, 4).stream()
            .filter(n -> n % 2 == 0)
            .mapToInt(n -> n * n)
            .sum();
        System.out.println(total);
    }
}
"#,
        r#"import java.util.List;

class Solution {
    public static void main(String[] args) {
        // TODO: square only the even numbers before summing.
        int total = List.of(1, 2, 3, 4).stream()
            .mapToInt(n -> n)
            .sum();
        System.out.println(total);
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "20\n",
        }],
        JAVA_STREAM_REFS
    ),
    lesson!(
        "java-comparators-sorting",
        "java",
        "intermediate",
        "Comparators and sorting",
        "Comparator objects define ordering for sorted collections and list sorting without changing the stored type.",
        r#"import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

record User(String name, int score) {}

class Solution {
    public static void main(String[] args) {
        List<User> users = new ArrayList<>(List.of(
            new User("Ada", 3),
            new User("Lin", 5),
            new User("Bo", 4)
        ));
        users.sort(Comparator.comparingInt(User::score).reversed());
        User best = users.get(0);
        System.out.println(best.name() + ":" + best.score());
    }
}
"#,
        r#"import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

record User(String name, int score) {}

class Solution {
    public static void main(String[] args) {
        List<User> users = new ArrayList<>(List.of(
            new User("Ada", 3),
            new User("Lin", 5),
            new User("Bo", 4)
        ));
        // TODO: sort by score descending, not by name.
        users.sort(Comparator.comparing(User::name));
        User best = users.get(0);
        System.out.println(best.name() + ":" + best.score());
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "Lin:5\n",
        }],
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Comparator.html",
            "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Collections.html",
        ]
    ),
    lesson!(
        "java-try-with-resources",
        "java",
        "intermediate",
        "Try-with-resources",
        "try-with-resources closes AutoCloseable values automatically after the block, even when reading or parsing fails.",
        r#"import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;

class Solution {
    public static void main(String[] args) throws IOException {
        try (ByteArrayInputStream in = new ByteArrayInputStream("ok".getBytes(StandardCharsets.UTF_8))) {
            System.out.println(new String(in.readAllBytes(), StandardCharsets.UTF_8));
        }
    }
}
"#,
        r#"import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;

class Solution {
    public static void main(String[] args) throws IOException {
        try (ByteArrayInputStream in = new ByteArrayInputStream("todo".getBytes(StandardCharsets.UTF_8))) {
            // TODO: read ok from the managed resource.
            System.out.println(new String(in.readAllBytes(), StandardCharsets.UTF_8));
        }
    }
}
"#,
        EMPTY_HELLO,
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/javase/tutorial/essential/exceptions/tryResourceClose.html",
            "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/AutoCloseable.html",
        ]
    ),
    lesson!(
        "java-packages-imports",
        "java",
        "basic",
        "Packages and imports",
        "A package names a namespace across files; in this single-file judge, imports are the practical way to use packaged JDK classes.",
        r#"import java.util.ArrayList;
import java.util.List;

class Solution {
    public static void main(String[] args) {
        List<String> words = new ArrayList<>();
        words.add("o");
        words.add("k");
        System.out.println(String.join("", words));
    }
}
"#,
        r#"import java.util.ArrayList;
import java.util.List;

class Solution {
    public static void main(String[] args) {
        List<String> words = new ArrayList<>();
        words.add("o");
        // TODO: add the second letter using the imported List implementation.
        System.out.println(String.join("", words));
    }
}
"#,
        EMPTY_HELLO,
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/javase/tutorial/java/package/index.html",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-7.html",
        ]
    ),
    lesson!(
        "java-annotations",
        "java",
        "advanced",
        "Annotations",
        "Annotations attach metadata to declarations; tools and frameworks can read them without changing normal method execution.",
        r#"@interface Audit {
    String value();
}

class Solution {
    @Audit("stdout")
    static String label() {
        return "ok";
    }

    public static void main(String[] args) {
        System.out.println(label());
    }
}
"#,
        r#"@interface Audit {
    String value();
}

class Solution {
    @Audit("stdout")
    static String label() {
        // TODO: keep the annotated method behavior correct.
        return "";
    }

    public static void main(String[] args) {
        System.out.println(label());
    }
}
"#,
        EMPTY_HELLO,
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/javase/tutorial/java/annotations/index.html",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-9.html",
        ]
    ),
    lesson!(
        "java-sealed-classes",
        "java",
        "advanced",
        "Sealed classes",
        "sealed restricts which classes or records may implement a hierarchy, making closed domain alternatives visible to readers.",
        r#"sealed interface Shape permits Rect, Dot {
    int measure();
}

record Rect(int width, int height) implements Shape {
    public int measure() {
        return width * height;
    }
}

record Dot() implements Shape {
    public int measure() {
        return 0;
    }
}

class Solution {
    public static void main(String[] args) {
        Shape shape = new Rect(3, 4);
        System.out.println(shape.measure());
    }
}
"#,
        r#"sealed interface Shape permits Rect, Dot {
    int measure();
}

record Rect(int width, int height) implements Shape {
    public int measure() {
        // TODO: compute rectangle area inside the permitted record.
        return width + height;
    }
}

record Dot() implements Shape {
    public int measure() {
        return 0;
    }
}

class Solution {
    public static void main(String[] args) {
        Shape shape = new Rect(3, 4);
        System.out.println(shape.measure());
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "12\n",
        }],
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-8.html",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-9.html",
        ]
    ),
    lesson!(
        "java-testing-assert",
        "java",
        "intermediate",
        "Testing and assert",
        "Small testable methods let assertions check behavior before main prints; AssertionError is the simplest failure signal.",
        r#"class Solution {
    static int addTwo(int value) {
        return value + 2;
    }

    static void check() {
        if (addTwo(3) != 5) {
            throw new AssertionError("addTwo should add 2");
        }
    }

    public static void main(String[] args) {
        check();
        System.out.println(addTwo(3));
    }
}
"#,
        r#"class Solution {
    static int addTwo(int value) {
        // TODO: make the method satisfy the assertion and expected output.
        return value;
    }

    static void check() {
        if (addTwo(3) != 3) {
            throw new AssertionError("current starter expectation");
        }
    }

    public static void main(String[] args) {
        check();
        System.out.println(addTwo(3));
    }
}
"#,
        SUM_CASE,
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/AssertionError.html",
            "https://docs.oracle.com/javase/tutorial/essential/exceptions/",
        ]
    ),
    lesson!(
        "java-equality-hashcode",
        "java",
        "advanced",
        "equals and hashCode",
        "Hash-based collections rely on equals and hashCode agreeing about which objects represent the same value.",
        r#"import java.util.HashSet;
import java.util.Objects;
import java.util.Set;

class Point {
    private final int x;
    private final int y;

    Point(int x, int y) {
        this.x = x;
        this.y = y;
    }

    public boolean equals(Object other) {
        if (!(other instanceof Point point)) {
            return false;
        }
        return x == point.x && y == point.y;
    }

    public int hashCode() {
        return Objects.hash(x, y);
    }
}

class Solution {
    public static void main(String[] args) {
        Set<Point> points = new HashSet<>();
        points.add(new Point(2, 3));
        points.add(new Point(2, 3));
        System.out.println(points.size());
    }
}
"#,
        r#"import java.util.HashSet;
import java.util.Objects;
import java.util.Set;

class Point {
    private final int x;
    private final int y;

    Point(int x, int y) {
        this.x = x;
        this.y = y;
    }

    public boolean equals(Object other) {
        // TODO: compare Point values by fields, not object identity.
        return this == other;
    }

    public int hashCode() {
        return Objects.hash(x, y);
    }
}

class Solution {
    public static void main(String[] args) {
        Set<Point> points = new HashSet<>();
        points.add(new Point(2, 3));
        points.add(new Point(2, 3));
        System.out.println(points.size());
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "1\n",
        }],
        JAVA_COLLECTION_REFS
    ),
    lesson!(
        "java-overloading-varargs",
        "java",
        "advanced",
        "Overloading and varargs",
        "Overloading chooses a method by argument types, while varargs gathers remaining arguments into an array parameter.",
        r#"class Solution {
    static int total(int first, int... rest) {
        int sum = first;
        for (int value : rest) {
            sum += value;
        }
        return sum;
    }

    static String label(String name, int score) {
        return name + ":" + score;
    }

    public static void main(String[] args) {
        System.out.println(label("Ada", total(2, 3)));
    }
}
"#,
        r#"class Solution {
    static int total(int first, int... rest) {
        int sum = first;
        for (int value : rest) {
            // TODO: include every varargs value.
        }
        return sum;
    }

    static String label(String name, int score) {
        return name + ":" + score;
    }

    public static void main(String[] args) {
        System.out.println(label("Ada", total(2, 3)));
    }
}
"#,
        &[SyntaxCase {
            input: "",
            output: "Ada:5\n",
        }],
        &[
            "https://dev.java/learn/",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-8.html",
            "https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html",
        ]
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
