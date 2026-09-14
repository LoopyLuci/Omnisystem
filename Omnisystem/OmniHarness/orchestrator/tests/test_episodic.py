"""Real behavior tests for omniharness.memory.episodic.EpisodicMemory (real aiosqlite I/O)."""
from __future__ import annotations

import pytest

from omniharness.memory.episodic import EpisodicMemory
from omniharness.models.base import ChatMessage, ChatRequest, ChatResponse


@pytest.fixture
async def memory(tmp_path):
    mem = EpisodicMemory(db_path=str(tmp_path / "episodes.db"))
    await mem.init()
    yield mem
    await mem.close()


@pytest.mark.asyncio
async def test_add_turn_and_get_history_preserves_order(memory):
    await memory.add_turn("s1", "user", "hello")
    await memory.add_turn("s1", "assistant", "hi there")
    await memory.add_turn("s1", "user", "how are you")

    history = await memory.get_history("s1")
    assert [m.content for m in history] == ["hello", "hi there", "how are you"]
    assert [m.role for m in history] == ["user", "assistant", "user"]


@pytest.mark.asyncio
async def test_get_history_respects_max_turns_and_keeps_most_recent(memory):
    for i in range(5):
        await memory.add_turn("s1", "user", f"msg{i}")

    history = await memory.get_history("s1", max_turns=2)
    # Most recent 2 turns, still in chronological order.
    assert [m.content for m in history] == ["msg3", "msg4"]


@pytest.mark.asyncio
async def test_sessions_are_isolated(memory):
    await memory.add_turn("s1", "user", "in session 1")
    await memory.add_turn("s2", "user", "in session 2")

    h1 = await memory.get_history("s1")
    h2 = await memory.get_history("s2")
    assert [m.content for m in h1] == ["in session 1"]
    assert [m.content for m in h2] == ["in session 2"]


@pytest.mark.asyncio
async def test_search_history_keyword_match(memory):
    await memory.add_turn("s1", "user", "tell me about rust ownership")
    await memory.add_turn("s1", "assistant", "ownership tracks memory")
    await memory.add_turn("s1", "user", "what about python?")

    results = await memory.search_history("s1", "ownership")
    assert len(results) == 2
    assert all("ownership" in m.content for m in results)


@pytest.mark.asyncio
async def test_get_all_sessions_lists_distinct_sessions(memory):
    await memory.add_turn("alpha", "user", "a")
    await memory.add_turn("beta", "user", "b")
    await memory.add_turn("alpha", "user", "a2")

    sessions = await memory.get_all_sessions()
    assert set(sessions) == {"alpha", "beta"}


@pytest.mark.asyncio
async def test_delete_session_removes_only_its_turns(memory):
    await memory.add_turn("keep", "user", "stays")
    await memory.add_turn("drop", "user", "goes")
    await memory.add_turn("drop", "user", "goes too")

    deleted = await memory.delete_session("drop")
    assert deleted == 2
    assert await memory.get_history("drop") == []
    assert [m.content for m in await memory.get_history("keep")] == ["stays"]


@pytest.mark.asyncio
async def test_summarize_if_long_no_op_below_threshold(memory):
    for i in range(5):
        await memory.add_turn("s1", "user", f"msg{i}")

    class UnusedRouter:
        async def chat(self, req):
            raise AssertionError("router.chat should not be called below threshold")

    summarized = await memory.summarize_if_long("s1", UnusedRouter(), threshold_turns=40)
    assert summarized is False
    assert len(await memory.get_history("s1", max_turns=1000)) == 5


@pytest.mark.asyncio
async def test_summarize_if_long_collapses_old_turns_and_keeps_recent(memory):
    for i in range(15):
        await memory.add_turn("s1", "user", f"msg{i}")

    class FakeRouter:
        def __init__(self):
            self.received_request = None

        async def chat(self, req: ChatRequest) -> ChatResponse:
            self.received_request = req
            return ChatResponse(content="a concise summary", model_used=req.model_id)

    router = FakeRouter()
    summarized = await memory.summarize_if_long(
        "s1", router, threshold_turns=10, keep_recent=3,
    )
    assert summarized is True
    assert router.received_request is not None
    assert "msg0" in router.received_request.messages[0].content

    history = await memory.get_history("s1", max_turns=1000)
    # 1 summary message + 3 kept recent turns.
    assert len(history) == 4
    assert history[0].role == "system"
    assert "a concise summary" in history[0].content
    assert [m.content for m in history[1:]] == ["msg12", "msg13", "msg14"]
