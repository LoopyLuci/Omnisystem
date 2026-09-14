"""Real behavior tests for omniharness.models.router.ModelRouter."""
from __future__ import annotations

import pytest

from omniharness.models.base import (
    ChatMessage,
    ChatRequest,
    ChatResponse,
    ModelAdapter,
    ModelInfo,
)
from omniharness.models.router import ModelRouter


class FakeAdapter(ModelAdapter):
    """A minimal ModelAdapter double so we can exercise routing logic without
    touching any real network/provider SDK."""

    def __init__(self, provider: str, models: list[str], fail: bool = False, healthy: bool = True):
        self.provider_name = provider
        self._models = models
        self.fail = fail
        self.healthy = healthy
        self.chat_calls: list[ChatRequest] = []

    async def chat(self, request: ChatRequest) -> ChatResponse:
        self.chat_calls.append(request)
        if self.fail:
            raise RuntimeError(f"{self.provider_name} is down")
        return ChatResponse(content=f"reply-from-{self.provider_name}", model_used=request.model_id)

    async def stream(self, request: ChatRequest):
        yield f"chunk-from-{self.provider_name}"

    async def health(self) -> bool:
        if not self.healthy:
            raise RuntimeError("unhealthy")
        return self.healthy

    def list_models(self) -> list[ModelInfo]:
        return [ModelInfo(id=m, provider=self.provider_name) for m in self._models]


def make_request(model_id: str) -> ChatRequest:
    return ChatRequest(model_id=model_id, messages=[ChatMessage(role="user", content="hi")])


def test_register_adds_to_registry_and_providers_list():
    router = ModelRouter()
    router.register("anthropic", FakeAdapter("anthropic", ["claude-x"]))
    assert router.providers == ["anthropic"]
    assert router.list_providers() == ["anthropic"]


def test_route_by_explicit_provider_prefix():
    router = ModelRouter()
    a = FakeAdapter("anthropic", ["claude-x"])
    router.register("anthropic", a)
    router.register("openai", FakeAdapter("openai", ["gpt-x"]))
    assert router.route("anthropic/claude-x") is a


def test_route_infers_provider_from_bare_model_name():
    router = ModelRouter()
    a = FakeAdapter("openai", ["gpt-4o"])
    router.register("openai", a)
    assert router.route("gpt-4o") is a


def test_route_falls_back_to_first_registered_when_unknown():
    router = ModelRouter()
    a = FakeAdapter("anthropic", ["claude-x"])
    router.register("anthropic", a)
    # Unknown model, no matching provider -> falls back to first registered.
    assert router.route("nonexistent-model") is a


def test_route_raises_when_nothing_registered():
    router = ModelRouter()
    with pytest.raises(RuntimeError, match="No model adapters registered"):
        router.route("anything")


@pytest.mark.asyncio
async def test_chat_uses_primary_adapter_on_success():
    router = ModelRouter()
    router.register("anthropic", FakeAdapter("anthropic", ["claude-x"]))
    resp = await router.chat(make_request("anthropic/claude-x"))
    assert resp.content == "reply-from-anthropic"


@pytest.mark.asyncio
async def test_chat_falls_back_to_other_provider_when_primary_fails():
    router = ModelRouter()
    failing = FakeAdapter("anthropic", ["claude-x"], fail=True)
    backup = FakeAdapter("openai", ["gpt-4o"])
    router.register("anthropic", failing)
    router.register("openai", backup)

    resp = await router.chat(make_request("anthropic/claude-x"))
    assert resp.content == "reply-from-openai"
    assert len(backup.chat_calls) == 1
    # The fallback request should have been rewritten to the backup provider's model.
    assert backup.chat_calls[0].model_id.startswith("openai/")


@pytest.mark.asyncio
async def test_chat_raises_when_all_adapters_fail():
    router = ModelRouter()
    router.register("anthropic", FakeAdapter("anthropic", ["claude-x"], fail=True))
    router.register("openai", FakeAdapter("openai", ["gpt-4o"], fail=True))
    with pytest.raises(RuntimeError, match="All adapters failed"):
        await router.chat(make_request("anthropic/claude-x"))


@pytest.mark.asyncio
async def test_health_returns_false_for_unregistered_provider():
    router = ModelRouter()
    assert await router.health("ghost") is False


@pytest.mark.asyncio
async def test_health_swallows_adapter_exceptions():
    router = ModelRouter()
    router.register("broken", FakeAdapter("broken", [], healthy=False))
    assert await router.health("broken") is False


@pytest.mark.asyncio
async def test_health_all_checks_every_provider_concurrently():
    router = ModelRouter()
    router.register("good", FakeAdapter("good", [], healthy=True))
    router.register("bad", FakeAdapter("bad", [], healthy=False))
    results = await router.health_all()
    assert results == {"good": True, "bad": False}


def test_list_all_models_aggregates_across_adapters():
    router = ModelRouter()
    router.register("anthropic", FakeAdapter("anthropic", ["claude-x", "claude-y"]))
    router.register("openai", FakeAdapter("openai", ["gpt-4o"]))
    ids = sorted(m.id for m in router.list_all_models())
    assert ids == ["claude-x", "claude-y", "gpt-4o"]


def test_register_from_env_registers_only_present_keys(monkeypatch):
    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)
    monkeypatch.delenv("OPENAI_API_KEY", raising=False)
    monkeypatch.setenv("OPENAI_API_KEY", "sk-test-key")
    router = ModelRouter()
    router.register_from_env()
    assert "openai" in router.providers
    assert "anthropic" not in router.providers


def test_autodiscover_local_throttled(monkeypatch):
    router = ModelRouter()
    probed = []
    monkeypatch.setattr(ModelRouter, "_probe", staticmethod(lambda url, timeout=0.6: probed.append(url) or False))
    router.autodiscover_local(force=True)
    first_count = len(probed)
    assert first_count > 0
    # Calling again immediately (without force) should be throttled: no new probes.
    router.autodiscover_local()
    assert len(probed) == first_count
