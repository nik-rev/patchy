# v2.0.0

**Features**

- You can now pass `--use-gh-cli` flag and it will use the [`gh`](https://github.com/cli/cli) CLI. This lets you avoid "Rate limit" errors if you authenticate.

**Breaking**

I've rewritten the CLI in [Clap](https://github.com/clap-rs/clap) instead of a hand-written parser.

This has 2 advantages:

- Easier to maintain. The CLI is now declarative. Help is automatically generated. In total we went from about 3,000 lines of code (mostly tests) to just under 200 for the command-line parsing logic.
- More intuitive. The previous CLI interface was nothing like you'd find in any other command-line app.
- `patchy completions` subcommand generates completion for any shell

And it implies that you can't do stuff like fetch more than 1 PR or Branch using patchy. I don't think people used this much. However, if you'd like to do it you can just invoke `patchy` more than once.

# v1.3.0 - 2024-01-29

- Added new `patchy branch-fetch` subcommand, allows fetching GitHub branches locally. Usage:

```
Usage:

  patchy branch-fetch [<args>] [<flags>]
  » Fetch remote branches into a local branch

Examples:

  patchy branch-fetch helix-editor/helix/master
  » Fetch a single branch

  patchy branch-fetch 'helix-editor/helix/master@6049f20'
  » Fetch a single branch at a certain commit
```

- Using a `pr-fetch` with no arguments will bring up the help menu now.

# Patchy v1.2.7

- Improved error message
- Create sub-directories if they do not exist
- Fix issue where we tried to create directory when it already existed, so it would abort the entire program

# Patchy v1.2.6

- Improved error messages

# Patchy v1.2.2 - v1.2.5

- Fixes download links as github username of author changed

# Patchy v1.2.1

- Fixes `patchy pr-fetch` not working on pull requests merged by patchy

# Patchy v1.2.0

- Use `redirect.github.com` instead of `github.com` not to spam PRs
- Use more readable branch names
- Support pull requests in command-line and config starting with `#`
- Handle when no arguments were passed to gen-patch
- Improved error handling
- Add `--yes` and `-y` flags to `patchy run`
- Fixed `gen-patch` subcommand

# Patchy v1.1.5

- Performance improvements
