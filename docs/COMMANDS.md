# Commands

Type `/` outside the editor to open the command palette. Use `up/down` to move, `Enter` to run or complete the selected command, and `Esc` to cancel.

## Common

| Command | Action |
| --- | --- |
| `/home` | Return to the Learn syntax / Practice coding tests chooser |
| `/run` | Judge the current submission or lesson drill |
| `/code` | Return to the code editor |
| `/next` | In practice, open the next problem; in learn mode, open the next lesson |
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
| `/run` | Validate the current drill |
| `/next` | Open the next lesson |
| `/back` | Open the previous lesson |

Older lesson command names such as `/drill`, `/next-lesson`, and `/prev-lesson` still work as aliases.

## AI Help

| Command | Action |
| --- | --- |
| `/hint` | Ask the selected AI for a concise hint |
| `/hint <request>` | Ask about the current problem and submission |
| `/provider codex` | Use Codex |
| `/provider claude` | Use Claude Code |
| `/model auto` | Use the provider default model |
| `/model <name>` | Use a specific provider model |
| `/effort auto` | Use the provider default effort |
| `/effort low`, `/effort medium`, `/effort high`, `/effort xhigh` | Set AI effort |

Claude also supports `/effort max`.

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

Older command names such as `/prev`, `/previous`, `/list`, `/giveup`, `/give`, `/lang`, `/settings`, and `/quit` still work.
