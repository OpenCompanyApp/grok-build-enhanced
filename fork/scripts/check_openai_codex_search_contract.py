#!/usr/bin/env python3
"""Validate the normalized OpenAI Codex standalone-search contract.

The default mode is completely offline: it checks the committed snapshot and
local adapter sentinels.  ``--source`` additionally reads the requested commit
from an existing Codex Git checkout with ``git show``.  It never checks out a
revision, fetches from a remote, reads an auth file, or performs a network
request.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_SNAPSHOT = ROOT / "fork/contracts/openai_codex_search_contract.json"
PINNED_COMMIT = "315195492c80fdade38e917c18f9584efd599304"
SOURCE_FILES = (
    "codex-rs/codex-api/src/search.rs",
    "codex-rs/codex-api/src/endpoint/search.rs",
    "codex-rs/ext/web-search/src/extension.rs",
)
# This digest covers the parsed JSON document rather than whitespace. Updating
# the reviewed contract therefore requires an explicit checker change too.
EXPECTED_SNAPSHOT_SHA256 = "145289e9f73387bf0ce7e05881821d85e3a3dcbeee642ca41e0d4d97e1771aa2"


class ContractError(RuntimeError):
    """Raised when a breaking contract difference or invalid fixture is found."""


def canonical_digest(document: dict[str, Any]) -> str:
    payload = json.dumps(
        document, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def _find_braced_body(source: str, declaration: str) -> tuple[str, int]:
    match = re.search(declaration + r"\s*\{", source)
    if match is None:
        raise ContractError(f"pinned source is missing {declaration!r}")
    opening = source.find("{", match.start())
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index], match.start()
    raise ContractError(f"pinned source has an unterminated {declaration!r}")


def _strip_outer(generic: str, type_name: str) -> str | None:
    prefix = f"{type_name}<"
    if not generic.startswith(prefix) or not generic.endswith(">"):
        return None
    return generic[len(prefix) : -1].strip()


def _wire_type(rust_type: str) -> str:
    value = "".join(rust_type.split())
    optional = _strip_outer(value, "Option")
    if optional is not None:
        return _wire_type(optional)
    vector = _strip_outer(value, "Vec")
    if vector is not None:
        return f"array<{_wire_type(vector)}>"
    aliases = {"String": "string", "bool": "boolean"}
    return aliases.get(value, value.removeprefix("r#"))


def _field_contract(source: str, struct_name: str) -> dict[str, Any]:
    body, _ = _find_braced_body(source, rf"pub struct {re.escape(struct_name)}")
    required: list[str] = []
    optional: list[str] = []
    types: dict[str, str] = {}
    pattern = re.compile(r"^\s*pub\s+(r#)?([A-Za-z0-9_]+)\s*:\s*([^,\n]+),", re.MULTILINE)
    for match in pattern.finditer(body):
        name = match.group(2)
        rust_type = match.group(3).strip()
        (optional if rust_type.startswith("Option<") else required).append(name)
        types[name] = _wire_type(rust_type)
    if not types:
        raise ContractError(f"pinned source struct {struct_name} has no public fields")
    return {"required": required, "optional": optional, "types": types}


def _pascal_to_snake(value: str) -> str:
    first = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", value)
    return re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", first).lower()


def _enum_contract(source: str, enum_name: str) -> list[str]:
    body, start = _find_braced_body(source, rf"pub enum {re.escape(enum_name)}")
    prefix = source[max(0, start - 500) : start]
    rename_matches = re.findall(r'rename_all\s*=\s*"([a-z_]+)"', prefix)
    rename = rename_matches[-1] if rename_matches else None
    variants = re.findall(r"^\s*([A-Z][A-Za-z0-9_]*)\s*(?:\([^\n]*\))?\s*,", body, re.MULTILINE)
    if not variants:
        raise ContractError(f"pinned source enum {enum_name} has no variants")
    if rename == "lowercase":
        return [variant.lower() for variant in variants]
    if rename == "snake_case":
        return [_pascal_to_snake(variant) for variant in variants]
    return variants


def _tuple_enum_wire_types(source: str, enum_name: str) -> list[str]:
    body, _ = _find_braced_body(source, rf"pub enum {re.escape(enum_name)}")
    variants = re.findall(
        r"^\s*[A-Z][A-Za-z0-9_]*\s*\(([^\n]+)\)\s*,", body, re.MULTILINE
    )
    if not variants:
        raise ContractError(f"pinned source union {enum_name} has no tuple variants")
    return [_wire_type(variant.strip()) for variant in variants]


def _extract_endpoint(endpoint_source: str) -> dict[str, str]:
    path = re.search(r"fn path\(\).*?\{\s*\"([^\"]+)\"\s*\}", endpoint_source, re.DOTALL)
    method = re.search(r"\.execute\(Method::([A-Z]+),\s*Self::path\(\)", endpoint_source)
    if path is None or method is None:
        raise ContractError("pinned source search endpoint shape could not be extracted")
    return {"method": method.group(1), "path": path.group(1)}


def _extract_mode_projection(extension_source: str) -> dict[str, Any]:
    body, _ = _find_braced_body(
        extension_source, r"fn external_web_access_for_mode\([^)]*\)\s*->\s*ExternalWebAccess"
    )
    expected_fragments = {
        "disabled": "WebSearchMode::Disabled | WebSearchMode::Cached => ExternalWebAccess::Boolean(false)",
        "cached": "WebSearchMode::Disabled | WebSearchMode::Cached => ExternalWebAccess::Boolean(false)",
        "indexed": "WebSearchMode::Indexed => ExternalWebAccess::Mode(ExternalWebAccessMode::Indexed)",
        "live": "WebSearchMode::Live => ExternalWebAccess::Boolean(true)",
    }
    compact = " ".join(body.split())
    for mode, fragment in expected_fragments.items():
        if " ".join(fragment.split()) not in compact:
            raise ContractError(f"pinned source mode projection for {mode} changed")
    return {"disabled": False, "cached": False, "indexed": "indexed", "live": True}


def extract_source_contract(sources: dict[str, str], commit: str) -> dict[str, Any]:
    search = sources[SOURCE_FILES[0]]
    endpoint = sources[SOURCE_FILES[1]]
    extension = sources[SOURCE_FILES[2]]

    command_fields = _field_contract(search, "SearchCommands")
    operation_types = command_fields["types"]
    operation_structs = {
        "search_query": "SearchQuery",
        "image_query": "SearchQuery",
        "open": "OpenOperation",
        "click": "ClickOperation",
        "find": "FindOperation",
        "screenshot": "ScreenshotOperation",
        "finance": "FinanceOperation",
        "weather": "WeatherOperation",
        "sports": "SportsOperation",
        "time": "TimeOperation",
    }
    commands: dict[str, Any] = {}
    for name in command_fields["optional"]:
        wire_type = operation_types[name]
        if name == "response_length":
            fields = {"required": [], "optional": [], "types": {}}
        else:
            struct_name = operation_structs.get(name)
            if struct_name is None:
                # A newly added command is still extractable and is reported as
                # additive even if its operation DTO has not been reviewed yet.
                inner = wire_type.removeprefix("array<").removesuffix(">")
                struct_name = inner
            fields = _field_contract(search, struct_name)
        commands[name] = {"wire_type": wire_type, **fields}

    settings = _field_contract(search, "SearchSettings")
    settings["objects"] = {
        "user_location": _field_contract(search, "ApproximateLocation"),
        "filters": _field_contract(search, "SearchFilters"),
        "image_settings": _field_contract(search, "SearchImageSettings"),
    }

    enum_names = (
        "FinanceAssetType",
        "SportsToolName",
        "SportsFunction",
        "SportsLeague",
        "SearchResponseLength",
        "ExternalWebAccessMode",
        "LocationType",
        "SearchContextSize",
        "AllowedCaller",
    )
    if "allowed_callers: Some(vec![AllowedCaller::Direct])" not in extension:
        raise ContractError("pinned source no longer seals standalone search to direct callers")

    return {
        "schema_version": 1,
        "source": {
            "project": "OpenAI Codex",
            "repository": "https://github.com/openai/codex.git",
            "commit": commit,
            "files": list(SOURCE_FILES),
        },
        "endpoint": _extract_endpoint(endpoint),
        "request": {
            **_field_contract(search, "SearchRequest"),
            "input_wire_types": _tuple_enum_wire_types(search, "SearchInput"),
        },
        "response": _field_contract(search, "SearchResponse"),
        "commands": commands,
        "settings": settings,
        "enums": {name: _enum_contract(search, name) for name in enum_names},
        "wire_unions": {
            "ExternalWebAccess": _tuple_enum_wire_types(search, "ExternalWebAccess")
        },
        "mode_projection": _extract_mode_projection(extension),
        "security_defaults": {
            "allowed_callers": ["direct"],
            "location_type": "approximate",
        },
        "numeric_bounds": {
            "u64_min": 0,
            "u64_max": 18_446_744_073_709_551_615,
        },
    }


def _compare_fields(
    label: str,
    baseline: dict[str, Any],
    current: dict[str, Any],
    errors: list[str],
    warnings: list[str],
) -> None:
    baseline_required = set(baseline["required"])
    baseline_optional = set(baseline["optional"])
    current_required = set(current["required"])
    current_optional = set(current["optional"])

    for name in sorted((baseline_required | baseline_optional) - (current_required | current_optional)):
        errors.append(f"{label}.{name} was removed or renamed")
    for name in sorted(baseline_required & current_optional):
        errors.append(f"{label}.{name} changed from required to optional")
    for name in sorted(baseline_optional & current_required):
        errors.append(f"{label}.{name} changed from optional to required")
    for name in sorted(current_required - (baseline_required | baseline_optional)):
        errors.append(f"{label}.{name} is a newly required field")
    for name in sorted(current_optional - (baseline_required | baseline_optional)):
        warnings.append(f"{label}.{name} is a new optional field")
    for name in sorted((baseline_required | baseline_optional) & (current_required | current_optional)):
        if baseline["types"].get(name) != current["types"].get(name):
            errors.append(
                f"{label}.{name} wire type changed from "
                f"{baseline['types'].get(name)!r} to {current['types'].get(name)!r}"
            )


def compare_source_contracts(
    baseline: dict[str, Any], current: dict[str, Any]
) -> tuple[list[str], list[str]]:
    """Return breaking errors and additive review warnings."""

    errors: list[str] = []
    warnings: list[str] = []
    for scalar in ("endpoint", "wire_unions", "mode_projection", "security_defaults", "numeric_bounds"):
        if baseline[scalar] != current[scalar]:
            errors.append(f"{scalar} changed")

    _compare_fields("request", baseline["request"], current["request"], errors, warnings)
    if baseline["request"]["input_wire_types"] != current["request"]["input_wire_types"]:
        errors.append("request.input wire union changed")
    _compare_fields("response", baseline["response"], current["response"], errors, warnings)
    _compare_fields("settings", baseline["settings"], current["settings"], errors, warnings)

    baseline_commands = baseline["commands"]
    current_commands = current["commands"]
    for name in sorted(set(baseline_commands) - set(current_commands)):
        errors.append(f"command {name} was removed or renamed")
    for name in sorted(set(current_commands) - set(baseline_commands)):
        warnings.append(f"command {name} is additive and requires review")
    for name in sorted(set(baseline_commands) & set(current_commands)):
        if baseline_commands[name]["wire_type"] != current_commands[name]["wire_type"]:
            errors.append(f"command {name} wire type changed")
        _compare_fields(
            f"command.{name}",
            baseline_commands[name],
            current_commands[name],
            errors,
            warnings,
        )

    for name, fields in baseline["settings"]["objects"].items():
        if name not in current["settings"]["objects"]:
            errors.append(f"settings object {name} was removed or renamed")
            continue
        _compare_fields(
            f"settings.{name}",
            fields,
            current["settings"]["objects"][name],
            errors,
            warnings,
        )
    for name in sorted(set(current["settings"]["objects"]) - set(baseline["settings"]["objects"])):
        warnings.append(f"settings object {name} is additive and requires review")

    for name, values in baseline["enums"].items():
        if current["enums"].get(name) != values:
            errors.append(f"enum {name} values changed")
    for name in sorted(set(current["enums"]) - set(baseline["enums"])):
        warnings.append(f"enum {name} is additive and requires review")
    return errors, warnings


def _read_snapshot(path: Path) -> dict[str, Any]:
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ContractError(f"could not read contract snapshot {path}: {error}") from error
    if not isinstance(document, dict) or document.get("schema_version") != 1:
        raise ContractError("contract snapshot has an unsupported schema version")
    source = document.get("source")
    if not isinstance(source, dict) or source.get("commit") != PINNED_COMMIT:
        raise ContractError("contract snapshot is not pinned to the reviewed Codex commit")
    return document


def _require_fragments(path: Path, fragments: list[str]) -> None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as error:
        raise ContractError(f"could not read local adapter {path}: {error}") from error
    for fragment in fragments:
        if fragment not in text:
            raise ContractError(f"{path.relative_to(ROOT)} is missing adapter contract {fragment!r}")


def validate_local_adapter(repo_root: Path, snapshot: dict[str, Any]) -> None:
    limits = snapshot["adapter_limits"]
    backend = repo_root / "crates/codegen/xai-grok-tools/src/implementations/web_search/backends/openai_codex.rs"
    registry = repo_root / "crates/codegen/xai-grok-tools/src/implementations/web_search/backends/mod.rs"
    _require_fragments(
        backend,
        [
            'let url = format!("{}/alpha/search", self.base_url);',
            f"const MAX_CODEX_IMAGE_RESULTS: u64 = {limits['max_image_results']};",
            f"pub(crate) const MIN_CODEX_SEARCH_OUTPUT_TOKENS: u64 = {limits['min_useful_output_tokens']:_};",
            f"pub(crate) const MAX_CODEX_SEARCH_OUTPUT_TOKENS: u64 = {limits['max_output_tokens']:_};",
            'allowed_callers: ["direct"],',
            'r#type: "approximate",',
            "CodexWebSearchMode::Cached => ExternalWebAccessWire::Boolean(false)",
            "ExternalWebAccessModeWire::Indexed",
            "CodexWebSearchMode::Live => ExternalWebAccessWire::Boolean(true)",
        ],
    )
    _require_fragments(registry, [f'    "{name}",' for name in snapshot["commands"]])


def validate_offline(snapshot_path: Path, repo_root: Path) -> dict[str, Any]:
    snapshot = _read_snapshot(snapshot_path)
    digest = canonical_digest(snapshot)
    if digest != EXPECTED_SNAPSHOT_SHA256:
        raise ContractError(
            "normalized search contract changed without updating its reviewed checker digest "
            f"(expected {EXPECTED_SNAPSHOT_SHA256}, got {digest})"
        )
    validate_local_adapter(repo_root, snapshot)
    return snapshot


def _git_show(source: Path, commit: str, relative: str) -> str:
    environment = os.environ.copy()
    environment.update(
        {
            "GIT_NO_LAZY_FETCH": "1",
            "GIT_OPTIONAL_LOCKS": "0",
            "GIT_NO_REPLACE_OBJECTS": "1",
            "LC_ALL": "C",
        }
    )
    result = subprocess.run(
        ["git", "-C", str(source), "show", f"{commit}:{relative}"],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=environment,
    )
    if result.returncode != 0:
        reason = result.stderr.decode("utf-8", errors="replace").strip().splitlines()
        category = reason[-1] if reason else "Git object unavailable"
        raise ContractError(f"could not read pinned source object {relative}: {category}")
    return result.stdout.decode("utf-8")


def validate_source(snapshot: dict[str, Any], source: Path, commit: str) -> list[str]:
    sources = {relative: _git_show(source, commit, relative) for relative in SOURCE_FILES}
    extracted = extract_source_contract(sources, commit)
    errors, warnings = compare_source_contracts(snapshot, extracted)
    if errors:
        raise ContractError("breaking pinned-source drift: " + "; ".join(errors))
    return warnings


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=ROOT)
    parser.add_argument("--snapshot", type=Path, default=DEFAULT_SNAPSHOT)
    parser.add_argument(
        "--source",
        type=Path,
        help="existing OpenAI Codex Git checkout to inspect without checkout or fetch",
    )
    parser.add_argument("--commit", default=PINNED_COMMIT)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        snapshot = validate_offline(args.snapshot.resolve(), args.repo_root.resolve())
        warnings = (
            validate_source(snapshot, args.source.resolve(), args.commit)
            if args.source is not None
            else []
        )
    except (ContractError, OSError, subprocess.SubprocessError, UnicodeError) as error:
        print(f"OpenAI Codex search contract validation failed: {error}", file=sys.stderr)
        return 1
    for warning in warnings:
        print(f"OpenAI Codex search contract review warning: {warning}", file=sys.stderr)
    mode = "offline snapshot and pinned source" if args.source is not None else "offline snapshot"
    print(f"OpenAI Codex search contract OK: {mode}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
