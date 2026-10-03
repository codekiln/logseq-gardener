import importlib.util
from pathlib import Path
import subprocess
import unittest
from unittest import mock


CHECKER_PATH = Path(__file__).with_name("checker.py")
SPEC = importlib.util.spec_from_file_location("codekiln_help_checker", CHECKER_PATH)
checker = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(checker)


def request():
    return {
        "request_id": "request-1",
        "check": "codekiln-help/help/hierarchical",
        "target": ["fixture"],
    }


class DiscoveryChecker(checker.HelpChecker):
    def run_json(self, path, arguments, operation):
        children = ["child"] if not path else []
        return {
            "format_version": 1,
            "command_path": list(path),
            "programmatic": False,
            "child_commands": [{"name": child} for child in children],
        }


class LimitTests(unittest.TestCase):
    def test_command_count_limit_is_reported(self):
        instance = DiscoveryChecker(request())
        with mock.patch.object(checker, "COMMAND_COUNT_LIMIT", 1):
            instance.discover_paths()
        self.assertIn("command-count limit", instance.messages[0]["message"])

    def test_command_depth_limit_is_reported(self):
        instance = DiscoveryChecker(request())
        with mock.patch.object(checker, "COMMAND_DEPTH_LIMIT", 0):
            instance.discover_paths()
        self.assertIn("depth limit", instance.messages[0]["message"])

    def test_total_command_limit_is_reported_once(self):
        instance = checker.HelpChecker(request())
        instance.commands_run = checker.TOTAL_COMMANDS_LIMIT
        instance.run((), ["help"], "help")
        instance.run((), ["help"], "help")
        direct = [
            message
            for message in instance.messages
            if "total help-command limit" in message["message"]
        ]
        self.assertEqual(len(direct), 1)

    def test_document_size_limit_is_reported(self):
        def oversized(*_args, **kwargs):
            kwargs["stdout"].write(b"12345")
            return subprocess.CompletedProcess([], 0)

        instance = checker.HelpChecker(request())
        with (
            mock.patch.object(checker, "DOCUMENT_BYTES_LIMIT", 4),
            mock.patch.object(checker.subprocess, "run", side_effect=oversized),
        ):
            result = instance.run((), ["help"], "help")
        self.assertTrue(result["output_limit_exceeded"])
        self.assertIn("captured-document size limit", instance.messages[0]["message"])

    def test_per_command_timeout_is_reported(self):
        instance = checker.HelpChecker(request())
        timeout = subprocess.TimeoutExpired(["fixture"], 0.01)
        with mock.patch.object(checker.subprocess, "run", side_effect=timeout):
            result = instance.run((), ["help"], "help")
        self.assertTrue(result["timed_out"])
        self.assertIn("per-command timeout", instance.messages[0]["message"])


if __name__ == "__main__":
    unittest.main()
