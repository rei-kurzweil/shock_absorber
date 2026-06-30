# AI Harness Draft

Goal: build an AI harness that can make decisions about activity in the social graph.

Current root context:
- the backbone is the set of DIDs involved in the notification stream
- notifications are the primary event source
- attached context is loaded opportunistically from network sources and cached in memory

Current loaded graph context includes:
- notifications
- reply text / reply thread fragments
- bios
- pinned posts and their thread replies
- Clearsky moderation-list memberships

Near-term direction:
- treat each DID in the notification stream as a graph anchor
- attach cached facts and fetched expansions to those anchors
- expose that graph state to a decision layer

Open design questions:
- how to build and trim the context window
- what should stay in memory vs move to a database
- how to represent graph facts for prompting
- whether to use OpenAI-style local model APIs, OpenRouter, or both
- when decisions should be synchronous, deferred, or human-reviewed

Non-goal for now:
- locking into one model provider, one prompt format, or one persistence layer too early
