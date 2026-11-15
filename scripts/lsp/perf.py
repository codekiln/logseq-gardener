#!/usr/bin/env python3
"""Measure `lsg lsp` on a garden: loading, response times, reindexing and renames.

The script starts the server on the garden and speaks JSON-RPC over its standard input and
output. It reads the garden's files to choose samples and writes none of them: typed text
goes to the server as an unsaved buffer, and the answer to a rename is counted and dropped.

Usage: perf.py --lsg <binary> [--repeats N] [--samples N] [--loads N] [--record <page>] <garden>
"""

import argparse
import datetime
import json
import os
import platform
import random
import re
import statistics
import subprocess
import sys
import time
import urllib.parse
from collections import Counter

# The targets of the language server epic, in milliseconds.
TARGET_RESPONSE_P95 = 100.0
TARGET_REINDEX = 50.0
TARGET_LOAD = 5000.0

# Logseq's own keys, which the server refuses to rename.
BUILT_IN_KEYS = {
    "title", "icon", "template", "template-including-parent", "public", "filters",
    "exclude-from-graph-view", "macro", "filetags", "alias", "aliases", "tags", "id",
    "custom-id", "background-color", "heading", "collapsed", "created-at", "updated-at",
    "last-modified-at", "query-table", "query-properties", "query-sort-by",
    "query-sort-desc", "ls-type", "hl-type", "hl-page", "hl-stamp", "hl-color", "todo",
    "doing", "now", "later", "done", "card-last-interval", "card-repeats",
    "card-ease-factor", "card-next-schedule", "card-last-reviewed", "card-last-score",
}  # fmt: skip

PAGE_LINK = re.compile(r"\[\[([^\[\]\n]+)\]\]")
TAG = re.compile(r"(?:^|(?<=\s))#([\w][\w/.-]*)")
BLOCK_REFERENCE = re.compile(r"\(\(([0-9a-fA-F]{8}-[0-9a-fA-F-]{27})\)\)")
PROPERTY_KEY = re.compile(r"^[ \t]*([^\s:`]+)::(?:[ \t]|$)")

# Text typed into an unsaved buffer, with `|` at the cursor, by completion trigger.
COMPLETION_CASES = [
    ("[[ page", "- [[|"),
    ("[[ page", "- [[a|"),
    ("[[ page", "- [[lo|"),
    ("[[ page", "- [[the |"),
    ("[[ page", "- [[20|"),
    ("[[ page", "- see [[note|]]"),
    ("# tag", "- #|"),
    ("# tag", "- #c|"),
    ("# tag", "- #pro|"),
    ("(( block", "- ((|"),
    ("(( block", "- ((the|"),
    ("(( block", "- ((how to |))"),
    ("key", "- item\n  t|"),
    ("key", "- item\n  sta|"),
    ("key:: value", "- item\n  tags:: |"),
    ("key:: value", "- item\n  type:: b|"),
]


class Server:
    """One `lsg lsp` process and the JSON-RPC conversation with it."""

    def __init__(self, lsg, root_uri):
        self.started = time.perf_counter()
        self.process = subprocess.Popen(
            [lsg, "lsp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL
        )
        self.next_id = 0
        answer = self.request(
            "initialize",
            {
                "processId": os.getpid(),
                "rootUri": root_uri,
                "workspaceFolders": [{"uri": root_uri, "name": "garden"}],
                "capabilities": {
                    "general": {"positionEncodings": ["utf-8"]},
                    "workspace": {
                        # The script reports no file change, and the server starts no
                        # watcher of its own for a client that registers one.
                        "didChangeWatchedFiles": {"dynamicRegistration": True},
                        "workspaceEdit": {
                            "documentChanges": True,
                            "resourceOperations": ["rename"],
                        },
                    },
                },
            },
        )
        if "error" in answer:
            sys.exit(f"error: initialize failed: {answer['error']['message']}")
        encoding = answer["result"]["capabilities"].get("positionEncoding")
        if encoding != "utf-8":
            sys.exit(f"error: the server chose the position encoding {encoding}")
        self.notify("initialized", {})

    def send(self, message):
        body = json.dumps(message).encode()
        self.process.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
        self.process.stdin.flush()

    def read(self):
        length = 0
        while True:
            line = self.process.stdout.readline()
            if line == b"":
                sys.exit("error: lsg lsp closed its output")
            if line == b"\r\n":
                break
            if line.lower().startswith(b"content-length:"):
                length = int(line.split(b":")[1])
        return json.loads(self.process.stdout.read(length))

    def notify(self, method, params):
        self.send({"jsonrpc": "2.0", "method": method, "params": params})

    def request(self, method, params):
        """Send a request and return the whole response, answering the server's own
        requests on the way."""
        self.next_id += 1
        mine = self.next_id
        self.send({"jsonrpc": "2.0", "id": mine, "method": method, "params": params})
        while True:
            message = self.read()
            if "method" in message and "id" in message:
                self.send({"jsonrpc": "2.0", "id": message["id"], "result": None})
            elif message.get("id") == mine and "method" not in message:
                return message

    def timed(self, method, params):
        started = time.perf_counter()
        answer = self.request(method, params)
        return (time.perf_counter() - started) * 1000, answer

    def stop(self):
        self.request("shutdown", None)
        self.notify("exit", None)
        self.process.wait()


def percentile_95(values):
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int(round(0.95 * len(ordered) + 0.5)) - 1)]


def summary(values):
    return {
        "count": len(values),
        "median": statistics.median(values),
        "p95": percentile_95(values),
        "max": max(values),
    }


def garden_files(garden):
    """The Markdown files the server reads, as paths relative to the garden."""
    found = []
    for folder in ("pages", "journals"):
        for base, folders, files in os.walk(os.path.join(garden, folder)):
            folders[:] = sorted(f for f in folders if not f.startswith(".") and f != "node_modules")
            for name in sorted(files):
                path = os.path.join(base, name)
                if name.endswith(".md") and not name.startswith(".") and not os.path.islink(path):
                    found.append(os.path.relpath(path, garden))
    return found


def file_uri(garden, relative=""):
    path = os.path.join(garden, relative) if relative else garden
    return "file://" + urllib.parse.quote(path)


def scan(garden, files):
    """Where the garden writes page links, tags, block references and property keys."""
    places = {"page link": [], "tag": [], "block reference": []}
    links = Counter()
    link_place = {}
    keys = Counter()
    key_place = {}
    sizes = {}
    for relative in files:
        with open(os.path.join(garden, relative), encoding="utf-8", errors="replace") as handle:
            text = handle.read()
        sizes[relative] = len(text.encode())
        fenced = False
        for number, line in enumerate(text.split("\n")):
            if line.lstrip().startswith("```"):
                fenced = not fenced
                continue
            if fenced or "`" in line:
                continue

            def at(match, inside, line=line, number=number):
                column = len(line[: match.start() + inside].encode())
                return (relative, number, column)

            for match in PAGE_LINK.finditer(line):
                place = at(match, 3)
                places["page link"].append(place)
                name = match.group(1).strip().lower()
                links[name] += 1
                link_place.setdefault(name, place)
            for match in TAG.finditer(line):
                places["tag"].append(at(match, 1))
            for match in BLOCK_REFERENCE.finditer(line):
                places["block reference"].append(at(match, 3))
            match = PROPERTY_KEY.match(line)
            if match:
                key = match.group(1).lower().replace("_", "-")
                if key not in BUILT_IN_KEYS and not key.startswith("logseq."):
                    keys[key] += 1
                    key_place.setdefault(key, (relative, number, len(line[: match.start(1)].encode())))
    return places, links, link_place, keys, key_place, sizes


def position(place):
    return {"line": place[1], "character": place[2]}


def measure_loads(lsg, root_uri, loads):
    """Start the server `loads` times and return the timings and the last server."""
    reported, to_answer, status, server = [], [], None, None
    for _ in range(loads):
        if server:
            server.stop()
        server = Server(lsg, root_uri)
        # The server answers `initialize` at once and reads the garden before it answers
        # the next request.
        status = server.request("lsg/status", None)["result"]
        to_answer.append((time.perf_counter() - server.started) * 1000)
        reported.append(status["loadMilliseconds"])
    return server, status, reported, to_answer


def measure_navigation(server, garden, places, samples, seed):
    random.seed(seed)
    results = {}
    for kind, found in places.items():
        chosen = random.sample(found, min(samples, len(found)))
        for method, name in (
            ("textDocument/definition", "definition"),
            ("textDocument/hover", "hover"),
            ("textDocument/references", "references"),
        ):
            times, answered = [], 0
            for place in chosen:
                params = {
                    "textDocument": {"uri": file_uri(garden, place[0])},
                    "position": position(place),
                }
                if name == "references":
                    params["context"] = {"includeDeclaration": False}
                elapsed, answer = server.timed(method, params)
                times.append(elapsed)
                answered += bool(answer.get("result"))
            if times:
                results[(name, kind)] = dict(summary(times), answered=answered, times=times)
    return results


def measure_completion(server, uri, repeats):
    server.notify(
        "textDocument/didOpen",
        {"textDocument": {"uri": uri, "languageId": "markdown", "version": 1, "text": ""}},
    )
    version = 1
    groups, every = {}, []
    for group, typed in COMPLETION_CASES:
        cursor = typed.index("|")
        text = typed.replace("|", "", 1)
        before = text[:cursor]
        line = before.count("\n")
        character = len(before[before.rfind("\n") + 1 :].encode())
        version += 1
        server.notify(
            "textDocument/didChange",
            {"textDocument": {"uri": uri, "version": version}, "contentChanges": [{"text": text}]},
        )
        params = {"textDocument": {"uri": uri}, "position": {"line": line, "character": character}}
        times = []
        items = 0
        for _ in range(repeats):
            elapsed, answer = server.timed("textDocument/completion", params)
            times.append(elapsed)
            items = len((answer.get("result") or {}).get("items", []))
        entry = groups.setdefault(group, {"times": [], "items": []})
        entry["times"] += times
        entry["items"].append(items)
        every += times
    results = {group: dict(summary(entry["times"]), items=entry["items"]) for group, entry in groups.items()}
    return results, summary(every), version


def measure_reindex(server, garden, uri, version, sizes, repeats):
    """The time from a `didChange` to the answer to the request sent right after it."""
    results = {}
    baseline = [server.timed("lsg/status", None)[0] for _ in range(repeats)]
    results["lsg/status alone, for comparison"] = summary(baseline)

    # A small unsaved buffer whose whole text the editor sends again.
    times = []
    for count in range(repeats):
        version += 1
        text = f"- typing [[a page]] #tag {count}\n  type:: note\n"
        started = time.perf_counter()
        server.notify(
            "textDocument/didChange",
            {"textDocument": {"uri": uri, "version": version}, "contentChanges": [{"text": text}]},
        )
        server.request("lsg/status", None)
        times.append((time.perf_counter() - started) * 1000)
    results["a new unsaved page of two lines"] = summary(times)

    # The largest page and a page of median size, opened as a buffer with the file's text,
    # with one character typed at the end of the buffer each time.
    by_size = sorted(sizes, key=lambda name: sizes[name])
    for label, relative in (
        ("a page of median size", by_size[len(by_size) // 2]),
        ("the largest page", by_size[-1]),
    ):
        with open(os.path.join(garden, relative), encoding="utf-8", errors="replace") as handle:
            text = handle.read()
        page = file_uri(garden, relative)
        server.notify(
            "textDocument/didOpen",
            {"textDocument": {"uri": page, "languageId": "markdown", "version": 1, "text": text}},
        )
        server.request("lsg/status", None)
        times = []
        last = text.split("\n")
        line, character = len(last) - 1, len(last[-1].encode())
        for count in range(repeats):
            end = {"line": line, "character": character + count}
            started = time.perf_counter()
            server.notify(
                "textDocument/didChange",
                {
                    "textDocument": {"uri": page, "version": count + 2},
                    "contentChanges": [{"range": {"start": end, "end": end}, "text": "x"}],
                },
            )
            server.request("lsg/status", None)
            times.append((time.perf_counter() - started) * 1000)
        server.notify("textDocument/didClose", {"textDocument": {"uri": page}})
        server.request("lsg/status", None)
        results[f"{label} ({sizes[relative] / 1000:,.1f} kB), one character typed"] = summary(times)
    return results


def count_edit(answer):
    changes = (answer.get("result") or {}).get("documentChanges") or []
    texts = [change for change in changes if "kind" not in change]
    return {
        "edits": sum(len(change["edits"]) for change in texts),
        "files": len(texts),
        "renames": sum(1 for change in changes if change.get("kind") == "rename"),
        "megabytes": len(json.dumps(answer)) / 1e6,
    }


def measure_renames(server, garden, files, links, link_place, keys, key_place, repeats):
    """Ask for renames and count the answers. Nothing is applied."""
    # The most-linked names, and the name with the most namespace children among files.
    children = Counter()
    for relative in files:
        name = os.path.basename(relative)[: -len(".md")]
        if "___" in name:
            children[name.split("___")[0].lower()] += 1
    candidates = [name for name, _ in links.most_common(3)]
    for name, _ in children.most_common():
        if name in link_place:
            if name not in candidates:
                candidates.append(name)
            break
    results = []
    for kind, names, places, new_name in (
        ("page", candidates, link_place, "lsg perf renamed"),
        ("property key", [key for key, _ in keys.most_common(1)], key_place, "lsg-perf-renamed"),
    ):
        for name in names:
            place = places[name]
            params = {
                "textDocument": {"uri": file_uri(garden, place[0])},
                "position": position(place),
                "newName": new_name,
            }
            times, answer = [], None
            for _ in range(repeats):
                elapsed, answer = server.timed("textDocument/rename", params)
                times.append(elapsed)
            entry = {"kind": kind, "name": name, "median": statistics.median(times), "max": max(times)}
            if "error" in answer:
                entry["refused"] = answer["error"]["message"]
            else:
                entry.update(count_edit(answer))
            results.append(entry)
    return results


def command_output(command, cwd=None):
    try:
        return subprocess.run(
            command, cwd=cwd, capture_output=True, text=True, check=True
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return ""


def machine():
    chip = command_output(["sysctl", "-n", "machdep.cpu.brand_string"]) or platform.processor()
    return f"{chip or platform.machine()}, {platform.system()} {platform.release()}"


def ms(value):
    return f"{value:,.2f}" if value < 10 else f"{value:,.1f}" if value < 100 else f"{value:,.0f}"


def met(value, target):
    return "met" if value <= target else "missed"


def report(data):
    """The results as the outline of the docs page, without its first lines."""
    load, completion = data["load"], data["completion_all"]
    definition = data["definition_all"]
    reindex_worst = max(row["p95"] for label, row in data["reindex"].items() if "alone" not in label)
    lines = [
        "\t- ## Targets",
        "\t\t- The targets come from [Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638).",
        "\t\t- | Target | Measured | Result |",
        "\t\t  | --- | ---: | --- |",
        f"\t\t  | Loading under {TARGET_LOAD / 1000:.0f} s | {max(load['to_answer']) / 1000:.2f} s | {met(max(load['to_answer']), TARGET_LOAD)} |",
        f"\t\t  | Completion within {TARGET_RESPONSE_P95:.0f} ms at the 95th percentile | {ms(completion['p95'])} ms | {met(completion['p95'], TARGET_RESPONSE_P95)} |",
        f"\t\t  | Definition within {TARGET_RESPONSE_P95:.0f} ms at the 95th percentile | {ms(definition['p95'])} ms | {met(definition['p95'], TARGET_RESPONSE_P95)} |",
        f"\t\t  | Reindexing after an edit under {TARGET_REINDEX:.0f} ms | {ms(reindex_worst)} ms | {met(reindex_worst, TARGET_REINDEX)} |",
        "\t\t- Loading is the slowest of the measured starts, from starting the process to its answer to the first request after `initialize`. Reindexing is the highest 95th percentile among the edited pages below.",
        "\t- ## Loading",
        f"\t\t- The server read {data['status']['documents']:,} documents: {data['status']['pages']:,} pages and {data['status']['journals']:,} journals.",
        "\t\t- | Measurement | Starts | Median (ms) | Slowest (ms) |",
        "\t\t  | --- | ---: | ---: | ---: |",
        f"\t\t  | Reading and indexing, as `lsg/status` reports `loadMilliseconds` | {len(load['reported'])} | {ms(statistics.median(load['reported']))} | {ms(max(load['reported']))} |",
        f"\t\t  | From starting the process to the answer to the first request after `initialize` | {len(load['to_answer'])} | {ms(statistics.median(load['to_answer']))} | {ms(max(load['to_answer']))} |",
        "\t- ## Completion",
        "\t\t- | Typed | Requests | Median (ms) | 95th percentile (ms) | Slowest (ms) | Items per answer |",
        "\t\t  | --- | ---: | ---: | ---: | ---: | --- |",
    ]
    for group, row in data["completion"].items():
        items = f"{min(row['items'])} to {max(row['items'])}"
        lines.append(
            f"\t\t  | `{group}` | {row['count']} | {ms(row['median'])} | {ms(row['p95'])} | {ms(row['max'])} | {items} |"
        )
    lines.append(
        f"\t\t  | All | {completion['count']} | {ms(completion['median'])} | {ms(completion['p95'])} | {ms(completion['max'])} | |"
    )
    lines += [
        "\t- ## Definition, hover and references",
        "\t\t- | Request | At | Requests | Answered | Median (ms) | 95th percentile (ms) | Slowest (ms) |",
        "\t\t  | --- | --- | ---: | ---: | ---: | ---: | ---: |",
    ]
    for (name, kind), row in data["navigation"].items():
        lines.append(
            f"\t\t  | {name} | {kind} | {row['count']} | {row['answered']} | {ms(row['median'])} | {ms(row['p95'])} | {ms(row['max'])} |"
        )
    lines.append(
        f"\t\t  | definition | all three | {definition['count']} | | {ms(definition['median'])} | {ms(definition['p95'])} | {ms(definition['max'])} |"
    )
    lines += [
        "\t- ## Reindexing after an edit",
        "\t\t- The time from sending `textDocument/didChange` to receiving the answer to an `lsg/status` request sent right after it. The server handles messages in order, so the answer arrives after the index holds the edit.",
        "\t\t- | Edit | Edits | Median (ms) | 95th percentile (ms) | Slowest (ms) |",
        "\t\t  | --- | ---: | ---: | ---: | ---: |",
    ]
    for label, row in data["reindex"].items():
        lines.append(f"\t\t  | {label} | {row['count']} | {ms(row['median'])} | {ms(row['p95'])} | {ms(row['max'])} |")
    lines += [
        "\t- ## Rename",
        "\t\t- Each row is one `textDocument/rename` whose answer was counted and dropped; no edit was applied.",
        "\t\t- | Renamed | Median (ms) | Slowest (ms) | Text edits | Files edited | File renames | Answer (MB) |",
        "\t\t  | --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for row in data["renames"]:
        what = f"the {row['kind']} `{row['name']}`"
        if "refused" in row:
            lines.append(f"\t\t  | {what} | {ms(row['median'])} | {ms(row['max'])} | refused: {row['refused']} | | | |")
        else:
            lines.append(
                f"\t\t  | {what} | {ms(row['median'])} | {ms(row['max'])} | {row['edits']:,} | {row['files']:,} | {row['renames']:,} | {row['megabytes']:.2f} |"
            )
    return lines


def page(data):
    facts = data["facts"]
    lines = [
        "tags:: [[Diataxis/Reference]]",
        "see-also:: [[lsg/Ref/Command Line]], [[lsg/How To/Edit a Garden in Neovim]], [[lsg/Ref/Logseq Compatibility/Performance]]",
        "",
        "- # Language Server Performance",
        "\t- How long `lsg lsp` takes to load the encode garden, to answer completion, definition, hover and references, to reindex after an edit and to rename a page or a property key, on a release build. `mise run lsp:perf --record <garden>` writes this page.",
        f"\t- measured: {facts['date']}",
        f"\t- garden: {facts['garden']}",
        f"\t- garden commit: {facts['garden_commit']}",
        f"\t- lsg: {facts['lsg']}",
        f"\t- machine: {facts['machine']}",
        "\t- ## Method",
        "\t\t- [`scripts/lsp/perf.py`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/lsp/perf.py) starts the release build of `lsg lsp` on the garden and exchanges LSP messages with it on its standard input and output, as an editor does, with `utf-8` positions. Every time is measured in the script, from sending a request to reading its answer, so it includes writing and reading the JSON.",
        f"\t\t- Loading starts the server {facts['loads']} times. The other measurements use the last of those servers.",
        f"\t\t- Completion types each text into an unsaved buffer and asks {facts['repeats']} times at the cursor. `|` marks the cursor in the texts: {', '.join('`' + typed.replace(chr(10), ' ⏎ ') + '`' for _, typed in COMPLETION_CASES)}.",
        f"\t\t- Definition, hover and references ask once at each of up to {facts['samples']} page links, tags and block references, chosen from the garden's files with a fixed random seed. Answered counts the requests with an answer other than null or an empty list.",
        f"\t\t- Each edit is repeated {facts['repeats']} times and each rename {facts['rename_repeats']} times. The renamed pages are the three names with the most `[[links]]` and the name with the most namespace children among the page files, and the property key is the one most lines write.",
        "\t\t- The script writes no file of the garden: typed text reaches the server as buffer contents, and it applies no rename. It compares `git status` of the garden before and after the run.",
    ]
    return "\n".join(lines + report(data)) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("garden", help="garden root, the folder that holds logseq/config.edn")
    parser.add_argument("--lsg", required=True, help="the lsg binary to measure")
    parser.add_argument("--repeats", type=int, default=20, help="requests per completion text and edits per page")
    parser.add_argument("--samples", type=int, default=200, help="places per kind for definition, hover and references")
    parser.add_argument("--loads", type=int, default=5, help="times to start the server")
    parser.add_argument("--rename-repeats", type=int, default=5, help="requests per rename")
    parser.add_argument("--seed", type=int, default=644, help="seed for choosing the places")
    parser.add_argument("--record", help="write the results to this docs page")
    parser.add_argument("--json", help="write the results to this JSON file")
    args = parser.parse_args()

    garden = os.path.realpath(args.garden)
    if not os.path.isfile(os.path.join(garden, "logseq", "config.edn")):
        sys.exit(f"error: {garden} holds no logseq/config.edn")
    lsg = os.path.realpath(args.lsg)
    state = command_output(["git", "status", "--porcelain"], cwd=garden)

    files = garden_files(garden)
    places, links, link_place, keys, key_place, sizes = scan(garden, files)
    root_uri = file_uri(garden)
    print(f"Measuring {lsg} on {garden}", file=sys.stderr)
    server, status, reported, to_answer = measure_loads(lsg, root_uri, args.loads)
    # A buffer only; the file is never written.
    unsaved = file_uri(garden, "pages/lsg-perf-unsaved.md")
    completion, completion_all, version = measure_completion(server, unsaved, args.repeats)
    navigation = measure_navigation(server, garden, places, args.samples, args.seed)
    reindex = measure_reindex(server, garden, unsaved, version, sizes, args.repeats)
    server.notify("textDocument/didClose", {"textDocument": {"uri": unsaved}})
    renames = measure_renames(
        server, garden, files, links, link_place, keys, key_place, args.rename_repeats
    )
    server.stop()

    definition_times = [
        elapsed
        for (name, _), row in navigation.items()
        if name == "definition"
        for elapsed in row["times"]
    ]
    if not definition_times:
        sys.exit(f"error: {garden} writes no page link, tag or block reference to measure")
    data = {
        "facts": {
            "date": datetime.date.today().isoformat(),
            "garden": os.path.basename(garden),
            "garden_commit": command_output(["git", "rev-parse", "--short", "HEAD"], cwd=garden) or "none",
            "lsg": " ".join(
                part
                for part in (
                    command_output([lsg, "version"]).removeprefix("lsg "),
                    "(" + (command_output(["git", "rev-parse", "--short", "HEAD"]) or "unknown") + ")",
                )
                if part
            ),
            "machine": machine(),
            "loads": args.loads,
            "repeats": args.repeats,
            "rename_repeats": args.rename_repeats,
            "samples": args.samples,
        },
        "status": status,
        "load": {"reported": reported, "to_answer": to_answer},
        "completion": completion,
        "completion_all": completion_all,
        "navigation": navigation,
        "definition_all": summary(definition_times),
        "reindex": reindex,
        "renames": renames,
    }
    for key, value in data["facts"].items():
        print(f"{key.replace('_', ' ')}: {value}")
    print("\n".join(line.replace("\t", "  ") for line in report(data)))

    if command_output(["git", "status", "--porcelain"], cwd=garden) != state:
        sys.exit(f"error: git status of {garden} changed during the run")
    print("The garden's git status is the same as before the run.")
    if args.json:
        printable = dict(data, navigation={" at ".join(key): row for key, row in navigation.items()})
        with open(args.json, "w", encoding="utf-8") as handle:
            json.dump(printable, handle, indent=2)
    if args.record:
        with open(args.record, "w", encoding="utf-8") as handle:
            handle.write(page(data))
        print(f"Wrote {args.record}")


if __name__ == "__main__":
    main()
