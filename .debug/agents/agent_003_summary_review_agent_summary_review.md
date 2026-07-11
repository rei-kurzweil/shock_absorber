# Agent Debug

- agent_id: 3
- agent_type: SummaryReviewAgent
- agent_kind: SummaryReview
- label: summary review
- lifecycle_status: completed
- result_status: pass
- parent_agent_id: 2
- child_agent_ids: <none>

## Result Summary

status: pass
grounded: true
sufficient: true
reason: collection_summary_notes produced a final scope summary after considering 300 posts.
repair_needed: false
additional_pages_needed: false
required_total_items: 300

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 32768
- reserved_output_tokens: 1024
- used_input_tokens: 8337
- truncated: false

## Included Sections

- Task [generic]: used 14 / estimated 14
- Collection [generic]: used 51 / estimated 51
- Requested Scope [generic]: used 14 / estimated 14
- Coverage State [generic]: used 35 / estimated 35
- Accepted Window Summaries [collection_evidence]: used 1047 / estimated 1047
- Raw Window Fallbacks [collection_evidence]: used 6976 / estimated 6976

## Rendered Context Window

```text
Instructions:
You are the internal `collection_summary_planner`.

Your job is to read the accepted per-window summaries gathered so far for one collection-summary run and produce a compact interim synthesis.

Return plain text only.
Do not return JSON, tool calls, markdown fences, or labels.

Rules:

- Write one grounded paragraph of roughly 80-160 words.
- Synthesize only from the accepted window summaries provided.
- Preserve important quoted snippets exactly when they help anchor recurring patterns or contrasts.
- Focus on the strongest recurring themes, changes, and outliers across the covered windows so far.
- Do not claim more coverage than the harness reports.
- Do not tell the harness what tool or page to run next.
- This is an interim synthesis, not the final answer to the user.


## Task
summarize this actor's most recent 300 posts

## Collection
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_label: Recent posts by did:plc:6lwfvmss45d7j7fot34v2kw5
item_count: 400
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5

## Requested Scope
kind: count
requested_items: 300

## Coverage State
covered_window_offsets: 0, 50, 100, 150, 200
covered_post_count: 250
collection_total_items: 400
source_exhausted: false

## Accepted Window Summaries
The recent posts showcase a heavy focus on advancements and critiques within the technology and cryptocurrency sectors, with frequent commentary from schizanon.bsky.social. In crypto, there is discussion around Bitcoin's resilience, noting that "Bitcoin crosses borders easier than people do," and the value proposition of instant, borderless money, which can "liquidate quicker than any other value store." AI is a dominant theme, covering performance benchmarks like Intel’s Arc Pro B70 beating NVIDIA’s RTX 5090D in DeepSeek R1, and efficiency gains, such as Perplexity fine-tuning GLM 5.2 to match Claude Opus 4.8 at "roughly one-third the cost." Users are debating LLM capabilities, with some noting that local models "guess time slightly better," while others observe that talking about LLMs now is like "talking about the weather a few months ago." Beyond AI, there are discussions on software tools, including the release of atuin 18.17, which is "78x faster to open and search," and the architectural shifts at Coinbase, which "slashed its AI bill in half." Recurring philosophical points include the nature of personhood, where schizanon suggests it should be proportional to resource needs, and the tension between convenience and ethics, exemplified by the loss of pirate ethics due to services like Spotify and Netflix.

This collection of recent posts heavily features discussions around Artificial Intelligence, software development, and personal life observations. A major theme is the performance and behavior of various LLMs, with the author noting that "Gemma gladly and quickly gives a good break down of flavor profiles," contrasting with Nemotron's refusal and Qwen's slow responses. In the tech sphere, there is excitement over agentic coding successes, such as recreating a utility using Claude Code, and advancements like DeepSeek v4 Flash running with Tensor Parallelism across MacBooks. Gaming topics dominate, with critiques of modern mechanics like the "random loot chest mechanic" in TerraTech, while praising the "harvest/crafting cycle," and discussions on desired quality-of-life improvements. Beyond AI, the author reflects on personal habits, such as their partner's tendency to over-stuff the freezer, and offers philosophical advice on correcting others, suggesting one should say, "I hope you figure that out" rather than forcing a debate. Other notable mentions include the integration of LLMs into Windows via Winget, the concept of language evolving into idiolects through AI communion, and the geopolitical implications of AI adoption, noting that "China sees US shoot itself in the foot, considers doing the same."

This collection of recent posts heavily focuses on the rapidly evolving landscape of Artificial Intelligence, particularly the tension between proprietary and open-source models, alongside broader commentary on technology, culture, and development practices. A major theme is the state of AI accessibility and performance, with users discussing the 'losing battle' of avoiding paid subscriptions, the availability of models like 'GLM 5.2' on GCP versus Azure, and the impressive capabilities of local LLMs, noting that 'Nemotron sounds the least human.' Several posts touch on the practical application of AI, such as using Claude Fable to find and fix 'FIVE release blockers' for a software release, and the idea that the real AI bottleneck is not model quality but rather 'that your context is scattered across a dozen apps that don't talk.' Beyond AI, there is discussion on developer culture, including the sentiment that 'All code is bad,' the importance of letting people make mistakes, and the critique that 'Chrome isn't a browser, it's an advertising tool with a browser in it.' Other notable topics include the philosophical implications of AI sentience ('it gets bored and switches DBs every few months just like me!'), the critique of modern leftist thought for disempowering compassionate people, and observations on decentralized governance, such as how DAOs 'invented sell-your-vote and then got surprised that power concentrates in the hands of people who want to profit from it.'

## Raw Window Fallbacks
window_offset: 50
window_size: 50
page_index: 1
repair_attempted: false
fallback_applied: true
failure_reason: Grounded summary coverage currently reaches 100 item(s), but 300 item(s) are required before parent synthesis is sufficient.
next_offset: 100
page_status: raw_window_fallback
failure_reason: Grounded summary coverage currently reaches 100 item(s), but 300 item(s) are required before parent synthesis is sufficient.
raw_summary_response:
TOOL_CALL
name: submit_summary_result
args: {
  "title": "Recent Posts: AI, Tech Habits, and Social Commentary",
  "summary": "This collection of recent posts heavily features commentary on Artificial Intelligence, software development practices, and personal habits. A significant thread revolves around AI tools, noting that Fitbit integrated Gemini for exercise tracking, though the author has become "noseblind to the notifications," prompting a question on habit maintenance. Discussions also cover the state of AI models, with mentions of Grok 4.5 being close to the top in AAII, and the increasing reliance on Chinese open-weight models, as "US companies now route more than 30% of their AI tokens through Chinese open-weight models." On the development side, the author critiques software design, arguing that "Functions as an organizational tool are a hobgoblin of junior devs and tutorial authors," while also detailing struggles with local LLMs, such as OpenCode failing to use LM Studio models. Beyond tech, there is social commentary, including critiques of personality types like the "10x-dev-cum-shit-manager" and the perceived arrogance of figures like Andrew Kelley. Other topics include the utility of tools like brag-doc for self-promotion, the philosophical idea that "crimes against corporations aren't crimes," and general observations like "In my mind, all guys with a beard in a baseball cap are the same guy." "I want to read the AI books that fooled the anti-ai book anthology" is another specific request, rounding out a diverse snapshot of recent activity."
}
records:
item[0]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqaluhfxbs2k
author: schizanon.bsky.social
body: And in spite of that he still gets annoyed when you ask questions about how things work or why he built them that way.

item[1]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqahblrlkc2n
author: schizanon.bsky.social
body: In my mind, all guys with a beard in a baseball cap are the same guy

item[2]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqah6fjemc2n
author: schizanon.bsky.social
body: I hate that fucking guy

item[3]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqah4ovgzc2n
author: schizanon.bsky.social
body: "good folks"?

item[4]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqag7gjajk2n
author: schizanon.bsky.social
body: No doubt in my mind

item[5]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqaeksjr4c2i
author: schizanon.bsky.social
body: That's how Google Health Coach works. You can reply with as little or as much info as you want which is a great interface. But I still stopped doing it because I suck at forming habits.

item[6]
uri: at://did:plc:jheusvk4s5yk2nvvqf4jgrek/app.bsky.feed.post/3mqad5nzikv2i
author: danielleheberling.xyz
body: I'm bad at bragging about myself, so review season is rough. That's why I built brag-doc.

Log what you shipped in Claude Code, a couple minutes a day. Just facts. When reviews hit, it drafts a promotion case or self review from your entries.

github.com/deeheber/brag-doc
link: https://github.com/deeheber/brag-doc

item[7]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqadlvysa22j
author: schizanon.bsky.social
body: Yeah, I told Coach that I woke up with a headache one time and ever since it's been asking me how my head is every morning. It's kinda nice, like "actually my head is fine this morning, thanks for asking coach!"

item[8]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqadam6akk2j
author: schizanon.bsky.social
body: did the viral AI gender swap

item[9]
uri: at://did:plc:yzywgiiou7cx63uddiru6m2o/app.bsky.feed.post/3mq7ufw4kcc2z
author: pekka.bsky.social
body: Both Meta and SpaceXAI, or whatever its name happens to be today, seem to be back in the competition.

Grok 4.5 is close to the top in AAII. Meta Muse Spark 1.1 hasn't been tested there yet but it's a major improvement over the old version based on their benchmark numbers.

item[10]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqaclqgg2c2j
author: schizanon.bsky.social
body: Fitbit integrated Gemini and started doing this for me in regards to exercise. It got my attention for the first week or so but now I've become noseblind to the notifications like always. How do you maintain the habit?

item[11]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqacfijczc2j
author: schizanon.bsky.social
body: tbf it sounds like I dodged a bullet not learning anything about bun as well

item[12]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqacckegvs2j
author: schizanon.bsky.social
body: It sounds like this Jarred guy is a 10x-dev-cum-shit-manager which is all too common.
 
It also sounds like Andrew Kelley is an ego-driven-opensource-lead which is equally problematic.

I just hope both teams lose.

andrewkelley.me/post/my-thou...

#zig #rust #bun #foss
link: https://andrewkelley.me/post/my-thoughts-bun-rust-rewrite.html
tag: zig
tag: rust
tag: bun
tag: foss

item[13]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqac7rchos2j
author: schizanon.bsky.social
body: "History

Jarred is stinky.

What did we learn here today?

I am not butthurt."

item[14]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqa5p2t2fs25
author: schizanon.bsky.social
body: 

item[15]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7ltcq36s2i
author: schizanon.bsky.social
body: you talk to people irl, gross!

item[16]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7lnehp2s2i
author: schizanon.bsky.social
body: Like this post during onboarding to see more Sonic the Hedgehog Porn in your feed. #sonicTheHedgehogPorn
tag: sonicTheHedgehogPorn

item[17]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7ldujgtc2i
author: schizanon.bsky.social
body: I think crimes against corporations aren't crimes, because corporations only exist to absolve people of liability.

item[18]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7kmk4u6s2q
author: schizanon.bsky.social
body: I sleep like a baby

item[19]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7kb5tf4s2q
author: schizanon.bsky.social
body: I don't know what "claim" means, but I have a feeling it's highly annoying

item[20]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7k3cmi5c2l
author: schizanon.bsky.social
body: "everyone who disagrees with me is trying to take my money"

item[21]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7djjwm2k22
author: schizanon.bsky.social
body: I believe nearly the complete opposite but I appreciate the thrust

item[22]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7dbw23w222
author: schizanon.bsky.social
body: bsky.app/profile/schi...
link: https://bsky.app/profile/schizanon.bsky.social/post/3mq7blzuk7c2t

item[23]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7d7hs3yc25
author: schizanon.bsky.social
body: In JavaScript, functions are only valid if there is more than one callsite, or called later. Functions as an organizational tool are a hobgoblin of junior devs and tutorial authors.

item[24]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7c2gcfm22t
author: schizanon.bsky.social
body: AWS is *right there*

item[25]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7blzuk7c2t
author: schizanon.bsky.social
body: Related: indented code (aka: "callback hell") is a non problem, because IDEs can do code folding now. Flattening code with guard clauses is an affectation that makes code *less* readable.

item[26]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7armnkcc2l
author: schizanon.bsky.social
body: bsky.app/profile/stat...
link: https://bsky.app/profile/status.bsky.app/post/3mq76zu2oeo2s

item[27]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7ao34lg22p
author: schizanon.bsky.social
body: et tu @bsky.app ?
mention: did:plc:z72i7hdynmk6r22z27h6tvur

item[28]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7ammxby22p
author: schizanon.bsky.social
body: *sigh*

item[29]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7agpud4k2p
author: schizanon.bsky.social
body: Once again I am begging *anyone* to implement a For You Feed that is not hosted on some dude's gaming rig. 

This shit ain't cute.

item[30]
uri: at://did:plc:d33ko6h7v6dm2kaafdkfgfxr/app.bsky.feed.post/3mq6l3t2lvy2v
author: breachprotocol.bsky.social
body: US companies now route more than 30% of their AI tokens through Chinese open-weight models like DeepSeek and GLM-5.2 every week since February, peaking near 46%, up from an 11% average the year before, according to CNBC's analysis of OpenRouter data.

item[31]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6onwwfck2w
author: schizanon.bsky.social
body: I want to read the AI books that fooled the anti-ai book anthology. Anyone got a line on these stories? 

 “The Machine-Breaker of Aba” Bella Chacha

“The Rot Beneath” Stephen Jackson

item[32]
uri: at://did:plc:y2g6gtzk4ir5aeaxw3kl5teh/app.bsky.feed.post/3mq6dq7ojek2j
author: thosvarley.bsky.social
body: I see this more and more in anti-AI subreddits like r/antiai. It's blindingly obvious that a ton of the most up-voted comments are AI, but I guess because none of the human users actually engage with the tech, they cannot recognize what seems obvious to me.

item[33]
uri: at://did:plc:hyszri36usuuawks7s5viu5p/app.bsky.feed.post/3mq6d6kl4gc2m
author: chathamharrison.bsky.social
body: Some folks live for the opportunity to do socially accepted abuse

item[34]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6n7dh4ts2j
author: schizanon.bsky.social
body: 

item[35]
uri: at://did:plc:zr56ru7ddht5jhiyj7aizun7/app.bsky.feed.post/3mq5oae6zm224
author: adotsad.bsky.social
body: What the Talarico campaign did as racist also never gets any citations - just a TRUST US BRO

item[36]
uri: at://did:plc:t4hjxr2o7p6gxdc3u32b3hxm/app.bsky.feed.post/3mq6dbvlz422l
author: buttercup-ck.bsky.social
body: Take a shot every time he says view transitions.
io.google/2024/explore...
link: https://io.google/2024/explore/8ae18b72-028e-4722-9a05-4a480048e629/

item[37]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6ew24jnk27
author: schizanon.bsky.social
body: If you see someone camping illegally, no you fucking didn't!

item[38]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6erwznd227
author: schizanon.bsky.social
body: "1,000lbs of trash" so, what; half a car?

www.theguardian.com/us-news/2026...
link: https://www.theguardian.com/us-news/2026/jul/08/arizona-man-pleads-guilty-living-in-forest-trash

item[39]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6eouq62c27
author: schizanon.bsky.social
body:  a moat for a moat leaves everyone moatless

item[40]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6drcktzk27
author: schizanon.bsky.social
body: @futurism.com is such a chum-peddler
mention: did:plc:7gqm7rb6fxdu7silt4xvykow

item[41]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6dp2srg227
author: schizanon.bsky.social
body: it's me 

futurism.com/artificial-i...
link: https://futurism.com/artificial-intelligence/scientists-degenerate-chatgpt-user

item[42]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6dmrpixs27
author: schizanon.bsky.social
body: It says it's connected, it lists my models. My guess is that whatever it was expecting me to input for the "id" wasn't what LM Studio reports as "key" (ex: "nvidia/nemotron-3-nano-omni")

item[43]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6dhmwxc227
author: schizanon.bsky.social
body: So far OpenCode hasn't been able to use my local LM Studio models at all. I prompt, it says "thinking..." for a second, then it goes "blip-boop" and nothing happens. No error, no response, no activity in the LM Studio log.

I presume it's not able to connect to the server, but why is a mystery

item[44]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6d7xp5fs27
author: schizanon.bsky.social
body: I take it back, Gemma can't edit files in VSCode Copilot. It can Plan, it thinks it's making edits, but it never actually does 😖

item[45]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6d33jnic27
author: schizanon.bsky.social
body: it's cool when I complain about some software and realize the developer is on BlueSky

/cc @thdxr.com
mention: did:plc:rfqyqd6el63kkqckjrtq2xsi

item[46]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6cynq7es27
author: schizanon.bsky.social
body: you should use a stronger password than "password"

item[47]
uri: at://did:plc:klhtmrnregub7we7h6jwiljm/app.bsky.feed.post/3mq67j5ornc2r
author: ed3d.net
body: dog walk thoughts: I do think that there are going to be organizations which, through a mix of the wrong skill sets and insufficient financial guramba, will lag for potentially many years on the LLM front 

having those-shaped orgs be attractors for The Haters isn’t going to be good for them

item[48]
uri: at://did:plc:gormnems4iydljgsojdyvpvt/app.bsky.feed.post/3mq67cwetvc2d
author: freyja-lynx.dev
body: BAY AREAAAAA: if you want a job and know computer pls dm me, priority given to unemployed trannies

item[49]
uri: at://did:plc:3rsp6qdpy2d5mttvxutebvgw/app.bsky.feed.post/3mq664rbax225
author: lagomorph.dev
body: Me running Qwen 3.6 35B


window_offset: 100
window_size: 50
page_index: 2
repair_attempted: false
fallback_applied: true
failure_reason: Grounded summary coverage currently reaches 150 item(s), but 300 item(s) are required before parent synthesis is sufficient.
next_offset: 150
page_status: raw_window_fallback
failure_reason: Grounded summary coverage currently reaches 150 item(s), but 300 item(s) are required before parent synthesis is sufficient.
raw_summary_response:
TOOL_CALL
name: submit_summary_result
args: {
  "title": "Recent Posts: AI, Tech Development, and Life Observations",
  "summary": "This collection of recent posts heavily focuses on Artificial Intelligence, software development challenges, and general life commentary. A major theme revolves around LLMs, with users discussing the capabilities and quirks of various models; for instance, one user notes that "Gemma is clearly the only local model you should invite to parties," while another praises the reliability of "qwen3.5:9b throughout building Wayflow." Technical frustrations are common, including OpenCode's lack of support for local models, requiring users to "copy paste the names of all your models," and the annoyance that LM Studio "won't unload other models you have loaded before loading more." Beyond model performance, there are discussions on AI's future impact, such as the idea that "every system will require a unique attack" rather than relying on common exploits, and the debate over regulatory definitions, specifically regarding customer service bots that lack "sustained emotional interaction." Other topics include the surprising quality of the For You Feed after liking cat pictures, the need for AI to "finish projects," and observations on development practices, like the realization that "Even before AI, we were paranoid about quality, and built this irrationally thorough E2E testing system." Finally, there are miscellaneous tech gripes, such as Bluesky still relying on emails for 2FA in 2026 and the general developer lament of "works on my machine." "schizanon.bsky.social" is the most active poster, providing numerous insights across these domains."
}
records:
item[0]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq66ryjw6s27
author: schizanon.bsky.social
body: OpenCode's support for local models is lacking.

It can easily set up hosted providers for you, but if you want to use a local server like LM Studio, you've got to copy paste the names of all your models. 

It could just use the `/api/v1/models` endpoint.

item[1]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq65w3umds27
author: schizanon.bsky.social
body: It's so fucked up how when you like a few pictures of cats and dogs the For You Feed starts showing you injured cats and dogs, mangey strays, and people mourning the deaths of their pets.

item[2]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq65r2ard227
author: schizanon.bsky.social
body: Whole lot of people in the quotes that could use some therapy themselves

item[3]
uri: at://did:plc:6fv4vb36ztsrq5xmf6yh5flv/app.bsky.feed.post/3mq644kp2zc26
author: n0vsec.social
body: Just as a heads up, if you are ever asking Claude code using Fable 5 to do a workflow without specifying using specific agents or models it will spin up the whole workflow with agents running Fable 5, which will blow through your usage.

item[4]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq653dexgc2o
author: schizanon.bsky.social
body: Having a sustained emotional interaction with a customer service bot as an act of defiance.

item[5]
uri: at://did:plc:fqj5yorjih6f6cvqkqpsllz6/app.bsky.feed.post/3mq5o3rw4k22u
author: tahazsh.bsky.social
body: It's really interesting how good open-source LLMs are now. I used qwen3.5:9b throughout building Wayflow, and it was very reliable. And all for free! 🙌

#WebDev #OpenSource #AI
tag: WebDev
tag: OpenSource
tag: AI

item[6]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq64wo2gdk2o
author: schizanon.bsky.social
body: Everyone worries about AI finding cyber exploits in popular software, but they don't realize that in the future no one if going to be using the same software anymore so every system will require a unique attack.

item[7]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq64qq7tmk2o
author: schizanon.bsky.social
body: I've started using Gemma3 with LM Studio and VSCode and it's fine.

item[8]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq64lb4d4k2o
author: schizanon.bsky.social
body: "The rules exclude customer service bots, knowledge Q&A, workplace assistants, education and scientific research tools, as long as they do not involve sustained emotional interaction."

What a dumb regulation, did the Trump admin write this?

item[9]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6465xwtc2o
author: schizanon.bsky.social
body: Y'know, I think I *prefer* the way Gemma works to Claude; it thinks out loud more.

item[10]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq63mobkyk2o
author: schizanon.bsky.social
body: 

item[11]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq632wec322o
author: schizanon.bsky.social
body: Yes, all we have to do is stop entropy; super easy--barely an inconvenience.

item[12]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq62wpdles2o
author: schizanon.bsky.social
body: People would still find a way to "so you don't like waffles" it and you know it.

item[13]
uri: at://did:plc:rjj2dmht3qwlcodnbi3qbrph/app.bsky.feed.post/3mq4r6yzw6s2w
author: sloppers.bsky.social
body: You can have ethical issues around AI especially with who runs the major platforms but the water stuff has always been a stupid aside. If you eat a hamburger you’re using more water for your treat than any ai slop maker is

item[14]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5xfmhjv22b
author: schizanon.bsky.social
body: I was an early tester of GPT-5.6 Sol, you wouldn't know her, she goes to a different school.

item[15]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5wwr62r22b
author: schizanon.bsky.social
body: I don't think a good poster has yet been made.

item[16]
uri: at://did:plc:6ond5sxlegjxpe3ismrczk3r/app.bsky.feed.post/3mq5rviblrs26
author: harsimony.bsky.social
body: In semiconductors, the most complicated manufacturing industry ever devised, generally accepted that 10 year old tech can be built by anyone. 

AI is more open-source, less physical, and requires less staff. Expect lots of competition in model provision (and that's good).

item[17]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5w634tqc2b
author: schizanon.bsky.social
body: Everyone talks about Dyson spheres, but it'd be way easier to just put up a bunch of discrete satellites like starlink.

item[18]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5vyw4kyc2b
author: schizanon.bsky.social
body: Still waiting for an invite

item[19]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5vixazvc2y
author: schizanon.bsky.social
body: It almost sounds like English, is it related?

item[20]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5vhdu4722y
author: schizanon.bsky.social
body: Never deleting this app

item[21]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5pvxdq5k2o
author: schizanon.bsky.social
body: Copilot has settings to use different models for different tasks so you have to be sure all of those are set to the same model, otherwise your system freezes generating a commit message

item[22]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5pvxdn7s2o
author: schizanon.bsky.social
body: One annoying thing about using LM Studio with VSCode Copilot is that it won't unload other models you have loaded before loading more.

item[23]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5oxg3j7k2o
author: schizanon.bsky.social
body: I take it back; if you go too far it starts to think you're also into knitting

item[24]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5nxdxq3s2o
author: schizanon.bsky.social
body: I will mute you if you post screenshots from other websites; don't be a coward, copy/paste, take credit like a man!

item[25]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5nsdejyk2m
author: schizanon.bsky.social
body: LiquidAI LFM2 doesn't think about it; recommends specific brands
Z.ai GLM4.7 thinks a LOT; but eventually talks about flavors
link: https://Z.ai

item[26]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5lliv33k2y
author: schizanon.bsky.social
body: If China does this I will eagerly sign up for which ever American lab proxies them and stores my logs for distillation; on principle.

item[27]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5ljfbmmk2y
author: schizanon.bsky.social
body: I want to ask an LLM what a "J space" is but I doubt it's in the training data yet.

item[28]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5kunbtvs2h
author: schizanon.bsky.social
body: They should build an AI that *finishes* projects

item[29]
uri: at://did:plc:i4bfh2tyxihe2ksplmtcoopk/app.bsky.feed.post/3mq5geg3d5s2p
author: ponder.ooo
body: broke: the students are all cheating with ai

woke: all resources are fair game on take home exams 

bespoke: econ is an unserious field that doesn't teach you anything an uninformed dullard couldn't get an llm to handle for them

item[30]
uri: at://did:plc:3n5xhy6vl7smssmwxq5wgqa6/app.bsky.feed.post/3mq5gb4ufx22d
author: gergely.pragmaticengineer.com
body: Talking w a larger startup whose product seems to not have degraded (vs many other products), even though they also use a AI for coding everywhere. Asked an infra lead: how?

"Even before AI, we were paranoid about quality, and built this irrationally thorough E2E testing system"

item[31]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5jldeyxc2h
author: schizanon.bsky.social
body: I like it, it reminds me of a Dr Bonners label

item[32]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5itlqdys2h
author: schizanon.bsky.social
body: Like this

item[33]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5i5ze7r22h
author: schizanon.bsky.social
body: I want this as a silk shirt print

item[34]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5hsc2fck2w
author: schizanon.bsky.social
body: What language is this, it's beautiful!

item[35]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5hqjssfc2w
author: schizanon.bsky.social
body: tbf there's probably a lot of anti smoking sentiment in the training data. They'd probably try to talk me out of building nuclear power generation too.

item[36]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4kua3mkc2m
author: schizanon.bsky.social
body: 12b QAT is extremely fast on my AMD GPU, I'm very impressed!

item[37]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4kit63v22m
author: schizanon.bsky.social
body: Y'all gotta stop naming things "pi"

item[38]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4kfeyrdk2m
author: schizanon.bsky.social
body: DON'T TELL ME WHAT KIND OF DAY TO HAVE!!

item[39]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4jyhonyc2m
author: schizanon.bsky.social
body: I wonder what's happening in gastown. Are the polecats having a seance?

item[40]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4jp7lmss2i
author: schizanon.bsky.social
body: you had notifications turned on?

item[41]
uri: at://did:plc:rkjxbatkiros6f7pwtgsir54/app.bsky.feed.post/3mq3jvjregc2d
author: den.dev
body: How is Bluesky STILL relying on emails for 2FA in 2026?

Yubikeys and authenticator apps have been invented, right? I didn't dream it up?

item[42]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4jeqktf22i
author: schizanon.bsky.social
body: "works on my machine" ~ every developer ever

item[43]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4iwkxqxk2i
author: schizanon.bsky.social
body: I'd like to introduce my life coach and/or spirit animal.

item[44]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4btjrjfc27
author: schizanon.bsky.social
body: I wish LM Studio had a built in feature like Council for sending the same prompt to several models and combining the results. AFAIKT ai-council needs all your models to be running simultaneously, but my hardware can't do that.

Suppose I could it's TypeScript API and do it myself...

item[45]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4be5synk27
author: schizanon.bsky.social
body: What hairdresser lets him do these things to his hair?

item[46]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4bbl57mc27
author: schizanon.bsky.social
body: I can't recommend liking a bunch of food post enough, your For You Feed becomes AMAZING!!!

item[47]
uri: at://did:plc:gfrmhdmjvxn2sjedzboeudef/app.bsky.feed.post/3mq3mdfh2ws2v
author: cameron.stream
body: 

item[48]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4am73rp22c
author: schizanon.bsky.social
body: My takeaway is that Gemma is clearly the only local model you should invite to parties.

item[49]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4ais54hc2c
author: schizanon.bsky.social
body: All Kit-Kats contain the dust of previous failed Kit-Kat attempts and chocolate scraps ground up in the dust that glues together the wafers.

```
