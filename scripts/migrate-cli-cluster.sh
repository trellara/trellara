#!/usr/bin/env bash
# Migrate one flat filename-prefix cluster in trellara-cli into a real module tree.
#
#   ./scripts/migrate-cli-cluster.sh <prefix> [--apply] [--allow-dirty]
#
# Default is a dry run. Nothing is written without --apply.
#
# Why this is safe: every symbol reaches call sites through a glob re-export
# chain (module_manifest_* -> lib.rs -> crate root). Moving files into
# directories and repointing the manifest/exports layers preserves that chain
# exactly, so NO call site changes. Removing the globs is a separate later
# phase that genuinely needs the compiler.
#
# Run `cargo test --workspace` after EVERY cluster. Roll back with:
#   git checkout -- crates/trellara-cli && git clean -fd crates/trellara-cli

set -euo pipefail

PREFIX="${1:-}"
APPLY=""
ALLOW_DIRTY="false"
SRC="crates/trellara-cli/src"

for arg in "${@:2}"; do
  case "$arg" in
    --apply) APPLY="--apply" ;;
    --allow-dirty) ALLOW_DIRTY="true" ;;
    *)
      echo "unknown argument: $arg" >&2
      echo "usage: $0 <prefix> [--apply] [--allow-dirty]" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$PREFIX" ]]; then
  echo "usage: $0 <prefix> [--apply]" >&2
  echo "example: $0 quickstart --apply" >&2
  exit 2
fi
[[ -d "$SRC" ]] || { echo "error: run from the repo root (no $SRC)" >&2; exit 2; }

# Refuse to run on a dirty trellara-cli tree so rollback is always clean.
if [[ "$APPLY" == "--apply" && "$ALLOW_DIRTY" != "true" ]] && ! git diff --quiet -- "$SRC" 2>/dev/null; then
  echo "error: $SRC has uncommitted changes. Commit or stash first so you can roll back." >&2
  echo "pass --allow-dirty only when you have inspected the current diff and accepted mixed changes." >&2
  exit 1
fi

RUST_KEYWORDS="as break const continue crate dyn else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while async await box do final macro override priv try typeof unsized virtual yield"

python3 - "$PREFIX" "$APPLY" "$SRC" "$RUST_KEYWORDS" <<'PY'
import os, re, sys, shutil

prefix, apply_flag, src, keywords = sys.argv[1], sys.argv[2], sys.argv[3], set(sys.argv[4].split())
APPLY = (apply_flag == "--apply")

# ---- 1. collect the cluster (depth-1 files only; never touch tests/) ----
names = sorted(
    f[:-3] for f in os.listdir(src)
    if f.endswith(".rs") and (f[:-3] == prefix or f.startswith(prefix + "_"))
    and os.path.isfile(os.path.join(src, f))
)
if not names:
    print(f"no files matching {prefix}*.rs in {src}"); sys.exit(0)

nameset = set(names)

# ---- 2. longest-existing-prefix determines the parent, so multi-word leaf
#         names like `required_surfaces` stay a single segment ----
def segments(name, _memo={}):
    """Longest *proper* existing-prefix determines the parent, resolved recursively.
       evidence_registry_required_surfaces -> ['evidence','registry','required_surfaces']"""
    if name in _memo: return _memo[name]
    if name == prefix:
        _memo[name] = [prefix]; return _memo[name]
    parts = name.split("_")
    best = None
    for j in range(len(parts) - 1, 0, -1):        # proper prefixes only
        cand = "_".join(parts[:j])
        if cand in nameset and cand != name:
            best = cand; break
    if best is not None:
        leaf = name[len(best) + 1:]
        segs = segments(best) + [leaf]
    else:
        segs = [prefix] + ([name[len(prefix) + 1:]] if name != prefix else [])
    _memo[name] = segs
    return segs

plan = {}   # name -> destination path relative to src
for n in names:
    segs = segments(n)
    bad = [s for s in segs if s in keywords or not re.fullmatch(r"[a-z_][a-z0-9_]*", s)]
    if bad:
        print(f"SKIP {n}.rs: reserved/invalid segment {bad} — stays flat, "
              f"rename the file (e.g. override->override_policy) to include it", file=sys.stderr)
        continue
    plan[n] = os.path.join(*segs) + ".rs"

# The cluster root must be <prefix>/mod.rs, NOT <prefix>.rs.
# A #[path]-loaded module resolves children relative to its file's DIRECTORY
# (no stem subdir), so <prefix>.rs would look for src/<child>.rs. Using mod.rs
# makes the root own the directory and restores idiomatic resolution below it.
if prefix in plan:
    plan[prefix] = os.path.join(prefix, "mod.rs")

# children of each module, for generating `mod` declarations
children = {}
for n, dest in plan.items():
    if os.path.basename(dest) == "mod.rs":
        continue                       # the root declares children, it is not one
    parent = os.path.dirname(dest)
    if parent:
        children.setdefault(parent, []).append(os.path.basename(dest)[:-3])

print(f"cluster '{prefix}': {len(plan)} files")
for n in sorted(plan):
    print(f"  {n}.rs -> {plan[n]}")

if not APPLY:
    print("\n(dry run — re-run with --apply to execute)")
    sys.exit(0)

# ---- 3. move files ----
for n, dest in plan.items():
    srcp = os.path.join(src, n + ".rs")
    d = os.path.join(src, dest)
    if os.path.abspath(srcp) == os.path.abspath(d):
        continue                      # bare <prefix>.rs already sits at the cluster root
    os.makedirs(os.path.dirname(d) or src, exist_ok=True)
    shutil.move(srcp, d)

# ---- 4. prepend `pub(crate) mod <child>;` to each module that gained children ----
for parent, kids in children.items():
    pfile = (os.path.join(src, prefix, "mod.rs") if parent == prefix
             else os.path.join(src, parent + ".rs"))
    decls = "".join(f"pub(crate) mod {k};\n" for k in sorted(kids))
    if os.path.exists(pfile):
        body = open(pfile).read()
        open(pfile, "w").write(decls + "\n" + body)
    else:
        # intermediate module with no own file (e.g. the cluster root)
        os.makedirs(os.path.dirname(pfile), exist_ok=True)
        open(pfile, "w").write(decls)

# ---- 5. manifests: drop the per-file #[path]/mod pairs, add one for the root ----
touched_manifest = None
for mf in sorted(f for f in os.listdir(src) if f.startswith("module_manifest_") and f.endswith(".rs")):
    p = os.path.join(src, mf); s = orig = open(p).read()
    for n in plan:
        s = re.sub(r'#\[path = "' + re.escape(n) + r'\.rs"\]\npub\(crate\) mod ' + re.escape(n) + r';\n', '', s)
    if s != orig:
        touched_manifest = mf
        open(p, "w").write(s)

if touched_manifest:
    p = os.path.join(src, touched_manifest); s = open(p).read()
    if f'pub(crate) mod {prefix};' not in s:
        open(p, "w").write(f'#[path = "{prefix}/mod.rs"]\npub(crate) mod {prefix};\n' + s)

# ---- 6. exports_internal_*: repoint crate::flat_name -> crate::nested::path ----
for ef in sorted(f for f in os.listdir(src) if f.startswith("exports") and f.endswith(".rs")):
    p = os.path.join(src, ef); s = orig = open(p).read()
    # longest names first so prefixes don't shadow each other
    for n in sorted(plan, key=len, reverse=True):
        dest = plan[n]
        if os.path.basename(dest) == "mod.rs":
            nested = os.path.dirname(dest).replace(os.sep, "::")
        else:
            nested = dest[:-3].replace(os.sep, "::")
        s = s.replace(f"crate::{n}::", f"crate::{nested}::")
    if s != orig:
        open(p, "w").write(s)

# ---- 7. report any surviving flat module-path references ----
stale = []
for root, _, files in os.walk(src):
    if os.sep + "tests" in root: continue
    for f in files:
        if not f.endswith(".rs"): continue
        fp = os.path.join(root, f)
        txt = open(fp, errors="ignore").read()
        for n in plan:
            if f"crate::{n}::" in txt:
                stale.append(f"{fp}: crate::{n}::")

print(f"\nmoved {len(plan)} files into {prefix}/")
if stale:
    print("WARNING — stale flat module paths remain:")
    for x in sorted(set(stale)): print("  " + x)
else:
    print("no stale module paths remain")
PY

if [[ "$APPLY" == "--apply" ]]; then
  cat <<'EOF'

NEXT — verify before moving to the next cluster:
  cargo test --workspace
  cargo clippy --workspace --all-targets -- -D warnings
  cargo fmt --all

If it fails, roll back:
  git checkout -- crates/trellara-cli && git clean -fd crates/trellara-cli
EOF
fi
