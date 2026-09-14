"""Real behavior tests for omniharness.react.tools.ToolRegistry."""
from __future__ import annotations

import json
import math
import os

import pytest

from omniharness.react.tools import ToolRegistry, Tool, get_registry


def test_builtins_registered():
    reg = ToolRegistry()
    names = {t.name for t in reg.list_all()}
    assert {
        "read_file", "write_file", "append_file", "list_dir",
        "http_get", "http_post", "calculator", "search_web",
        "get_time", "json_parse", "regex_search",
    } <= names


def test_as_openai_functions_shape():
    reg = ToolRegistry()
    fns = reg.as_openai_functions()
    calc = next(f for f in fns if f["function"]["name"] == "calculator")
    assert calc["type"] == "function"
    assert calc["function"]["parameters"]["required"] == ["expression"]


def test_as_anthropic_tools_shape():
    reg = ToolRegistry()
    tools = reg.as_anthropic_tools()
    calc = next(t for t in tools if t["name"] == "calculator")
    assert "input_schema" in calc
    assert calc["input_schema"]["required"] == ["expression"]


def test_register_and_unregister():
    reg = ToolRegistry()

    async def noop(x: int) -> str:
        return str(x)

    reg.register(Tool(name="custom", description="d", schema={}, fn=noop))
    assert reg.get("custom") is not None
    assert reg.unregister("custom") is True
    assert reg.get("custom") is None
    assert reg.unregister("custom") is False


@pytest.mark.asyncio
async def test_execute_unknown_tool():
    reg = ToolRegistry()
    result = await reg.execute("does_not_exist", {})
    assert "not found" in result
    assert "calculator" in result  # lists available tools


@pytest.mark.asyncio
async def test_execute_invalid_json_arguments():
    reg = ToolRegistry()
    result = await reg.execute("calculator", "{not valid json")
    assert result.startswith("Error: invalid JSON arguments")


@pytest.mark.asyncio
async def test_execute_wrong_arguments():
    reg = ToolRegistry()
    result = await reg.execute("calculator", {"wrong_kw": "1+1"})
    assert result.startswith("Error: wrong arguments for 'calculator'")


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "expr,expected",
    [
        ("2 + 2", 4.0),
        ("3 * (4 - 1)", 9.0),
        ("2 ** 10", 1024.0),
        ("10 % 3", 1.0),
        ("-5 + 2", -3.0),
        ("sqrt(16)", 4.0),
        ("pi > 3", None),  # handled separately below (comparisons unsupported)
    ],
)
async def test_calculator_arithmetic(expr, expected):
    reg = ToolRegistry()
    result = await reg.execute("calculator", {"expression": expr})
    if expected is None:
        # Comparisons are not part of the supported AST node set.
        assert result.startswith("Error evaluating")
    else:
        assert float(result) == pytest.approx(expected)


@pytest.mark.asyncio
async def test_calculator_rejects_unsafe_expression():
    reg = ToolRegistry()
    # __import__ is a Call to a Name not in the allowed math namespace.
    result = await reg.execute("calculator", {"expression": "__import__('os').system('echo pwned')"})
    assert result.startswith("Error evaluating")
    assert "Only simple calls" in result


@pytest.mark.asyncio
async def test_json_parse_valid_and_invalid():
    reg = ToolRegistry()
    ok = await reg.execute("json_parse", {"text": '{"a": 1, "b": [2, 3]}'})
    assert json.loads(ok) == {"a": 1, "b": [2, 3]}

    bad = await reg.execute("json_parse", {"text": "{not json"})
    assert bad.startswith("JSON parse error")


@pytest.mark.asyncio
async def test_regex_search_matches_and_limit():
    reg = ToolRegistry()
    result = await reg.execute("regex_search", {"pattern": r"\d+", "text": "a1 b22 c333"})
    assert json.loads(result) == ["1", "22", "333"]


@pytest.mark.asyncio
async def test_regex_search_invalid_pattern():
    reg = ToolRegistry()
    result = await reg.execute("regex_search", {"pattern": "(unclosed", "text": "abc"})
    assert result.startswith("Regex error")


@pytest.mark.asyncio
async def test_file_roundtrip(tmp_path):
    reg = ToolRegistry()
    target = tmp_path / "nested" / "file.txt"

    write_result = await reg.execute("write_file", {"path": str(target), "content": "hello"})
    assert "Written 5 chars" in write_result
    assert target.read_text() == "hello"

    append_result = await reg.execute("append_file", {"path": str(target), "content": " world"})
    assert "Appended 6 chars" in append_result
    assert target.read_text() == "hello world"

    read_result = await reg.execute("read_file", {"path": str(target)})
    assert read_result == "hello world"


@pytest.mark.asyncio
async def test_read_file_truncates_long_files(tmp_path):
    reg = ToolRegistry()
    target = tmp_path / "big.txt"
    target.write_text("\n".join(f"line{i}" for i in range(600)))

    result = await reg.execute("read_file", {"path": str(target)})
    lines = result.splitlines()
    assert lines[-1] == "... (100 more lines)"
    assert len(lines) == 501  # 500 content lines + the truncation marker


@pytest.mark.asyncio
async def test_list_dir(tmp_path):
    reg = ToolRegistry()
    (tmp_path / "a.txt").write_text("x")
    (tmp_path / "subdir").mkdir()

    result = await reg.execute("list_dir", {"path": str(tmp_path)})
    assert "a.txt" in result
    assert "subdir  [DIR]" in result


@pytest.mark.asyncio
async def test_get_time_is_iso_utc():
    reg = ToolRegistry()
    result = await reg.execute("get_time", {})
    assert result.endswith("+00:00")


def test_get_registry_is_singleton():
    r1 = get_registry()
    r2 = get_registry()
    assert r1 is r2
