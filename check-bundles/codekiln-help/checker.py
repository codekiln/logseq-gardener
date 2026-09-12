#!/usr/bin/env python3
"""Checker CLI for the codekiln-help bundle."""

from __future__ import annotations

from collections import deque
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from typing import Any


FORMAT_VERSION = 1
METHOD = "mechanistic"
COMMAND_COUNT_LIMIT = 64
COMMAND_DEPTH_LIMIT = 8
DOCUMENT_BYTES_LIMIT = 1_048_576
TOTAL_COMMANDS_LIMIT = 1_024
COMMAND_TIMEOUT_SECONDS = 2


class HelpChecker:
    def __init__(self, request: dict[str, Any]) -> None:
        self.request = request
        self.target = [str(part) for part in request["target"]]
        self.messages: list[dict[str, Any]] = []
        self.expectations = 0
        self.commands_run = 0
        self.total_limit_reported = False
        self.hierarchy_limit_exceeded = False
        self.outlines: dict[tuple[str, ...], list[dict[str, Any]]] = {}

    def check(self) -> dict[str, Any]:
        paths = self.discover_paths()
        if not self.hierarchy_limit_exceeded:
            for path in paths:
                self.check_path(path)
        failures = len(self.messages)
        score = 4.0 if failures == 0 else max(
            0.0, 4.0 * (self.expectations - failures) / max(self.expectations, 1)
        )
        return {
            "format_version": FORMAT_VERSION,
            "request_id": self.request["request_id"],
            "check": self.request["check"],
            "method": METHOD,
            "outcome": "result",
            "result": {
                "score": score,
                "messages": self.messages,
            },
        }

    def discover_paths(self) -> list[tuple[str, ...]]:
        discovered: list[tuple[str, ...]] = []
        queue: deque[tuple[str, ...]] = deque([()])
        seen: set[tuple[str, ...]] = set()
        while queue:
            path = queue.popleft()
            if path in seen:
                continue
            if len(path) > COMMAND_DEPTH_LIMIT:
                self.hierarchy_limit_exceeded = True
                self.fail(
                    "Command hierarchy exceeds the depth limit.",
                    path,
                    {"limit": COMMAND_DEPTH_LIMIT},
                )
                continue
            if len(seen) >= COMMAND_COUNT_LIMIT:
                self.hierarchy_limit_exceeded = True
                self.fail(
                    "Command hierarchy exceeds the command-count limit.",
                    path,
                    {"limit": COMMAND_COUNT_LIMIT},
                )
                break
            seen.add(path)
            discovered.append(path)
            response = self.run_json(
                path,
                ["help", "--format", "json"],
                "command discovery",
            )
            if response is None:
                continue
            if not self.validate_base_response(response, path, False, "command discovery"):
                continue
            children = response.get("child_commands")
            self.expectations += 1
            if not isinstance(children, list):
                self.fail(
                    "JSON help must contain a child_commands array.",
                    path,
                    {"response": response},
                )
                continue
            for child in children:
                name = child.get("name") if isinstance(child, dict) else None
                if not isinstance(name, str) or not name or name.startswith("-"):
                    self.fail(
                        "A discovered child command has an invalid name.",
                        path,
                        {"child": child},
                    )
                    continue
                queue.append((*path, name))
        return discovered

    def check_path(self, path: tuple[str, ...]) -> None:
        default_help = self.run(path, ["help"], "default help")
        option_help = self.run(path, ["--help"], "--help")
        self.expect(
            default_help["ok"] and bool(default_help["stdout"].strip()),
            "The help command must succeed and write help to standard output.",
            path,
            default_help,
        )
        self.expect(
            option_help["ok"] and option_help["stdout"] == default_help["stdout"],
            "The help command and --help must describe the same command.",
            path,
            {"help": default_help, "option_help": option_help},
        )

        programmatic = self.run(
            path,
            ["help", "--programmatic"],
            "programmatic help",
        )
        programmatic_text = programmatic["stdout"].lower()
        guidance = (
            default_help["stdout"] in programmatic["stdout"]
            and ("pipe" in programmatic_text or "redirect" in programmatic_text)
            and ("json" in programmatic_text or "--format" in programmatic_text)
            and "section" in programmatic_text
            and (
                "pager" in programmatic_text
                or "full-screen" in programmatic_text
                or "interactive" in programmatic_text
            )
        )
        self.expect(
            programmatic["ok"] and guidance,
            "Programmatic help must retain default help and explain piping, JSON, sections, and avoiding interactive output.",
            path,
            programmatic,
        )

        outline = self.run_json(
            path,
            ["help", "outline", "--format", "json"],
            "outline",
        )
        if outline is not None and self.validate_outline(outline, path, False):
            headings = outline["headings"]
            self.outlines[path] = headings
            self.check_outline_filters(path)
            self.check_sections(path, headings, False)

        programmatic_outline = self.run_json(
            path,
            [
                "help",
                "outline",
                "--programmatic",
                "--format",
                "json",
            ],
            "programmatic outline",
        )
        if programmatic_outline is not None and self.validate_outline(
            programmatic_outline, path, True
        ):
            self.check_sections(path, programmatic_outline["headings"], True)

    def check_outline_filters(self, path: tuple[str, ...]) -> None:
        level = self.run_json(
            path,
            ["help", "outline", "--level", "2", "--format", "json"],
            "outline level filter",
        )
        if level is not None and self.validate_base_response(
            level, path, False, "outline level filter"
        ):
            headings = level.get("headings", [])
            self.expect(
                isinstance(headings, list)
                and all(
                    isinstance(heading, dict) and heading.get("level") == 2
                    for heading in headings
                ),
                "--level 2 must return only level-two headings.",
                path,
                {"response": level},
            )
        maximum = self.run_json(
            path,
            ["help", "outline", "--max-level", "2", "--format", "json"],
            "outline maximum-level filter",
        )
        if maximum is not None and self.validate_base_response(
            maximum, path, False, "outline maximum-level filter"
        ):
            headings = maximum.get("headings", [])
            self.expect(
                isinstance(headings, list)
                and all(
                    isinstance(heading, dict)
                    and isinstance(heading.get("level"), int)
                    and 1 <= heading["level"] <= 2
                    for heading in headings
                ),
                "--max-level 2 must return only headings through level two.",
                path,
                {"response": maximum},
            )

    def validate_outline(
        self,
        response: dict[str, Any],
        path: tuple[str, ...],
        programmatic: bool,
    ) -> bool:
        if not self.validate_base_response(response, path, programmatic, "outline"):
            return False
        headings = response.get("headings")
        self.expectations += 1
        if not isinstance(headings, list) or not headings:
            self.fail(
                "A help outline must contain at least one heading.",
                path,
                {"response": response},
            )
            return False
        valid = True
        for heading in headings:
            heading_valid = (
                isinstance(heading, dict)
                and isinstance(heading.get("level"), int)
                and 1 <= heading["level"] <= 6
                and isinstance(heading.get("title"), str)
                and bool(heading["title"])
                and isinstance(heading.get("section"), str)
                and bool(heading["section"])
            )
            if not heading_valid:
                valid = False
                self.fail(
                    "An outline heading has an invalid level, title, or section.",
                    path,
                    {"heading": heading},
                )
        return valid

    def check_sections(
        self,
        path: tuple[str, ...],
        headings: list[dict[str, Any]],
        programmatic: bool,
    ) -> None:
        for section in dict.fromkeys(heading["section"] for heading in headings):
            programmatic_argument = ["--programmatic"] if programmatic else []
            response = self.run_json(
                path,
                [
                    "help",
                    "section",
                    section,
                    *programmatic_argument,
                    "--format",
                    "json",
                ],
                f"section {section}",
            )
            if response is None or not self.validate_base_response(
                response, path, programmatic, f"section {section}"
            ):
                continue
            sections = response.get("sections")
            expected_count = sum(
                heading["section"] == section for heading in headings
            )
            self.expect(
                isinstance(sections, list)
                and len(sections) == expected_count
                and all(
                    isinstance(value, dict)
                    and value.get("section") == section
                    and isinstance(value.get("content"), str)
                    for value in sections
                ),
                "Section retrieval must return every advertised match with content.",
                path,
                {
                    "requested_section": section,
                    "outline": headings,
                    "response": response,
                },
            )

            recursive = self.run_json(
                path,
                [
                    "help",
                    "section",
                    section,
                    "--recursive",
                    *programmatic_argument,
                    "--format",
                    "json",
                ],
                f"recursive section {section}",
            )
            if recursive is None or not self.validate_base_response(
                recursive, path, programmatic, f"recursive section {section}"
            ):
                continue
            expected = []
            for index, heading in enumerate(headings):
                if heading["section"] != section:
                    continue
                expected.append(heading)
                for descendant in headings[index + 1 :]:
                    if descendant["level"] <= heading["level"]:
                        break
                    expected.append(descendant)
            observed = recursive.get("sections")
            self.expect(
                isinstance(observed, list)
                and [
                    (value.get("level"), value.get("title"), value.get("section"))
                    for value in observed
                    if isinstance(value, dict)
                ]
                == [
                    (value["level"], value["title"], value["section"])
                    for value in expected
                ],
                "Recursive section retrieval must include each advertised match and its descendants in document order.",
                path,
                {
                    "requested_section": section,
                    "outline": headings,
                    "response": recursive,
                },
            )

        if not programmatic:
            missing = self.run(
                path,
                ["help", "section", "clilint-missing-section"],
                "unknown section",
            )
            self.expect(
                not missing["ok"] and missing["exit_status"] not in (None, 0),
                "Section retrieval must reject a section absent from the outline.",
                path,
                missing,
            )

    def validate_base_response(
        self,
        response: dict[str, Any],
        path: tuple[str, ...],
        programmatic: bool,
        operation: str,
    ) -> bool:
        valid = (
            response.get("format_version") == 1
            and response.get("command_path") == list(path)
            and response.get("programmatic") is programmatic
        )
        self.expect(
            valid,
            f"The {operation} response must identify its format, command path, and programmatic mode.",
            path,
            {"response": response},
        )
        return valid

    def run_json(
        self,
        path: tuple[str, ...],
        arguments: list[str],
        operation: str,
    ) -> dict[str, Any] | None:
        result = self.run(path, arguments, operation)
        if not result["ok"]:
            self.fail(
                f"The {operation} command must exit successfully.",
                path,
                result,
            )
            return None
        try:
            value = json.loads(result["stdout"])
        except json.JSONDecodeError as error:
            self.fail(
                f"The {operation} command must return valid JSON.",
                path,
                {**result, "parse_error": str(error)},
            )
            return None
        if not isinstance(value, dict):
            self.fail(
                f"The {operation} command must return one JSON object.",
                path,
                {**result, "decoded": value},
            )
            return None
        return value

    def run(
        self,
        path: tuple[str, ...],
        arguments: list[str],
        operation: str,
    ) -> dict[str, Any]:
        self.commands_run += 1
        if self.commands_run > TOTAL_COMMANDS_LIMIT:
            if not self.total_limit_reported:
                self.fail(
                    "The Checker exhausted the total help-command limit.",
                    path,
                    {"limit": TOTAL_COMMANDS_LIMIT},
                )
                self.total_limit_reported = True
            return {
                "ok": False,
                "operation": operation,
                "command_path": list(path),
                "error": "total help-command limit exceeded",
                "limit": TOTAL_COMMANDS_LIMIT,
            }
        command = [*self.target, *path, *arguments]
        with tempfile.TemporaryFile() as stdout_file, tempfile.TemporaryFile() as stderr_file:
            try:
                completed = subprocess.run(
                    command,
                    cwd=Path.cwd(),
                    stdin=subprocess.DEVNULL,
                    stdout=stdout_file,
                    stderr=stderr_file,
                    check=False,
                    timeout=COMMAND_TIMEOUT_SECONDS,
                )
                exit_status: int | None = completed.returncode
                timed_out = False
            except subprocess.TimeoutExpired:
                exit_status = None
                timed_out = True
            stdout_file.seek(0)
            stderr_file.seek(0)
            stdout_bytes = stdout_file.read(DOCUMENT_BYTES_LIMIT + 1)
            stderr_bytes = stderr_file.read(DOCUMENT_BYTES_LIMIT + 1)
        too_large = (
            len(stdout_bytes) > DOCUMENT_BYTES_LIMIT
            or len(stderr_bytes) > DOCUMENT_BYTES_LIMIT
        )
        stdout_bytes = stdout_bytes[:DOCUMENT_BYTES_LIMIT]
        stderr_bytes = stderr_bytes[:DOCUMENT_BYTES_LIMIT]
        result = {
            "ok": exit_status == 0 and not timed_out and not too_large,
            "operation": operation,
            "command_path": list(path),
            "command": command,
            "exit_status": exit_status,
            "timed_out": timed_out,
            "output_limit_exceeded": too_large,
            "stdout": stdout_bytes.decode("utf-8", errors="replace"),
            "stderr": stderr_bytes.decode("utf-8", errors="replace"),
        }
        if timed_out:
            self.fail(
                "A help command exceeded the per-command timeout.",
                path,
                {"operation": operation, "limit_seconds": COMMAND_TIMEOUT_SECONDS},
            )
        if too_large:
            self.fail(
                "A help command exceeded the captured-document size limit.",
                path,
                {"operation": operation, "limit_bytes": DOCUMENT_BYTES_LIMIT},
            )
        return result

    def expect(
        self,
        condition: bool,
        message: str,
        path: tuple[str, ...],
        evidence: Any,
    ) -> None:
        self.expectations += 1
        if not condition:
            self.fail(message, path, evidence)

    def fail(
        self,
        message: str,
        path: tuple[str, ...],
        evidence: Any,
    ) -> None:
        self.messages.append(
            {
                "level": "error",
                "message": message,
                "evidence": {
                    "command_path": list(path),
                    "detail": evidence,
                },
            }
        )


def error_response(request: dict[str, Any], message: str) -> dict[str, Any]:
    return {
        "format_version": FORMAT_VERSION,
        "request_id": request.get("request_id", "unknown"),
        "check": request.get("check", "unknown"),
        "method": METHOD,
        "outcome": "error",
        "error": {
            "message": message,
        },
    }


def main() -> int:
    try:
        request = json.load(sys.stdin)
    except (json.JSONDecodeError, OSError) as error:
        print(json.dumps(error_response({}, f"invalid Check Request: {error}")))
        return 0
    try:
        response = HelpChecker(request).check()
    except Exception as error:  # Keep Checker failures inside the protocol.
        print(json.dumps(error_response(request, f"Checker failed: {error}")))
        return 0
    print(json.dumps(response, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
