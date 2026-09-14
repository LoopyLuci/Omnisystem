"""Real behavior tests for omniharness.react.planner.HTNPlanner (HTN search + execution)."""
from __future__ import annotations

import pytest

from omniharness.react.planner import HTNPlanner, Task, Operator, Method


def test_builtin_operators_and_methods_registered():
    planner = HTNPlanner()
    assert {"ping", "read_file", "write_file", "llm_query"} <= set(planner._operators)
    assert "research" in planner._methods
    assert "code_change" in planner._methods


def test_plan_primitive_task_returns_itself():
    planner = HTNPlanner()
    plan = planner.plan(Task("ping", {"host": "example.com"}), {})
    assert plan == [Task("ping", {"host": "example.com"})]


def test_plan_compound_code_change_decomposes_to_read_then_write():
    planner = HTNPlanner()
    task = Task("code_change", {"path": "/tmp/x.py", "new_content": "print(1)"})
    plan = planner.plan(task, {})
    assert plan is not None
    assert [t.name for t in plan] == ["read_file", "write_file"]
    assert plan[0].params == {"path": "/tmp/x.py"}
    assert plan[1].params == {"path": "/tmp/x.py", "content": "print(1)"}


def test_plan_compound_method_condition_not_met_returns_none():
    planner = HTNPlanner()
    # code_change's Method.condition requires "path" in params
    plan = planner.plan(Task("code_change", {}), {})
    assert plan is None


def test_plan_unknown_compound_returns_none():
    planner = HTNPlanner()
    plan = planner.plan(Task("totally_unknown_task"), {})
    assert plan is None


def test_plan_research_decomposes_to_http_get_then_llm_query():
    planner = HTNPlanner()
    plan = planner.plan(Task("research", {"query": "rust vs go"}), {})
    # Regression test: "http_get" previously had neither an Operator nor a
    # Method registered, so any "research" task was silently unplannable
    # (plan() returned None). An "http_get" Operator is now registered.
    assert plan is not None
    assert [t.name for t in plan] == ["http_get", "llm_query"]
    assert "rust vs go" in plan[0].params["url"]
    assert "rust vs go" in plan[1].params["prompt"]


def test_operator_preconditions_block_planning():
    planner = HTNPlanner()
    planner.register_operator(Operator(
        name="locked",
        preconditions=lambda s: s.get("unlocked", False),
        effects=lambda s, p: {},
        execute_fn=lambda s, p: "done",
    ))
    assert planner.plan(Task("locked"), {}) is None
    assert planner.plan(Task("locked"), {"unlocked": True}) == [Task("locked")]


def test_method_tries_next_alternative_when_first_fails():
    planner = HTNPlanner()
    planner.register_operator(Operator(
        name="only_if_ready",
        preconditions=lambda s: s.get("ready", False),
        effects=lambda s, p: {},
        execute_fn=lambda s, p: "ran",
    ))
    planner.register_operator(Operator(
        name="always_ok",
        preconditions=lambda s: True,
        effects=lambda s, p: {},
        execute_fn=lambda s, p: "ran-fallback",
    ))
    planner.register_method(Method(
        compound_name="do_thing",
        condition=lambda s, p: True,
        decompose=lambda s, p: [Task("only_if_ready")],
    ))
    planner.register_method(Method(
        compound_name="do_thing",
        condition=lambda s, p: True,
        decompose=lambda s, p: [Task("always_ok")],
    ))
    # state has ready=False, so the first method's subtask fails preconditions;
    # the planner must backtrack and try the second registered method.
    plan = planner.plan(Task("do_thing"), {"ready": False})
    assert plan == [Task("always_ok")]


@pytest.mark.asyncio
async def test_execute_plan_ping_operator():
    planner = HTNPlanner()
    plan = [Task("ping", {"host": "myhost"})]
    result = await planner.execute_plan(plan, {})
    assert result["results"] == [{"task": "ping", "result": "PONG from myhost", "ok": True}]
    assert result["final_state"] == {}


@pytest.mark.asyncio
async def test_execute_plan_write_then_read_file(tmp_path):
    planner = HTNPlanner()
    target = tmp_path / "out.txt"
    plan = [
        Task("write_file", {"path": str(target), "content": "hi there"}),
        Task("read_file", {"path": str(target)}),
    ]
    result = await planner.execute_plan(plan, {})
    assert result["results"][0]["ok"] is True
    assert result["results"][0]["result"] == f"Wrote {target}"
    assert result["results"][1]["result"] == "hi there"
    assert result["final_state"]["last_write"] == str(target)
    assert result["final_state"]["last_file_content"] == "..."


@pytest.mark.asyncio
async def test_execute_plan_stops_on_first_error():
    planner = HTNPlanner()
    plan = [
        Task("read_file", {"path": "/nonexistent/does/not/exist.txt"}),
        Task("ping", {"host": "unreached"}),
    ]
    result = await planner.execute_plan(plan, {})
    assert len(result["results"]) == 1
    assert result["results"][0]["ok"] is False
    assert result["results"][0]["task"] == "read_file"
    assert "error" in result["results"][0]


@pytest.mark.asyncio
async def test_execute_plan_llm_query_uses_model_fn_from_state():
    planner = HTNPlanner()

    async def fake_model(prompt: str) -> str:
        return f"ANSWER: {prompt}"

    plan = [Task("llm_query", {"prompt": "what is 2+2"})]
    result = await planner.execute_plan(plan, {"model_fn": fake_model})
    # Regression test: llm_query used to call
    # asyncio.get_event_loop().run_until_complete(...) from inside
    # llm_execute, which raises "This event loop is already running"
    # whenever execute_plan (itself a coroutine) is the caller -- i.e.
    # always. llm_execute is now a proper async operator that gets awaited.
    assert result["results"][0]["ok"] is True
    assert result["results"][0]["result"] == "ANSWER: what is 2+2"


@pytest.mark.asyncio
async def test_execute_plan_llm_query_without_model_fn_uses_placeholder():
    planner = HTNPlanner()
    plan = [Task("llm_query", {"prompt": "hello"})]
    result = await planner.execute_plan(plan, {})
    assert result["results"][0]["result"] == "[LLM would respond to: hello]"
