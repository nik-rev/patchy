# patchy

![image](https://github.com/user-attachments/assets/95546bae-5f61-4a14-b849-c4805bbe27af)

`patchy` makes it easy to maintain personal forks in which you merge some pull requests of your liking to have more features than other people.

- [Why should I use it?](#why-should-i-use-it)
- [Usage](#usage)
  - [Config](#config)
  - [Patches](#patches)
  - [Versioning](#versioning)
- [Installation](#installation)
  - [Binary](#binary)
  - [Homebrew](#homebrew)
  - [Cargo](#cargo)
  - [PowerShell](#powershell)
  - [Nix](#nix)
- [Merge conflicts](#merge-conflicts)

## Why should I use it?

- Merge multiple pull requests and commits into a single repo effortlessly
- Sync those pull requests and the main branch with a single command
- Add new pull requests and update existing ones easily

## Usage

Go to a git repo, and initialize the config file:

```bash
patchy init
```

Invoke `patchy` by running the following command:

```bash
patchy run
```

### Config

I'm using the [Helix Editor](https://github.com/helix-editor/helix) but there are some pull requests which add awesome features.

I found myself very frequently doing the same tasks in order to sync the 4 pull requests I like to use and keep them up to date. With patchy, I just run one command and it handles the rest.

Here's my config:

```toml
# main repository to fetch from
repo = "helix-editor/helix"

# the repository's branch
remote-branch = "master"

# This is the branch where you will see all result from patchy's work. Set it to any branch you want.
# WARNING: Make sure you do not store any important work on this branch. It will be erased.
local-branch = "patchy"

# List of pull requests which you would like to merge
# TIP: Add comments above pull requests to help yourself understand which PRs do what
pull-requests = [
  # syntax highlighting for nginx files
  "12309",
  # adds file explorer
  "11285",
  # global status line
  "8908",
  # command expansions
  "11164",
]

# List of branches from other repositories to merge
# Format: "owner/repo/branch"
branches = [
  # Include master branch from another repo
  "helix-editor/helix/master",
  # Include a specific commit from a branch
  "helix-editor/helix/master @ 6049f20",
]

# An optional list of patches to apply, more on them later
patches = ["remove-tab"]
```

Running `patchy run` outputs:

![patchy output](https://github.com/user-attachments/assets/c0076588-6e57-4a80-9d05-955a4dff2580)

With this, all I will need to do is run `patchy run` and it will automatically update all of the pull requests and sync the master branch to the latest changes.

### Patches

You might want to apply some changes to your repo, but it's not a pull request. No worries! `patchy` is built for this.

Create a patch from a commit:

```bash
# obtain commit hashes e.g. from `git log`
patchy gen-patch <hash-of-commit>
```

For example, I'm running:

```bash
patchy gen-patch 7bb8ec5a77769d88855d41dd5fecfaece54cf471
```

It generated a file, `.patchy/feat-swap-light-and-dark-colors.patch`:

```patch
diff --git a/README.md m/README.md
index 11a909b2..4eae6a8d 100644
--- a/README.md
+++ m/README.md
@@ -2,8 +2,8 @@

 <h1>
 <picture>
-  <source media="(prefers-color-scheme: dark)" srcset="logo_dark.svg">
   <source media="(prefers-color-scheme: light)" srcset="logo_light.svg">
+  <source media="(prefers-color-scheme: dark)" srcset="logo_dark.svg">
   <img alt="Helix" height="128" src="logo_light.svg">
 </picture>
 </h1>
```

To use your new `.patch`, edit your `.patchy/config.toml` like so:

```diff
--- patches = []
+++ patches = [ "feat-swap-light-and-dark-colors" ]
```

### Versioning

Each pull request's branch contains commits. By default, we will always use the latest commit. However you can pin a commit to a specific version with the following syntax:

```toml
remote-branch = "main @ cfd225baedbb5fb9cbc9742f91244fa50882b580"

pull-requests = [
   "145 @ fccc58957eece10d0818dfa000bf5123e26ee32f",
   "88 @ a556aeef3736a3b6b79bb9507d26224f5c0c3449"
]

branches = [
   "helix-editor/helix/master @ 6049f20b7e3ca83f832790a0ad84d85a56205d47",
   "other-user/fork/feature-branch @ 3a56b1c"
]
```

Where the hashes represent each `sha1` hash of every commit.

This is handy if you don't want things to randomly break when some of the pull requests push a new change.

## Installation

Patchy can be installed on Linux, Windows and macOS.

### Binary

Install the binary directly into your system, available for macOS and Linux.

Recommended for Linux users.

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/nik-rev/patchy/releases/latest/download/patchy-bin-installer.sh | sh
```

### Homebrew

Recommended for macOS users.

```bash
brew install nik-rev/tap/patchy-bin
```

### Cargo

```bash
cargo install patchy-bin
```

### PowerShell

Recommended for Windows users.

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/nik-rev/patchy/releases/latest/download/patchy-installer.ps1 | iex"
```

### Nix

```bash
nix profile install github:nik-rev/patchy/main
```

<details>

<summary>
Example usage with flakes
</summary>

If the software you are using has a `flake.nix` which automatically builds this software, then using patchy with it is straightforward.

1. Use patchy to create your own remote fork.

   Let's say you fork the [`helix-editor/helix`](https://github.com/helix-editor/helix) and your fork is located at `your-username/helix`, the patchy branch is called `patchy`

1. Add your fork's input in your `flake.nix` as follows:

   ```nix
   inputs.helix.url = "github:your-username/helix/patchy";
   ```

1. Use the input in your home-manager:

   ```nix
   programs.helix.package = inputs.helix.packages.${pkgs.system}.helix;
   ```

This is easier when the target repository has a `flake.nix` which fully builds the software. Which, the [`helix-editor/helix`](https://github.com/helix-editor/helix) does have for example.

</details>

## Merge conflicts

If you merge a lot of PRs, it's likely some of them will clash with eachother and there will be conflicts.

Say you merge 10 pull requests, 3 of which couldn't be merged due to merge conflicts. The other 7 will be merged, while branches for those 3 will exist and you will be able to merge them yourself.

```text
  ✗ Could not merge branch 11164/command-expansion into the current branch for pull request #11164 Command expansion v2 since the merge is non-trivial.
You will need to merge it yourself:
  git merge --squash 11164/command-expansion
Note: To learn how to merge only once and re-use for subsequent invocations of patchy, see Merge conflicts (github)
```

<details>

<summary>

Fixing merge conflicts and retaining the fixes, declaratively

</summary>

Okay, now merge the branch:

```sh
git merge --squash 11164/command-expansion
```

We need `--squash` since `patchy gen-patch` does not work on Merge commits. But using `--squash` makes it into a non-merge commit.

Now fix the merge conflicts.

Then, commit your changes:

```sh
git commit -m "merge branch 11164/command-expansion"
```

This creates the following commit:

```text
2fb6c3c7 (HEAD -> patchy) Merge branch '11164/command-expansion' into patchy
```

Now that you have this commit, let's generate a `.patch` file for it:

```text
> patchy gen-patch 2fb6c3c7 --patch-filename=merge-11164
  ✓ Created patch file at .patchy/merge-11164.patch
```

Now, you can go ahead and add the patch to your config:

```diff
- pull-requests = [ "1234", "1111", "11164" ]
+ pull-requests = [ "1234", "1111" ]

- patches = [ ]
+ patches = [ "merge-11164" ]
```

When you do this, you won't need to solve this merge conflict anymore as the `merge-11164` patch includes both the entire pull request as well as how it was merged.

Note that if you change the _order_ of your pull requests in `pull-requests` you may see merge conflicts again. It is recommended to keep the order the same once you fix the conflicts.

</details>
