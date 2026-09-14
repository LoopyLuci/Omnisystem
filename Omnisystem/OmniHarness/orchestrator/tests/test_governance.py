"""Real behavior tests for omniharness.substrate.governance (budgets, policy, audit chain, kill switch)."""
from __future__ import annotations

import pytest

from omniharness.substrate.governance import (
    AuditLog,
    Aborted,
    Budget,
    BudgetExceeded,
    CapabilityPolicy,
    Governor,
    KillSwitch,
    PolicyViolation,
    Usage,
)


# ---------------------------------------------------------------------------
# CapabilityPolicy
# ---------------------------------------------------------------------------


def test_policy_empty_allowlist_permits_everything():
    policy = CapabilityPolicy()
    assert policy.model_allowed("any-model") is True
    assert policy.tool_allowed("any-tool") is True


def test_policy_allowlist_restricts():
    policy = CapabilityPolicy(allowed_models=["m1"], allowed_tools=["t1"])
    assert policy.model_allowed("m1") is True
    assert policy.model_allowed("m2") is False
    assert policy.tool_allowed("t1") is True
    assert policy.tool_allowed("t2") is False


def test_policy_denylist_wins_even_if_allowed():
    policy = CapabilityPolicy(allowed_tools=["t1"], denied_tools=["t1"])
    assert policy.tool_allowed("t1") is False


# ---------------------------------------------------------------------------
# AuditLog — tamper-evident hash chain
# ---------------------------------------------------------------------------


def test_audit_log_chains_hashes_and_verifies():
    log = AuditLog()
    log.append("event_a", {"x": 1})
    log.append("event_b", {"y": 2})
    events = log.events()
    assert len(events) == 2
    assert events[1]["prev"] == events[0]["hash"]
    assert log.verify() is True


def test_audit_log_detects_tampering():
    log = AuditLog()
    log.append("event_a", {"x": 1})
    log.append("event_b", {"y": 2})
    # Mutate a past event's payload in place -> hash chain must break.
    log._events[0]["payload"]["x"] = 999
    assert log.verify() is False


def test_audit_log_kernel_mirror_is_called_and_failures_are_swallowed():
    log = AuditLog()
    seen = []
    log.attach_kernel_mirror(lambda kind, payload: seen.append((kind, payload)))
    log.append("model_call", {"model": "m"})
    assert seen == [("model_call", {"model": "m"})]

    # A mirror that raises must not break append() or the chain.
    def boom(kind, payload):
        raise RuntimeError("kernel unreachable")

    log.attach_kernel_mirror(boom)
    log.append("another", {})
    assert log.verify() is True


# ---------------------------------------------------------------------------
# KillSwitch
# ---------------------------------------------------------------------------


def test_kill_switch_trips_with_reason():
    kill = KillSwitch()
    assert kill.tripped is False
    kill.trip("user aborted")
    assert kill.tripped is True
    assert kill.reason == "user aborted"


# ---------------------------------------------------------------------------
# Governor — budgets, policy enforcement, kill switch, reporting
# ---------------------------------------------------------------------------


def test_checkpoint_raises_aborted_when_killed():
    gov = Governor()
    gov.kill.trip("stop now")
    with pytest.raises(Aborted, match="stop now"):
        gov.checkpoint()
    # The abort must itself be recorded in the audit log.
    assert gov.audit.events()[-1]["kind"] == "aborted"


def test_checkpoint_enforces_max_steps():
    gov = Governor(budget=Budget(max_steps=2))
    gov.checkpoint()
    gov.checkpoint()
    with pytest.raises(BudgetExceeded, match="max_steps"):
        gov.checkpoint()


def test_check_model_enforces_policy():
    gov = Governor(policy=CapabilityPolicy(allowed_models=["good-model"]))
    gov.check_model("good-model")  # should not raise
    with pytest.raises(PolicyViolation, match="not permitted"):
        gov.check_model("bad-model")


def test_check_model_enforces_max_calls():
    gov = Governor(budget=Budget(max_model_calls=1))
    gov.check_model("m")
    gov.record_call("m", 100)
    with pytest.raises(BudgetExceeded, match="max_model_calls"):
        gov.check_model("m")


def test_check_tool_enforces_policy():
    gov = Governor(policy=CapabilityPolicy(denied_tools=["rm_rf"]))
    gov.check_tool("read_file")  # ok
    with pytest.raises(PolicyViolation, match="not permitted"):
        gov.check_tool("rm_rf")


def test_record_call_tracks_usage_and_enforces_token_budget():
    gov = Governor(budget=Budget(max_tokens=1000))
    gov.record_call("m", 400)
    assert gov.usage.model_calls == 1
    assert gov.usage.tokens == 400
    with pytest.raises(BudgetExceeded, match="max_tokens"):
        gov.record_call("m", 700)  # 400 + 700 = 1100 > 1000


def test_record_call_enforces_cost_budget():
    gov = Governor(budget=Budget(max_cost_usd=0.001, max_tokens=10**9))
    # cost = tokens/1000 * 0.005; need > 0.001 => tokens > 200
    with pytest.raises(BudgetExceeded, match="max_cost_usd"):
        gov.record_call("m", 1000)


def test_parallelism_clamps_to_budget_and_policy():
    gov = Governor(budget=Budget(max_parallel=4), policy=CapabilityPolicy(max_agents=2))
    assert gov.parallelism(10) == 2  # policy is the tighter bound
    assert gov.parallelism(1) == 1
    assert gov.parallelism(0) == 1  # clamped to at least 1


def test_report_reflects_state():
    gov = Governor()
    gov.checkpoint()
    gov.record_call("m", 100)
    report = gov.report()
    assert report["usage"]["model_calls"] == 1
    assert report["usage"]["tokens"] == 100
    assert report["audit_valid"] is True
    assert report["audit_events"] >= 1
    assert report["killed"] is False


def test_usage_snapshot_rounds_cost_and_elapsed():
    usage = Usage()
    usage.cost_usd = 1.23456789
    snap = usage.snapshot()
    assert snap["cost_usd"] == round(1.23456789, 4)
    assert isinstance(snap["elapsed_s"], float)
