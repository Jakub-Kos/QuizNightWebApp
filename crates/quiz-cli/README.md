# quiz-cli

The theme workshop command (a port of the old `scripts/workshop.mjs`). It keeps `public/workshop/`
(`index.json` and `themes/<id>/theme.json` with its files) and checks theme packs submitted as
GitHub issues. Every theme goes through `normalize_theme` from quiz-core, the same check as an import
in the app, and must pass it without dropping anything.

## Usage

Run it from the repository root (the workshop folder is `public/workshop` below the current folder),
or point it elsewhere with `--dir`:

```sh
quiz-cli index                         # check every theme, write index.json (fails if one does not pass)
quiz-cli add <zip or URL> [--replace]  # add a reviewed pack; --replace overwrites a theme with that id
quiz-cli check <zip or URL>            # print the Markdown review report, add nothing
quiz-cli check --from-issue            # the same for the .zip attached to $ISSUE_BODY
quiz-cli --dir /tmp/workshop index     # another workshop folder (not in the Node script)
```

From the repository root without installing it:

```sh
cargo run -q --release -p quiz-cli -- index
```

Arguments are read as the Node script reads them: the pack is the first argument that does not
start with `--`, flags may come before or after it, unknown flags are ignored, and an unknown
command prints the usage line and exits with 1. Messages, exit codes and written files are the
Node script's (`✓ …` on stdout, `! …` warnings and `✗ …` errors on stderr, exit code 1 on `✗`);
`check` always exits with 0 and prints its report on stdout.

## Where it differs from the Node script

On purpose, for safety:

- `add` refuses a pack whose theme refers to a file outside the theme's folder (`../x.png`,
  `/etc/x.png`), before writing anything:
  `✗ the theme refers to a file outside its folder (../x.png); fix the pack before adding it`.
  The Node script writes such a file wherever the path leads, then fails the index check and leaves
  a broken theme folder behind. The review report (`check`) is unchanged and still passes such a
  pack, as in Node.
- A zip entry that unpacks to more than its declared size is refused as "this is not a .zip file"
  (the zip reader stops at the declared size). fflate cuts such an entry to its declared size, after
  inflating all of it, and takes stored entries whole whatever they declare.
- Entries whose CRC does not match are refused; fflate does not check CRCs.

Not matched:

- Error text from inside JSON parsing (`theme.json cannot be read (…)`) is serde_json's, not V8's.
- Rare filesystem errors are worded differently; missing files and permissions are worded as Node
  words them.
- Network errors other than a refused connection, too many redirects, a bad URL or a cut-off body
  are all "TypeError: fetch failed"; neither tool has a timeout.
