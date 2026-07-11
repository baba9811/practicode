# Commands

Type `/` outside the editor to open the command palette. Use `up/down` to move, `Enter` to run or complete the selected command, and `Esc` to cancel.

## Common

| Command | Action |
| --- | --- |
| `/home` | Return to the Continue today's session / Practice coding tests chooser |
| `/run` | Judge the current submission or syntax exercise |
| `/code` | Return to the code editor |
| `/vim` | Open the code editor (compatibility alias) |
| `/next` | In practice, open the next problem; in guided learning, advance the current step and then the queue |
| `/back` | In practice, go back through problem history; in learn mode, open the previous lesson |
| `/doctor` | Check local runtimes and show install hints |
| `/help` | Show in-app help |
| `/exit` | Quit |

## Practice

| Command | Action |
| --- | --- |
| `/problems` | Browse problems with `up/down` or `j/k`, open with `Enter` |
| `/open <id>` | Open by number, id, or slug |
| `/answer` | Show the reference answer |
| `/generate <request>` | Ask AI to create a new local problem in the background |

`/next` is local-first: it opens unsolved local problems before asking AI. If no local problem remains, `/next` may generate one in the foreground and pause editing until generation finishes.

Examples:

```text
/generate a slightly harder string problem
/generate hashmap practice, easy
/generate sorting problem, no graph yet
```

## Learning

| Command | Action |
| --- | --- |
| `/learn` | Open syntax learning |
| `/run` | Validate the current exercise |
| `/ask <question>` | Ask about the current lesson, worked example, or exercise |
| `/next` | Advance Review → Delta → Predict → Exercise → Reflect, then open the next queued lesson |
| `/back` | Open the previous lesson |
| `/lesson` | Show the complete localized lesson reference |
| `/progress` | Show a privacy-safe core/due/retained/mastered summary |

Learning sessions queue at most two due reviews before one new core lesson. `F5` runs the current exercise, `F6` cycles Lesson/Code/Result, and `F1` opens contextual help. AI-assisted capstone attempts cannot complete a course until a later unassisted pass.

## AI Help

| Command | Action |
| --- | --- |
| `/ask <question>` | In learn mode, ask about the current lesson; in practice mode, ask about the current problem and submission |
| `/hint` | Ask the selected AI for a concise hint |
| `/hint <request>` | Ask about the current problem and submission |
| `/provider codex` | Use Codex |
| `/provider claude` | Use Claude Code |
| `/model auto` | Use the provider default model |
| `/model <name>` | Use a specific provider model |
| `/effort auto` | Use the provider default effort |
| `/effort low`, `/effort medium`, `/effort high`, `/effort xhigh` | Set AI effort |

Claude also supports `/effort max`.

Invoking `/ask` or `/hint` sends the current problem or lesson, submission code, latest result, and your request to the selected provider CLI. AI-backed `/next` and `/generate` run from `PRACTICODE_HOME` with the configured provider's local permissions; they can read learning state, the problem bank, notes, indexes, and submissions and can update generated problem files. A custom `ai_next_command` has whatever access that program is granted. Review those settings before enabling AI.

## Profile And Preferences

| Command | Action |
| --- | --- |
| `/profile` | Show your current user profile |
| `/difficulty auto` | Set gradual progression |
| `/difficulty easy`, `/difficulty medium`, `/difficulty hard` | Pin future problem difficulty |
| `/topics <list>` | Set preferred topics |
| `/avoid <list>` | Set topics to avoid |
| `/generate-languages <list|all>` | Limit generated answer languages |
| `/generate-ui <list|all>` | Limit generated problem text languages |
| `/note` | Edit problem-generation notes |
| `/notes` | Show saved problem-generation notes |
| `/language python`, `/language ts`, `/language java`, `/language rust` | Set code language |
| `/ui en`, `/ui ko`, `/ui ja`, `/ui zh`, `/ui es` | Set UI language |
| `/theme dark`, `/theme light` | Set theme |
| `/update` | Show update instructions |

Inside `/profile`, use `up/down` to move and `Space` or `Enter` to cycle common settings. Use slash commands for free-form lists such as `/topics arrays, strings`.

## Aliases

Older command names such as `/vim`, `/prev`, `/previous`, `/list`, `/giveup`, `/give`, `/lang`, `/settings`, and `/quit` still work.
