#!/usr/bin/env python3
"""Unit tests for the credential-free Codex search-contract guardrail."""

from __future__ import annotations

import copy
import importlib.util
import json
import sys
import unittest
from pathlib import Path

sys.dont_write_bytecode = True

SCRIPT = Path(__file__).resolve().parents[1] / "check_openai_codex_search_contract.py"
SNAPSHOT = Path(__file__).resolve().parents[2] / "contracts/openai_codex_search_contract.json"
SPEC = importlib.util.spec_from_file_location("check_openai_codex_search_contract", SCRIPT)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("could not load Codex search-contract checker")
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


def fixture_contract() -> dict[str, object]:
    return json.loads(SNAPSHOT.read_text(encoding="utf-8"))


class SearchContractComparisonTests(unittest.TestCase):
    def compare(self, current: dict[str, object]) -> tuple[list[str], list[str]]:
        return CHECKER.compare_source_contracts(fixture_contract(), current)

    def test_identical_contract_has_no_drift(self) -> None:
        errors, warnings = self.compare(fixture_contract())
        self.assertEqual(errors, [])
        self.assertEqual(warnings, [])

    def test_added_command_is_a_review_warning(self) -> None:
        current = copy.deepcopy(fixture_contract())
        current["commands"]["maps"] = {
            "wire_type": "array<MapsOperation>",
            "required": ["location"],
            "optional": ["radius"],
            "types": {"location": "string", "radius": "u64"},
        }

        errors, warnings = self.compare(current)

        self.assertEqual(errors, [])
        self.assertIn("command maps is additive and requires review", warnings)

    def test_added_optional_field_is_a_review_warning(self) -> None:
        current = copy.deepcopy(fixture_contract())
        current["commands"]["open"]["optional"].append("line_count")
        current["commands"]["open"]["types"]["line_count"] = "u64"

        errors, warnings = self.compare(current)

        self.assertEqual(errors, [])
        self.assertIn("command.open.line_count is a new optional field", warnings)

    def test_removed_field_is_breaking(self) -> None:
        current = copy.deepcopy(fixture_contract())
        current["commands"]["search_query"]["optional"].remove("domains")
        current["commands"]["search_query"]["types"].pop("domains")

        errors, _ = self.compare(current)

        self.assertIn("command.search_query.domains was removed or renamed", errors)

    def test_newly_required_field_is_breaking(self) -> None:
        current = copy.deepcopy(fixture_contract())
        current["commands"]["open"]["required"].append("render_mode")
        current["commands"]["open"]["types"]["render_mode"] = "string"

        errors, _ = self.compare(current)

        self.assertIn("command.open.render_mode is a newly required field", errors)

    def test_optional_field_becoming_required_is_breaking(self) -> None:
        current = copy.deepcopy(fixture_contract())
        current["settings"]["optional"].remove("filters")
        current["settings"]["required"].append("filters")

        errors, _ = self.compare(current)

        self.assertIn("settings.filters changed from optional to required", errors)

    def test_mode_wire_change_is_breaking(self) -> None:
        current = copy.deepcopy(fixture_contract())
        current["mode_projection"]["cached"] = "indexed"

        errors, _ = self.compare(current)

        self.assertIn("mode_projection changed", errors)


class SearchContractOfflineTests(unittest.TestCase):
    def test_committed_snapshot_and_adapter_pass_offline(self) -> None:
        document = CHECKER.validate_offline(SNAPSHOT, CHECKER.ROOT)
        self.assertEqual(document["source"]["commit"], CHECKER.PINNED_COMMIT)


if __name__ == "__main__":
    unittest.main()
