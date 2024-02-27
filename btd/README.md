# Buck Target Determinator

Given a set of file changes, map that to a set of Buck target changes, including recursive dependencies. E.g. if `fbcode/project/file.c` changes, BTD might report that `fbcode//project:library` changed directly and `fbcode//project:binary` depends on that. BTD requires both the list of changed files, and information from Buck about the targets before/after. BTD is available as a dotslash binary at `tools/utd/btd/btd` in the repo.

As an example:

```shell
btd --cells ~/data/cells.json --changes ~/data/changes.txt --base ~/data/fbcode_base.jsonl --diff ~/data/fbcode_diff.jsonl
```

Or to have BTD run Buck2 itself:

```shell
btd --cells ~/data/cells.json --changes ~/data/changes.txt --base ~/data/fbcode_base.jsonl --universe fbcode//...
```

Where:

* `btd` is the binary. Within Meta a precompiled version of `btd` is available at `~/fbsource/tools/utd/btd/btd`.
* `cells.json` is the output of `buck2 audit cell --json` in the root of the repo.
* `changes.txt` is the output of `hg status --rev hash_before::hash_after -amr --root-relative`.
* `fbcode_base.jsonl` is the output of `supertd targets fbcode//... --output fbcode_base.jsonl` in the base state, before the changes. Pass `--dry-run` to see the `buck2` command that is equivalent to.
* `fbcode_diff.jsonl` is the output of that above command run on the diff state, after the changes.

BTD reports the list of changed targets at level 0 (immediate impact), and increasing levels (dependencies of something in a lower level).

## When to use BTD

BTD is considered a reusable tool, albeit one tailored to the needs of target determinator workflows. You are welcome to reuse BTD, but within Meta, its usually best to drop a message on the [Target Determinator group](https://fb.workplace.com/groups/targetdeterminator) first.

To use BTD, you need to have a set of changed files, and care about how that maps to Buck targets. If you don't have a set of changed files and just want to understand the entire graph, then using either `buck2 query` or `buck2 targets` (perhaps with the flags listed above) is often a better choice. If you don't care about the impacted targets, then the output won't be much use to you. If both those things do apply, BTD is probably a good choice, perhaps with some filtering after (e.g. to select only targets of a particular type or with a particular label).

## Decisions

Most of BTD is "obvious", but there are a few places where there are legitimate choices as to what should be done:

* **Error handling**: If a package gives an error in the base state and the diff state, but the error message is different, we consider it the same error. The rationale is that error messages are not always 100% deterministic.

## BTD at Meta

BTD is used as part of the target determinator workflow, for selecting which tests to run on a diff.

* If you want to understand why target determinator didn't run, see [this wiki page](https://www.internalfb.com/intern/wiki/Fbcode_CI/FAQ:_Frequently_Asked_Questions/Why_didn't_my_job_build_test_run_on_D_(I'm_sad_trunk_broke)/).
* If you want to run BTD approximately how it might have been run on a given commit (e.g. running `hg` and `buck2` for you), try [`run_btd_on_this_commit.sh`](https://www.internalfb.com/code/fbsource/tools/utd/btd/run_btd_on_this_commit.sh).

## Caching

The output of BTD is deterministic. If you pass both `--base` and `--diff` then BTD won't ask the system for any information and the output will be entirely derived from the inputs you pass. If you omit `--diff`, then BTD will invoke `buck2` equivalently to the `supertd targets` command outlined above.

The `buck2 targets` command can be cached between revisions if none of the following files have changed.

1. `BUCK`, `BUCK.v2`, `TARGETS` or `TARGETS.v2` files.
2. `PACKAGE` files.
3. Files with the `.bzl` extension (but can exclude `.td.bzl` files).
4. Files with the `.bcfg` and `.buckconfig` extensions.
5. Files in `**/mode/**` or `**/buckconfigs/**`.