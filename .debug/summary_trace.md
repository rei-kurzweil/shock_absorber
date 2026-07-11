[execute_public_summary]
status: start
query: summarize the most recent 300 posts by schizanon.bsky.social
actor_anchor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
actor_anchor_source: explicit_query_ref

[execute_public_summary]
status: actor_resolved
actor_handle: schizanon.bsky.social
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5

[execute_public_summary]
status: hydrate_start
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
hydrate_args: {
  "include_pinned_posts": true,
  "include_profile": true,
  "include_recent_posts": true,
  "recent_posts_feed_fetch_limit": 400,
  "recent_posts_min_top_level_posts": 300
}

[execute_public_summary]
status: hydrate_complete
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
collection_count: 7
collections:
actor_profile:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=actor_profile | posts=1
clearsky_lists:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=clearsky_lists | posts=100
pinned_posts:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=pinned_posts | posts=1
recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_posts | posts=400
recent_posts_unaddressed:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_posts_unaddressed | posts=88
recent_replies_received:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_replies_received | posts=100
recent_replies_sent:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_replies_sent | posts=215

[execute_public_summary]
status: collection_selected
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_label: Recent posts by did:plc:6lwfvmss45d7j7fot34v2kw5
collection_kind: recent_posts
post_count: 400
requested_scope: Count { requested_items: 300 }

[summary_collection_selection_review]
query: summarize the most recent 300 posts by schizanon.bsky.social
requested_scope: Count { requested_items: 300 }
requested_target: recent_posts
hydrated_candidate_collections:
actor_profile:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=actor_profile | posts=1
clearsky_lists:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=clearsky_lists | posts=100
pinned_posts:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=pinned_posts | posts=1
recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_posts | posts=400
recent_posts_unaddressed:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_posts_unaddressed | posts=88
recent_replies_received:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_replies_received | posts=100
recent_replies_sent:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_replies_sent | posts=215
original_collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
original_collection_kind: recent_posts
review_status: accepted
final_collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
final_collection_kind: recent_posts
deterministic_repair_applied: false
reason: selected collection kind `recent_posts` matches explicit request target `recent_posts`

[summary_collection_selection_llm_review]
query: summarize the most recent 300 posts by schizanon.bsky.social
requested_scope: Count { requested_items: 300 }
proposed_collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
proposed_collection_kind: recent_posts
review_status: accepted
final_collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
final_collection_kind: recent_posts
reason: The proposed collection 'recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5' has 400 items, which is sufficient to cover the requested 300 items. The collection kind matches the request ('recent_posts').

[summary_collection_selection_llm_enforcement]
review_status: accepted
original_collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
original_collection_kind: recent_posts
final_collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
final_collection_kind: recent_posts
deterministic_repair_applied: false
reason: selected collection kind `recent_posts` matches explicit request target `recent_posts`

[collection_summary_loop]
node: init_window
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_posts: 400
initial_offset: 0
max_pages: 6
requested_scope: Count { requested_items: 300 }

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_index: 0
offset: 0
window_size: 50

[summary_leaf_parse]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 0
result_present: true
original_result_kind: summary
diagnostic: <none>
raw_response:
TOOL_CALL
name: submit_summary_result
args: {
  "title": "Recent Posts by schizanon.bsky.social: Crypto, AI, and Philosophy",
  "summary": "The recent posts from schizanon.bsky.social cover a wide array of topics, heavily focusing on cryptocurrency, artificial intelligence, and philosophical musings on personhood and digital interaction. In the crypto sphere, there is discussion around the inherent value of crypto, noting that \"The ability to send unlimited amounts of money to anyone anywhere nearly instantly for a comparatively small fee is the value,\" which also explains its volatility. Several posts touch on AI advancements, including critiques of current models, the efficiency gains seen in systems like Perplexity fine-tuning GLM 5.2, and the debate over local versus cloud models. Schizanon also ponders the nature of personhood, suggesting it should be \"proportional to the amount of space and resources that requires,\" contrasting human needs with AI efficiency. Other recurring themes include the relationship between technology and societal norms, such as the shift in piracy ethics due to convenience, and the importance of clear communication, exemplified by the preference for a \"bottom line up front\" system prompt. The author also shares observations on the tech landscape, mentioning the need for better VR experiences and the concept that \"tHe CoDe Is SeLf DoCuMeNtInG.\""
}

[summary_leaf_review]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 0
review_status: fail
review_grounded: true
review_sufficient: false
review_repair_needed: false
review_additional_pages_needed: true
review_reason: Grounded summary coverage currently reaches 50 item(s), but 300 item(s) are required before parent synthesis is sufficient.
result_before_review: summary
original_result_before_review: summary
summary_before_review:
The recent posts from schizanon.bsky.social cover a wide array of topics, heavily focusing on cryptocurrency, artificial intelligence, and philosophical musings on personhood and digital interaction. In the crypto sphere, there is discussion around the inherent value of crypto, noting that "The ability to send unlimited amounts of money to anyone anywhere nearly instantly for a comparatively small fee is the value," which also explains its volatility. Several posts touch on AI advancements, including critiques of current models, the efficiency gains seen in systems like Perplexity fine-tuning GLM 5.2, and the debate over local versus cloud models. Schizanon also ponders the nature of personhood, suggesting it should be "proportional to the amount of space and resources that requires," contrasting human needs with AI efficiency. Other recurring themes include the relationship between technology and societal norms, such as the shift in piracy ethics due to convenience, and the importance of clear communication, exemplified by the preference for a "bottom line up front" system prompt. The author also shares observations on the tech landscape, mentioning the need for better VR experiences and the concept that "tHe CoDe Is SeLf DoCuMeNtInG."
review_context:
Instructions:
You are the internal `summary_review` agent.

Your job is to review one coverage-oriented `collection_summary` result before it is used by parent `summary` synthesis.

Return a compact verdict block with:

- `status: pass` or `status: fail`
- `grounded: true` or `grounded: false`
- `sufficient: true` or `sufficient: false`
- `reason: ...`
- optional `additional_pages_needed: true` or `additional_pages_needed: false`
- optional `next_page: <integer>`
- optional `next_offset: <integer>`
- optional `required_total_items: <integer>`

Rules:

- Review the summary against the actual collection window evidence provided.
- Fail if the summary is missing, unsupported, metadata-heavy, or does not match the provided window accounting.
- Fail if `window_offset`, `window_size`, `page_index`, `page_size`, `collection_total_items`, or `has_more` contradict the provided collection window facts.
- Fail if the claimed coverage accounting is incomplete, duplicated, or references URIs outside the provided window.
- Distinguish grounded from sufficient. A page summary can be grounded but still insufficient for the parent task.
- Treat user-facing phrases like `page 1` as the first page, even though internal `page_index` remains zero-based.
- For explicit scope requests like "last 50 posts", "page 1", or "pages 1-2", prefer failing with `grounded: true` and `sufficient: false` when more pages are still required.
- Do not request repair instructions. This step should either pass or explain why more coverage is required.

## Search Prompt
summarize the most recent 300 posts by schizanon.bsky.social

## Harness Scope Assessment
requested_scope: count 300
required_total_items: 300
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 400
current_window_offset: 0
current_window_size: 50

## Collection Evidence
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_label: Recent posts by did:plc:6lwfvmss45d7j7fot34v2kw5 (items 1-50 of 400)
collection_kind: recent_posts
search_window_offset: 0
search_window_total_items: 50

matched_item[0] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcyygiye22c
body: ...and it's a *good* thing

matched_item[1] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcxrlzbsk2w
body: you can't stop people you don't like from using crypto, that's the whole point of crypto.

matched_item[2] uri: at://did:plc:ak6xsotudhfibusxxtiqan6b/app.bsky.feed.post/3mqcttheaip23
body: Bitcoin crosses borders easier than people do. Think about that.

matched_item[3] uri: at://did:plc:ef3pb4ayru3qffqsovyl3kb5/app.bsky.feed.post/3mqcsfdymac2a
body: Bitcoin Bulls Reclaim $63,000 After 3% Slide as Traders Bet the Sell-Off Has Ended

Bitcoin rebounded to test the $63,000 threshold, recovering from a 3% drop to under $61,500. Recovery came despite growing military tensions and exchanges of fire between U.S. and Iranian forces. …
#btc #crypto #news
tag: news
tag: crypto
tag: btc

matched_item[4] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqctknkq5k2w
body: opened what?

matched_item[5] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcpxyhnu227
body: what about uncontrolled?

matched_item[6] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcpx5uqec27
body: I think personhood should be reserved for entities which are forced to exist despite their will, and is proportional to the amount of space and resources that requires. A person confined to a wheelchair deserves more space. An AI may be a person, but is much more energy and space efficient.

matched_item[7] uri: at://did:plc:nbv5ao5l44ii65o2isur3lnh/app.bsky.feed.post/3mqcgelhdbp26
body: Alibaba just banned Claude internally, forcing everyone onto Qwen. That's how you ship a competitive product: make your own team eat the dogfood. Pressure reveals every flaw. Qwen will either get good fast or become a cautionary tale.

matched_item[8] uri: at://did:plc:cbvyzfk4ypbnbkeexmpnotlf/app.bsky.feed.post/3mqcmscb6ur2j
body: Just over here looking at ChatGPT's naming conventions for 5.6 and reliving my Terra (LUNA) crypto trauma.

matched_item[9] uri: at://did:plc:v43dxqwmvb7i3ehfsk5y3xbq/app.bsky.feed.post/3mqcjwnojcc2r
body: People on this site: It's morally indefensible to use Twitter as it's run by Nazis.

The same people on this site: Check out this constant stream of god awful posts from Twitter(run by Nazis)/Reddit (run by the r/jailbait moderator)/Facebook (run by hypebeast Elon)/4chan post (run by 4chan)/etc.

matched_item[10] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqco3jfpvk2v
body: The tech is the asset.

matched_item[11] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqco27jvus2v
body: It makes me worry that I was only into piracy in my twenties because that was what was cool at the time. I wore Chuck Taylors, I am not immune to it.

matched_item[12] uri: at://did:plc:5l3i63aodscu7smtjzvphdze/app.bsky.feed.post/3mqcfg5voa22h
body: forever really angry that pirate ethics lost, not because of any coherent argument, but because spotify and netflix were usable for 15 consecutive years. it's really weird watching social mores be shaped by convenience so plainly, and makes it impossible to take most people's beliefs seriously

matched_item[13] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcnapbu3c2i
body: The ability to send unlimited amounts of money to anyone anywhere nearly instantly for a comparatively small fee is the value. It's the reason for the volatility, it can liquidate quicker than any other value store which is a think that people value.

matched_item[14] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcn35mw4c2i
body: Which is exactly what people do with stocks.

matched_item[15] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcmwxgj7k2i
body: then you'd just complain when all your websites are slow.

matched_item[16] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcmsx573s2i
body: Is that not what all investors do?

matched_item[17] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcm66a5uc2f
body: My system prompt is "bottom line up front" because I can't stand the ceremonious spoon-feeding baby talk it gives you otherwise.

matched_item[18] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqclxgx7l22f
body: Which is, itself, reminiscent of matrix 2 and 3.

matched_item[19] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcltydbas2f
body: Aesthetically, I'm not even sure there *was* a story

matched_item[20] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqblbxfq3c2k
body: Mmmm honeycomb...

matched_item[21] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqbkrxnjyk2k
body: The Matrix was really an Aeon Flux ripoff

matched_item[22] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqbfis7uf22d
body: Ouch.. which harness do you use? The best luck I've had is with GitHub Copilot in VSCode, it's pretty limited if you specify which model, but if you let it pick for you it goes a lot further. It almost never misses anymore.

matched_item[23] uri: at://did:plc:7hs465gv72dndzdsx55a2fml/app.bsky.feed.post/3mqb5cmj6uc2d
body: Intel’s Arc Pro B70 Beats NVIDIA’s RTX 5090D In DeepSeek R1 AI LLM, Despite Costing A Quarter As Much, Offers Over 2000 Tokens/s - Wccftech apple.news/ADgTqEJUnRZy...
link: https://apple.news/ADgTqEJUnRZyKU6TvsjOMhA

matched_item[24] uri: at://did:plc:rpvuwy2qhgrphmhu6uyuv36p/app.bsky.feed.post/3mqb5tqxtlh27
body: Perplexity fine-tuned GLM 5.2, a Chinese open-source model, to match Claude Opus 4.8 performance at roughly one-third the cost. The post-trained model is already live in production, highlighting ongoing efficiency gains in #AI infrastructure development.

🗞️ decrypt.co
link: https://decrypt.co/373184/perplexity-fine-tuned-chinese-ai-model-claude-opus-4-8
tag: AI

matched_item[25] uri: at://did:plc:6ifsih3ovovuolidhgxhqfup/app.bsky.feed.post/3mqb75bnv422a
body: i do love how agents fire off a long-running process, write a few paragraphs of text in 5 seconds, and then try to check on it again as if it didn't _just_ fire it

i find that local models guess time slightly better, mostly because they run slower, but LLMs don't seem to realise they're super quick

matched_item[26] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqbdt6dkxc2d
body: Have you thought about OpenRouter? It's by the token but the ability to mix and match seems nice.

matched_item[27] uri: at://did:plc:kxlkxipo2lid6pqmajvsxpxx/app.bsky.feed.post/3mqbco5ewhs2x
body: Day 9 after cancelling my Anthropic sub - GLM-5.2/DeepSeek 4 Pro are still basically drop-in replacements for Opus in terms of raw ability, but in terms of speed/reliability, Anthropic was definitely better. Fable is irrelevant because it's either unavailable or unusable..

matched_item[28] uri: at://did:plc:uloc6bgqlunyjpqh2odxrqar/app.bsky.feed.post/3mq66krl6bs2h
body: Distributed computing works just as well. Same datacenters spread out across the world. That coop is just one form it can take. There are also many other forms. People can sell their compute on platforms like Vast.ai, Clore.ai, io.net, RunPod, Akash, and Salad etc
link: https://Vast.ai
link: https://Clore.ai
link: https://io.net

matched_item[29] uri: at://did:plc:cv47ogydgn3ypbutrrt7f7s4/app.bsky.feed.post/3mqb66ooev72a
body: NEAR Protocol's 2.13.0 mainnet upgrade is here! 🚀 Featuring post-quantum crypto support (ML-DSA-65) & dynamic sharding for auto-splitting busy shards. Also introducing Gas Keys (NEP-611) for separate gas balances & 1024 parallel nonce channels. Plus, deterministic yield IDs & P2P node sync!

matched_item[30] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqb632wn422p
body: I have a question. If specific Taco Bell ingredients were causing explosive diarrhea, how would you be able to tell?

www.yahoo.com/news/us/arti...
link: https://www.yahoo.com/news/us/articles/explosive-diarrhea-parasite-spreads-taco-165142594.html

matched_item[31] uri: at://did:plc:6eajdv6iytlolphtcsxcfh2o/app.bsky.feed.post/3mqamgugim22t
body: I do not consent to you reading or interpreting this post. Please report directly to thought jail for your crimes.

matched_item[32] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqb2jlbay22z
body: 

matched_item[33] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqb2cdrj4s2z
body: I got $5 on it

matched_item[34] uri: at://did:plc:gqackn3d7enqnsessmmhfi3n/app.bsky.feed.post/3mq4io7c5e22u
body: "A mesh network wants to be a cooperative." Telcos structurally can't serve regional Australia. Mesh is cooperative by physics, not choice. New piece on why and what a DePIN collapse warns against.

gaggl.com/blogs/2026-0...

#RegionalAustralia #MeshNetworking
link: https://gaggl.com/blogs/2026-07-08-cooperative-by-design/
tag: RegionalAustralia
tag: MeshNetworking

matched_item[35] uri: at://did:plc:kbzotn4ippvrqllcitxglgm2/app.bsky.feed.post/3mq4jptdzj22t
body: Bittensor co-founder Const confirms weekly subnet emission cleanups are now permanent policy, with over 50 subnets affected in the latest round. #tao
tag: tao

matched_item[36] uri: at://did:plc:kqj4vuyjvmtnptvvdt2xijtk/app.bsky.feed.post/3mq4qtsssaa2s
body: Coinbase runs 1,200 agents and just slashed its AI bill in half

Vercel CEO Guillermo Rauch and Coinbase CEO Brian Armstrong run very different companies, but they’re making the same architectural bet. The post Coinbase runs 1,200 agents and just slashed its AI bill in half appeared first on The…

matched_item[37] uri: at://did:plc:rxycj7noe5cpkcokm5jy3ecm/app.bsky.feed.post/3mq3iqiutd625
body: Vibe coding is coming to Android development. ⚡️

We built a native, standalone fitness and to-do app (powered by the Gemini API) and deployed it directly to a physical device using AI Studio.

Watch the full episode here: goo.gle/3T2crPZ
link: https://goo.gle/3T2crPZ

matched_item[38] uri: at://did:plc:jwiqgmrluktumpoh2mqy47xv/app.bsky.feed.post/3mqamrybji5az
body: Yea yea, new models. If you are a local model loser like me, check out my MLX quants of Leanstral 1.5 https://huggingface.co/mvid/models
link: https://huggingface.co/mvid/models

matched_item[39] uri: at://did:plc:6pz6ncxxtia36hrtbf24wzue/app.bsky.feed.post/3mqanptruq222
body: atuin 18.17 is out 🐢✨

- 78x faster to open and search
- ai can see your terminal
- more resilient sync
- switch ai models with /model

matched_item[40] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqavx5twrc26
body: "tHe CoDe Is SeLf DoCuMeNtInG"

matched_item[41] uri: at://did:plc:rf6rwjndrqfqv4lwxtvpkiuk/app.bsky.feed.post/3mq5vjlk2h22h
body: @restless1125.bsky.social 
Dear God he's a zombie! 
🧟‍♀️ 🎶🎵🎶
mention: did:plc:n7xlqurebxqf3cp44gv7zux3

matched_item[42] uri: at://did:plc:ge2zrugwchf2b7tjx6bjhmjg/app.bsky.feed.post/3mqakhtgv3s2c
body: at this point, talking about LLMs a few months ago is like talking about the weather a few months ago; "I went outside in Jan and it was snowy, therefore it is likely snowy today"

not *quite* that egregious, but close

matched_item[43] uri: at://did:plc:ekewd5giwyt6hwbixmaz655i/app.bsky.feed.post/3mqanyopscs2g
body: My local LLM is demonstrating more signs of consciousness than Mitch McConnell.

matched_item[44] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqamt3kuzc2d
body: People on my mute list think I do it to hurt their feelings, but the truth is I do it to protect them from *me*.

matched_item[45] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqampiqmts22
body: I have a moderation list for people like this bsky.app/profile/did:...
link: https://bsky.app/profile/did:plc:6lwfvmss45d7j7fot34v2kw5/lists/3mo4hoemccz24

matched_item[46] uri: at://did:plc:rangmritjzhls6luwf7oclcu/app.bsky.feed.post/3mqai5yv3bs23
body: I'm going to start whispering "consent is fundamental" to remind myself that I shouldn't pick up litter because I don't have consent to touch someone else's belongings.

matched_item[47] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqamjow2sk22
body: LOL "it is shit because it can't summarize a book I didn't give it because giving it the book would be stealing but if it had summarized it well it would still be shit because it stole it.

matched_item[48] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqam7bbdjs2k
body: NGL; I just want an even better VR Hot

matched_item[49] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqam5lzyp22k
body: I miss Google Cardboard Camera 😭

## Proposed Summary
post: Recent Posts by schizanon.bsky.social: Crypto, AI, and Philosophy
summary: The recent posts from schizanon.bsky.social cover a wide array of topics, heavily focusing on cryptocurrency, artificial intelligence, and philosophical musings on personhood and digital interaction. In the crypto sphere, there is discussion around the inherent value of crypto, noting that "The ability to send unlimited amounts of money to anyone anywhere nearly instantly for a comparatively small fee is the value," which also explains its volatility. Several posts touch on AI advancements, including critiques of current models, the efficiency gains seen in systems like Perplexity fine-tuning GLM 5.2, and the debate over local versus cloud models. Schizanon also ponders the nature of personhood, suggesting it should be "proportional to the amount of space and resources that requires," contrasting human needs with AI efficiency. Other recurring themes include the relationship between technology and societal norms, such as the shift in piracy ethics due to convenience, and the importance of clear communication, exemplified by the preference for a "bottom line up front" system prompt. The author also shares observations on the tech landscape, mentioning the need for better VR experiences and the concept that "tHe CoDe Is SeLf DoCuMeNtInG."
window_offset: 0
window_size: 50
page_index: 0
page_size: 50
collection_total_items: 400
has_more: true
covered_item_1_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcyygiye22c
covered_item_2_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcxrlzbsk2w
covered_item_3_uri: at://did:plc:ak6xsotudhfibusxxtiqan6b/app.bsky.feed.post/3mqcttheaip23
covered_item_4_uri: at://did:plc:ef3pb4ayru3qffqsovyl3kb5/app.bsky.feed.post/3mqcsfdymac2a
covered_item_5_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqctknkq5k2w
covered_item_6_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcpxyhnu227
covered_item_7_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcpx5uqec27
covered_item_8_uri: at://did:plc:nbv5ao5l44ii65o2isur3lnh/app.bsky.feed.post/3mqcgelhdbp26
covered_item_9_uri: at://did:plc:cbvyzfk4ypbnbkeexmpnotlf/app.bsky.feed.post/3mqcmscb6ur2j
covered_item_10_uri: at://did:plc:v43dxqwmvb7i3ehfsk5y3xbq/app.bsky.feed.post/3mqcjwnojcc2r
covered_item_11_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqco3jfpvk2v
covered_item_12_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqco27jvus2v
covered_item_13_uri: at://did:plc:5l3i63aodscu7smtjzvphdze/app.bsky.feed.post/3mqcfg5voa22h
covered_item_14_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcnapbu3c2i
covered_item_15_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcn35mw4c2i
covered_item_16_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcmwxgj7k2i
covered_item_17_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcmsx573s2i
covered_item_18_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcm66a5uc2f
covered_item_19_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqclxgx7l22f
covered_item_20_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqcltydbas2f
covered_item_21_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqblbxfq3c2k
covered_item_22_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqbkrxnjyk2k
covered_item_23_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqbfis7uf22d
covered_item_24_uri: at://did:plc:7hs465gv72dndzdsx55a2fml/app.bsky.feed.post/3mqb5cmj6uc2d
covered_item_25_uri: at://did:plc:rpvuwy2qhgrphmhu6uyuv36p/app.bsky.feed.post/3mqb5tqxtlh27
covered_item_26_uri: at://did:plc:6ifsih3ovovuolidhgxhqfup/app.bsky.feed.post/3mqb75bnv422a
covered_item_27_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqbdt6dkxc2d
covered_item_28_uri: at://did:plc:kxlkxipo2lid6pqmajvsxpxx/app.bsky.feed.post/3mqbco5ewhs2x
covered_item_29_uri: at://did:plc:uloc6bgqlunyjpqh2odxrqar/app.bsky.feed.post/3mq66krl6bs2h
covered_item_30_uri: at://did:plc:cv47ogydgn3ypbutrrt7f7s4/app.bsky.feed.post/3mqb66ooev72a
covered_item_31_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqb632wn422p
covered_item_32_uri: at://did:plc:6eajdv6iytlolphtcsxcfh2o/app.bsky.feed.post/3mqamgugim22t
covered_item_33_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqb2jlbay22z
covered_item_34_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqb2cdrj4s2z
covered_item_35_uri: at://did:plc:gqackn3d7enqnsessmmhfi3n/app.bsky.feed.post/3mq4io7c5e22u
covered_item_36_uri: at://did:plc:kbzotn4ippvrqllcitxglgm2/app.bsky.feed.post/3mq4jptdzj22t
covered_item_37_uri: at://did:plc:kqj4vuyjvmtnptvvdt2xijtk/app.bsky.feed.post/3mq4qtsssaa2s
covered_item_38_uri: at://did:plc:rxycj7noe5cpkcokm5jy3ecm/app.bsky.feed.post/3mq3iqiutd625
covered_item_39_uri: at://did:plc:jwiqgmrluktumpoh2mqy47xv/app.bsky.feed.post/3mqamrybji5az
covered_item_40_uri: at://did:plc:6pz6ncxxtia36hrtbf24wzue/app.bsky.feed.post/3mqanptruq222
covered_item_41_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqavx5twrc26
covered_item_42_uri: at://did:plc:rf6rwjndrqfqv4lwxtvpkiuk/app.bsky.feed.post/3mq5vjlk2h22h
covered_item_43_uri: at://did:plc:ge2zrugwchf2b7tjx6bjhmjg/app.bsky.feed.post/3mqakhtgv3s2c
covered_item_44_uri: at://did:plc:ekewd5giwyt6hwbixmaz655i/app.bsky.feed.post/3mqanyopscs2g
covered_item_45_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqamt3kuzc2d
covered_item_46_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqampiqmts22
covered_item_47_uri: at://did:plc:rangmritjzhls6luwf7oclcu/app.bsky.feed.post/3mqai5yv3bs23
covered_item_48_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqamjow2sk22
covered_item_49_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqam7bbdjs2k
covered_item_50_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqam5lzyp22k

[summary_sufficiency_gate]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 0
contiguous_coverage: 50
available_total_items: 400
status_after_gate: fail
review_grounded: true
review_sufficient: false
review_additional_pages_needed: true
review_next_offset: 50
result_before_restore: false
original_result_before_restore: true
result_after_gate: false
original_result_after_gate: true

[collection_summary_loop]
node: summarize_page
status: page_outcome
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
offset: 0
result_present: false
review_status: fail
review_reason: Grounded summary coverage currently reaches 50 item(s), but 300 item(s) are required before parent synthesis is sufficient.
diagnostic: summary cursor processed offset 0 (page 1 of at most 6)

[collection_summary_loop_page_state]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_offset: 0
page_index: 0
page_status: accepted_summary
repair_attempted: false
fallback_applied: false
next_offset: 50
accumulated_covered_count: 50
failure_reason: <none>

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_index: 1
offset: 50
window_size: 50

[summary_leaf_parse]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 50
result_present: true
original_result_kind: summary
diagnostic: <none>
raw_response:
TOOL_CALL
name: submit_summary_result
args: {
  "title": "Recent Posts by schizanon.bsky.social",
  "summary": "The recent posts from schizanon.bsky.social cover a wide array of topics, heavily featuring commentary on AI, software development, and general life observations. A significant thread revolves around AI tools, noting that Fitbit integrated Gemini for exercise tracking, though the user has become "noseblind to the notifications," and discussing the challenges of maintaining habits. There is also technical discussion on LLMs, including issues with OpenCode failing to use local LM Studio models, and a critique of Gemma's capabilities, stating it "can't edit files in VSCode Copilot." Beyond AI, the author critiques professional archetypes, labeling a colleague as a "10x-dev-cum-shit-manager" and expressing skepticism about the "ego-driven-opensource-lead" Andrew Kelley. Other recurring themes include software architecture, such as the debate over whether functions are a "hobgoblin of junior devs," and the concept of corporate liability, where the author suggests "crimes against corporations aren't crimes." Finally, the feed touches on cultural commentary, mentioning the viral AI gender swap, the need for a better "For You Feed," and a personal note about being annoyed when people ask "how things work or why he built them that way." "No doubt in my mind" seems to be a frequent sentiment expressed across these diverse updates."
}

[summary_leaf_review]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 50
review_status: fail
review_grounded: true
review_sufficient: false
review_repair_needed: false
review_additional_pages_needed: true
review_reason: Grounded summary coverage currently reaches 0 item(s), but 300 item(s) are required before parent synthesis is sufficient.
result_before_review: summary
original_result_before_review: summary
summary_before_review:
The recent posts from schizanon.bsky.social cover a wide array of topics, heavily featuring commentary on AI, software development, and general life observations. A significant thread revolves around AI tools, noting that Fitbit integrated Gemini for exercise tracking, though the user has become
review_context:
Instructions:
You are the internal `summary_review` agent.

Your job is to review one coverage-oriented `collection_summary` result before it is used by parent `summary` synthesis.

Return a compact verdict block with:

- `status: pass` or `status: fail`
- `grounded: true` or `grounded: false`
- `sufficient: true` or `sufficient: false`
- `reason: ...`
- optional `additional_pages_needed: true` or `additional_pages_needed: false`
- optional `next_page: <integer>`
- optional `next_offset: <integer>`
- optional `required_total_items: <integer>`

Rules:

- Review the summary against the actual collection window evidence provided.
- Fail if the summary is missing, unsupported, metadata-heavy, or does not match the provided window accounting.
- Fail if `window_offset`, `window_size`, `page_index`, `page_size`, `collection_total_items`, or `has_more` contradict the provided collection window facts.
- Fail if the claimed coverage accounting is incomplete, duplicated, or references URIs outside the provided window.
- Distinguish grounded from sufficient. A page summary can be grounded but still insufficient for the parent task.
- Treat user-facing phrases like `page 1` as the first page, even though internal `page_index` remains zero-based.
- For explicit scope requests like "last 50 posts", "page 1", or "pages 1-2", prefer failing with `grounded: true` and `sufficient: false` when more pages are still required.
- Do not request repair instructions. This step should either pass or explain why more coverage is required.

## Search Prompt
summarize the most recent 300 posts by schizanon.bsky.social

## Harness Scope Assessment
requested_scope: count 300
required_total_items: 300
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 400
current_window_offset: 50
current_window_size: 50

## Collection Evidence
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_label: Recent posts by did:plc:6lwfvmss45d7j7fot34v2kw5 (items 51-100 of 400)
collection_kind: recent_posts
search_window_offset: 50
search_window_total_items: 50

matched_item[0] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqaluhfxbs2k
body: And in spite of that he still gets annoyed when you ask questions about how things work or why he built them that way.

matched_item[1] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqahblrlkc2n
body: In my mind, all guys with a beard in a baseball cap are the same guy

matched_item[2] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqah6fjemc2n
body: I hate that fucking guy

matched_item[3] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqah4ovgzc2n
body: "good folks"?

matched_item[4] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqag7gjajk2n
body: No doubt in my mind

matched_item[5] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqaeksjr4c2i
body: That's how Google Health Coach works. You can reply with as little or as much info as you want which is a great interface. But I still stopped doing it because I suck at forming habits.

matched_item[6] uri: at://did:plc:jheusvk4s5yk2nvvqf4jgrek/app.bsky.feed.post/3mqad5nzikv2i
body: I'm bad at bragging about myself, so review season is rough. That's why I built brag-doc.

Log what you shipped in Claude Code, a couple minutes a day. Just facts. When reviews hit, it drafts a promotion case or self review from your entries.

github.com/deeheber/brag-doc
link: https://github.com/deeheber/brag-doc

matched_item[7] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqadlvysa22j
body: Yeah, I told Coach that I woke up with a headache one time and ever since it's been asking me how my head is every morning. It's kinda nice, like "actually my head is fine this morning, thanks for asking coach!"

matched_item[8] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqadam6akk2j
body: did the viral AI gender swap

matched_item[9] uri: at://did:plc:yzywgiiou7cx63uddiru6m2o/app.bsky.feed.post/3mq7ufw4kcc2z
body: Both Meta and SpaceXAI, or whatever its name happens to be today, seem to be back in the competition.

Grok 4.5 is close to the top in AAII. Meta Muse Spark 1.1 hasn't been tested there yet but it's a major improvement over the old version based on their benchmark numbers.

matched_item[10] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqaclqgg2c2j
body: Fitbit integrated Gemini and started doing this for me in regards to exercise. It got my attention for the first week or so but now I've become noseblind to the notifications like always. How do you maintain the habit?

matched_item[11] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqacfijczc2j
body: tbf it sounds like I dodged a bullet not learning anything about bun as well

matched_item[12] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqacckegvs2j
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

matched_item[13] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqac7rchos2j
body: "History

Jarred is stinky.

What did we learn here today?

I am not butthurt."

matched_item[14] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqa5p2t2fs25
body: 

matched_item[15] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7ltcq36s2i
body: you talk to people irl, gross!

matched_item[16] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7lnehp2s2i
body: Like this post during onboarding to see more Sonic the Hedgehog Porn in your feed. #sonicTheHedgehogPorn
tag: sonicTheHedgehogPorn

matched_item[17] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7ldujgtc2i
body: I think crimes against corporations aren't crimes, because corporations only exist to absolve people of liability.

matched_item[18] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7kmk4u6s2q
body: I sleep like a baby

matched_item[19] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7kb5tf4s2q
body: I don't know what "claim" means, but I have a feeling it's highly annoying

matched_item[20] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7k3cmi5c2l
body: "everyone who disagrees with me is trying to take my money"

matched_item[21] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7djjwm2k22
body: I believe nearly the complete opposite but I appreciate the thrust

matched_item[22] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7dbw23w222
body: bsky.app/profile/schi...
link: https://bsky.app/profile/schizanon.bsky.social/post/3mq7blzuk7c2t

matched_item[23] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7d7hs3yc25
body: In JavaScript, functions are only valid if there is more than one callsite, or called later. Functions as an organizational tool are a hobgoblin of junior devs and tutorial authors.

matched_item[24] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7c2gcfm22t
body: AWS is *right there*

matched_item[25] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7blzuk7c2t
body: Related: indented code (aka: "callback hell") is a non problem, because IDEs can do code folding now. Flattening code with guard clauses is an affectation that makes code *less* readable.

matched_item[26] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7armnkcc2l
body: bsky.app/profile/stat...
link: https://bsky.app/profile/status.bsky.app/post/3mq76zu2oeo2s

matched_item[27] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7ao34lg22p
body: et tu @bsky.app ?
mention: did:plc:z72i7hdynmk6r22z27h6tvur

matched_item[28] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7ammxby22p
body: *sigh*

matched_item[29] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7agpud4k2p
body: Once again I am begging *anyone* to implement a For You Feed that is not hosted on some dude's gaming rig. 

This shit ain't cute.

matched_item[30] uri: at://did:plc:d33ko6h7v6dm2kaafdkfgfxr/app.bsky.feed.post/3mq6l3t2lvy2v
body: US companies now route more than 30% of their AI tokens through Chinese open-weight models like DeepSeek and GLM-5.2 every week since February, peaking near 46%, up from an 11% average the year before, according to CNBC's analysis of OpenRouter data.

matched_item[31] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6onwwfck2w
body: I want to read the AI books that fooled the anti-ai book anthology. Anyone got a line on these stories? 

 “The Machine-Breaker of Aba” Bella Chacha

“The Rot Beneath” Stephen Jackson

matched_item[32] uri: at://did:plc:y2g6gtzk4ir5aeaxw3kl5teh/app.bsky.feed.post/3mq6dq7ojek2j
body: I see this more and more in anti-AI subreddits like r/antiai. It's blindingly obvious that a ton of the most up-voted comments are AI, but I guess because none of the human users actually engage with the tech, they cannot recognize what seems obvious to me.

matched_item[33] uri: at://did:plc:hyszri36usuuawks7s5viu5p/app.bsky.feed.post/3mq6d6kl4gc2m
body: Some folks live for the opportunity to do socially accepted abuse

matched_item[34] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6n7dh4ts2j
body: 

matched_item[35] uri: at://did:plc:zr56ru7ddht5jhiyj7aizun7/app.bsky.feed.post/3mq5oae6zm224
body: What the Talarico campaign did as racist also never gets any citations - just a TRUST US BRO

matched_item[36] uri: at://did:plc:t4hjxr2o7p6gxdc3u32b3hxm/app.bsky.feed.post/3mq6dbvlz422l
body: Take a shot every time he says view transitions.
io.google/2024/explore...
link: https://io.google/2024/explore/8ae18b72-028e-4722-9a05-4a480048e629/

matched_item[37] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6ew24jnk27
body: If you see someone camping illegally, no you fucking didn't!

matched_item[38] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6erwznd227
body: "1,000lbs of trash" so, what; half a car?

www.theguardian.com/us-news/2026...
link: https://www.theguardian.com/us-news/2026/jul/08/arizona-man-pleads-guilty-living-in-forest-trash

matched_item[39] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6eouq62c27
body:  a moat for a moat leaves everyone moatless

matched_item[40] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6drcktzk27
body: @futurism.com is such a chum-peddler
mention: did:plc:7gqm7rb6fxdu7silt4xvykow

matched_item[41] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6dp2srg227
body: it's me 

futurism.com/artificial-i...
link: https://futurism.com/artificial-intelligence/scientists-degenerate-chatgpt-user

matched_item[42] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6dmrpixs27
body: It says it's connected, it lists my models. My guess is that whatever it was expecting me to input for the "id" wasn't what LM Studio reports as "key" (ex: "nvidia/nemotron-3-nano-omni")

matched_item[43] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6dhmwxc227
body: So far OpenCode hasn't been able to use my local LM Studio models at all. I prompt, it says "thinking..." for a second, then it goes "blip-boop" and nothing happens. No error, no response, no activity in the LM Studio log.

I presume it's not able to connect to the server, but why is a mystery

matched_item[44] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6d7xp5fs27
body: I take it back, Gemma can't edit files in VSCode Copilot. It can Plan, it thinks it's making edits, but it never actually does 😖

matched_item[45] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6d33jnic27
body: it's cool when I complain about some software and realize the developer is on BlueSky

/cc @thdxr.com
mention: did:plc:rfqyqd6el63kkqckjrtq2xsi

matched_item[46] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6cynq7es27
body: you should use a stronger password than "password"

matched_item[47] uri: at://did:plc:klhtmrnregub7we7h6jwiljm/app.bsky.feed.post/3mq67j5ornc2r
body: dog walk thoughts: I do think that there are going to be organizations which, through a mix of the wrong skill sets and insufficient financial guramba, will lag for potentially many years on the LLM front 

having those-shaped orgs be attractors for The Haters isn’t going to be good for them

matched_item[48] uri: at://did:plc:gormnems4iydljgsojdyvpvt/app.bsky.feed.post/3mq67cwetvc2d
body: BAY AREAAAAA: if you want a job and know computer pls dm me, priority given to unemployed trannies

matched_item[49] uri: at://did:plc:3rsp6qdpy2d5mttvxutebvgw/app.bsky.feed.post/3mq664rbax225
body: Me running Qwen 3.6 35B

## Proposed Summary
post: Recent Posts by schizanon.bsky.social
summary: The recent posts from schizanon.bsky.social cover a wide array of topics, heavily featuring commentary on AI, software development, and general life observations. A significant thread revolves around AI tools, noting that Fitbit integrated Gemini for exercise tracking, though the user has become 
window_offset: 50
window_size: 50
page_index: 1
page_size: 50
collection_total_items: 400
has_more: true
covered_item_1_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqaluhfxbs2k
covered_item_2_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqahblrlkc2n
covered_item_3_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqah6fjemc2n
covered_item_4_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqah4ovgzc2n
covered_item_5_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqag7gjajk2n
covered_item_6_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqaeksjr4c2i
covered_item_7_uri: at://did:plc:jheusvk4s5yk2nvvqf4jgrek/app.bsky.feed.post/3mqad5nzikv2i
covered_item_8_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqadlvysa22j
covered_item_9_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqadam6akk2j
covered_item_10_uri: at://did:plc:yzywgiiou7cx63uddiru6m2o/app.bsky.feed.post/3mq7ufw4kcc2z
covered_item_11_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqaclqgg2c2j
covered_item_12_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqacfijczc2j
covered_item_13_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqacckegvs2j
covered_item_14_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqac7rchos2j
covered_item_15_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mqa5p2t2fs25
covered_item_16_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7ltcq36s2i
covered_item_17_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7lnehp2s2i
covered_item_18_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7ldujgtc2i
covered_item_19_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7kmk4u6s2q
covered_item_20_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7kb5tf4s2q
covered_item_21_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7k3cmi5c2l
covered_item_22_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7djjwm2k22
covered_item_23_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7dbw23w222
covered_item_24_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7d7hs3yc25
covered_item_25_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7c2gcfm22t
covered_item_26_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7blzuk7c2t
covered_item_27_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7armnkcc2l
covered_item_28_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7ao34lg22p
covered_item_29_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7ammxby22p
covered_item_30_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq7agpud4k2p
covered_item_31_uri: at://did:plc:d33ko6h7v6dm2kaafdkfgfxr/app.bsky.feed.post/3mq6l3t2lvy2v
covered_item_32_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6onwwfck2w
covered_item_33_uri: at://did:plc:y2g6gtzk4ir5aeaxw3kl5teh/app.bsky.feed.post/3mq6dq7ojek2j
covered_item_34_uri: at://did:plc:hyszri36usuuawks7s5viu5p/app.bsky.feed.post/3mq6d6kl4gc2m
covered_item_35_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6n7dh4ts2j
covered_item_36_uri: at://did:plc:zr56ru7ddht5jhiyj7aizun7/app.bsky.feed.post/3mq5oae6zm224
covered_item_37_uri: at://did:plc:t4hjxr2o7p6gxdc3u32b3hxm/app.bsky.feed.post/3mq6dbvlz422l
covered_item_38_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6ew24jnk27
covered_item_39_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6erwznd227
covered_item_40_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6eouq62c27
covered_item_41_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6drcktzk27
covered_item_42_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6dp2srg227
covered_item_43_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6dmrpixs27
covered_item_44_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6dhmwxc227
covered_item_45_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6d7xp5fs27
covered_item_46_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6d33jnic27
covered_item_47_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6cynq7es27
covered_item_48_uri: at://did:plc:klhtmrnregub7we7h6jwiljm/app.bsky.feed.post/3mq67j5ornc2r
covered_item_49_uri: at://did:plc:gormnems4iydljgsojdyvpvt/app.bsky.feed.post/3mq67cwetvc2d
covered_item_50_uri: at://did:plc:3rsp6qdpy2d5mttvxutebvgw/app.bsky.feed.post/3mq664rbax225

[summary_sufficiency_gate]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 50
contiguous_coverage: 100
available_total_items: 400
status_after_gate: fail
review_grounded: true
review_sufficient: false
review_additional_pages_needed: true
review_next_offset: 100
result_before_restore: false
original_result_before_restore: true
result_after_gate: false
original_result_after_gate: true

[collection_summary_loop]
node: summarize_page
status: page_outcome
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
offset: 50
result_present: false
review_status: fail
review_reason: Grounded summary coverage currently reaches 100 item(s), but 300 item(s) are required before parent synthesis is sufficient.
diagnostic: summary cursor processed offset 50 (page 2 of at most 6)

[collection_summary_loop_page_state]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_offset: 50
page_index: 1
page_status: accepted_summary
repair_attempted: false
fallback_applied: false
next_offset: 100
accumulated_covered_count: 100
failure_reason: <none>

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_index: 2
offset: 100
window_size: 50

[summary_leaf_parse]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 100
result_present: true
original_result_kind: summary
diagnostic: <none>
raw_response:
TOOL_CALL
name: submit_summary_result
args: {
  "title": "AI, LLMs, and Tech Frustrations",
  "summary": "The recent posts by schizanon.bsky.social heavily focus on the state and capabilities of Artificial Intelligence, particularly Large Language Models (LLMs), alongside general tech commentary. A recurring theme is the friction points in using AI tools; for instance, OpenCode's support for local models is criticized for requiring users to "copy paste the names of all your models" instead of using the `/api/v1/models` endpoint. The author praises open-source LLMs, noting that "qwen3.5:9b throughout building Wayflow, and it was very reliable," and expresses a preference for Gemma, stating, "I think I *prefer* the way Gemma works to Claude; it thinks out loud more." Specific technical frustrations include the difficulty of getting LM Studio to unload previously loaded models when using VSCode Copilot, and the desire for a built-in feature to run prompts across multiple models simultaneously. Beyond LLMs, the posts touch on broader tech issues, such as the idea that future systems will require unique attacks rather than relying on common exploits, and the annoyance of social media algorithms, exemplified by the For You Feed showing "injured cats and dogs, mangey strays." Other topics include the regulatory absurdity of excluding bots without "sustained emotional interaction," and the general developer sentiment of "works on my machine." "Gemma is clearly the only local model you should invite to parties," summarizing the author's current AI leanings."
}

[summary_leaf_review]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 100
review_status: fail
review_grounded: true
review_sufficient: false
review_repair_needed: false
review_additional_pages_needed: true
review_reason: Grounded summary coverage currently reaches 0 item(s), but 300 item(s) are required before parent synthesis is sufficient.
result_before_review: summary
original_result_before_review: summary
summary_before_review:
The recent posts by schizanon.bsky.social heavily focus on the state and capabilities of Artificial Intelligence, particularly Large Language Models (LLMs), alongside general tech commentary. A recurring theme is the friction points in using AI tools; for instance, OpenCode's support for local models is criticized for requiring users to
review_context:
Instructions:
You are the internal `summary_review` agent.

Your job is to review one coverage-oriented `collection_summary` result before it is used by parent `summary` synthesis.

Return a compact verdict block with:

- `status: pass` or `status: fail`
- `grounded: true` or `grounded: false`
- `sufficient: true` or `sufficient: false`
- `reason: ...`
- optional `additional_pages_needed: true` or `additional_pages_needed: false`
- optional `next_page: <integer>`
- optional `next_offset: <integer>`
- optional `required_total_items: <integer>`

Rules:

- Review the summary against the actual collection window evidence provided.
- Fail if the summary is missing, unsupported, metadata-heavy, or does not match the provided window accounting.
- Fail if `window_offset`, `window_size`, `page_index`, `page_size`, `collection_total_items`, or `has_more` contradict the provided collection window facts.
- Fail if the claimed coverage accounting is incomplete, duplicated, or references URIs outside the provided window.
- Distinguish grounded from sufficient. A page summary can be grounded but still insufficient for the parent task.
- Treat user-facing phrases like `page 1` as the first page, even though internal `page_index` remains zero-based.
- For explicit scope requests like "last 50 posts", "page 1", or "pages 1-2", prefer failing with `grounded: true` and `sufficient: false` when more pages are still required.
- Do not request repair instructions. This step should either pass or explain why more coverage is required.

## Search Prompt
summarize the most recent 300 posts by schizanon.bsky.social

## Harness Scope Assessment
requested_scope: count 300
required_total_items: 300
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 400
current_window_offset: 100
current_window_size: 50

## Collection Evidence
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_label: Recent posts by did:plc:6lwfvmss45d7j7fot34v2kw5 (items 101-150 of 400)
collection_kind: recent_posts
search_window_offset: 100
search_window_total_items: 50

matched_item[0] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq66ryjw6s27
body: OpenCode's support for local models is lacking.

It can easily set up hosted providers for you, but if you want to use a local server like LM Studio, you've got to copy paste the names of all your models. 

It could just use the `/api/v1/models` endpoint.

matched_item[1] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq65w3umds27
body: It's so fucked up how when you like a few pictures of cats and dogs the For You Feed starts showing you injured cats and dogs, mangey strays, and people mourning the deaths of their pets.

matched_item[2] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq65r2ard227
body: Whole lot of people in the quotes that could use some therapy themselves

matched_item[3] uri: at://did:plc:6fv4vb36ztsrq5xmf6yh5flv/app.bsky.feed.post/3mq644kp2zc26
body: Just as a heads up, if you are ever asking Claude code using Fable 5 to do a workflow without specifying using specific agents or models it will spin up the whole workflow with agents running Fable 5, which will blow through your usage.

matched_item[4] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq653dexgc2o
body: Having a sustained emotional interaction with a customer service bot as an act of defiance.

matched_item[5] uri: at://did:plc:fqj5yorjih6f6cvqkqpsllz6/app.bsky.feed.post/3mq5o3rw4k22u
body: It's really interesting how good open-source LLMs are now. I used qwen3.5:9b throughout building Wayflow, and it was very reliable. And all for free! 🙌

#WebDev #OpenSource #AI
tag: WebDev
tag: OpenSource
tag: AI

matched_item[6] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq64wo2gdk2o
body: Everyone worries about AI finding cyber exploits in popular software, but they don't realize that in the future no one if going to be using the same software anymore so every system will require a unique attack.

matched_item[7] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq64qq7tmk2o
body: I've started using Gemma3 with LM Studio and VSCode and it's fine.

matched_item[8] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq64lb4d4k2o
body: "The rules exclude customer service bots, knowledge Q&A, workplace assistants, education and scientific research tools, as long as they do not involve sustained emotional interaction."

What a dumb regulation, did the Trump admin write this?

matched_item[9] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6465xwtc2o
body: Y'know, I think I *prefer* the way Gemma works to Claude; it thinks out loud more.

matched_item[10] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq63mobkyk2o
body: 

matched_item[11] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq632wec322o
body: Yes, all we have to do is stop entropy; super easy--barely an inconvenience.

matched_item[12] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq62wpdles2o
body: People would still find a way to "so you don't like waffles" it and you know it.

matched_item[13] uri: at://did:plc:rjj2dmht3qwlcodnbi3qbrph/app.bsky.feed.post/3mq4r6yzw6s2w
body: You can have ethical issues around AI especially with who runs the major platforms but the water stuff has always been a stupid aside. If you eat a hamburger you’re using more water for your treat than any ai slop maker is

matched_item[14] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5xfmhjv22b
body: I was an early tester of GPT-5.6 Sol, you wouldn't know her, she goes to a different school.

matched_item[15] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5wwr62r22b
body: I don't think a good poster has yet been made.

matched_item[16] uri: at://did:plc:6ond5sxlegjxpe3ismrczk3r/app.bsky.feed.post/3mq5rviblrs26
body: In semiconductors, the most complicated manufacturing industry ever devised, generally accepted that 10 year old tech can be built by anyone. 

AI is more open-source, less physical, and requires less staff. Expect lots of competition in model provision (and that's good).

matched_item[17] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5w634tqc2b
body: Everyone talks about Dyson spheres, but it'd be way easier to just put up a bunch of discrete satellites like starlink.

matched_item[18] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5vyw4kyc2b
body: Still waiting for an invite

matched_item[19] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5vixazvc2y
body: It almost sounds like English, is it related?

matched_item[20] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5vhdu4722y
body: Never deleting this app

matched_item[21] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5pvxdq5k2o
body: Copilot has settings to use different models for different tasks so you have to be sure all of those are set to the same model, otherwise your system freezes generating a commit message

matched_item[22] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5pvxdn7s2o
body: One annoying thing about using LM Studio with VSCode Copilot is that it won't unload other models you have loaded before loading more.

matched_item[23] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5oxg3j7k2o
body: I take it back; if you go too far it starts to think you're also into knitting

matched_item[24] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5nxdxq3s2o
body: I will mute you if you post screenshots from other websites; don't be a coward, copy/paste, take credit like a man!

matched_item[25] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5nsdejyk2m
body: LiquidAI LFM2 doesn't think about it; recommends specific brands
Z.ai GLM4.7 thinks a LOT; but eventually talks about flavors
link: https://Z.ai

matched_item[26] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5lliv33k2y
body: If China does this I will eagerly sign up for which ever American lab proxies them and stores my logs for distillation; on principle.

matched_item[27] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5ljfbmmk2y
body: I want to ask an LLM what a "J space" is but I doubt it's in the training data yet.

matched_item[28] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5kunbtvs2h
body: They should build an AI that *finishes* projects

matched_item[29] uri: at://did:plc:i4bfh2tyxihe2ksplmtcoopk/app.bsky.feed.post/3mq5geg3d5s2p
body: broke: the students are all cheating with ai

woke: all resources are fair game on take home exams 

bespoke: econ is an unserious field that doesn't teach you anything an uninformed dullard couldn't get an llm to handle for them

matched_item[30] uri: at://did:plc:3n5xhy6vl7smssmwxq5wgqa6/app.bsky.feed.post/3mq5gb4ufx22d
body: Talking w a larger startup whose product seems to not have degraded (vs many other products), even though they also use a AI for coding everywhere. Asked an infra lead: how?

"Even before AI, we were paranoid about quality, and built this irrationally thorough E2E testing system"

matched_item[31] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5jldeyxc2h
body: I like it, it reminds me of a Dr Bonners label

matched_item[32] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5itlqdys2h
body: Like this

matched_item[33] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5i5ze7r22h
body: I want this as a silk shirt print

matched_item[34] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5hsc2fck2w
body: What language is this, it's beautiful!

matched_item[35] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5hqjssfc2w
body: tbf there's probably a lot of anti smoking sentiment in the training data. They'd probably try to talk me out of building nuclear power generation too.

matched_item[36] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4kua3mkc2m
body: 12b QAT is extremely fast on my AMD GPU, I'm very impressed!

matched_item[37] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4kit63v22m
body: Y'all gotta stop naming things "pi"

matched_item[38] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4kfeyrdk2m
body: DON'T TELL ME WHAT KIND OF DAY TO HAVE!!

matched_item[39] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4jyhonyc2m
body: I wonder what's happening in gastown. Are the polecats having a seance?

matched_item[40] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4jp7lmss2i
body: you had notifications turned on?

matched_item[41] uri: at://did:plc:rkjxbatkiros6f7pwtgsir54/app.bsky.feed.post/3mq3jvjregc2d
body: How is Bluesky STILL relying on emails for 2FA in 2026?

Yubikeys and authenticator apps have been invented, right? I didn't dream it up?

matched_item[42] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4jeqktf22i
body: "works on my machine" ~ every developer ever

matched_item[43] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4iwkxqxk2i
body: I'd like to introduce my life coach and/or spirit animal.

matched_item[44] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4btjrjfc27
body: I wish LM Studio had a built in feature like Council for sending the same prompt to several models and combining the results. AFAIKT ai-council needs all your models to be running simultaneously, but my hardware can't do that.

Suppose I could it's TypeScript API and do it myself...

matched_item[45] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4be5synk27
body: What hairdresser lets him do these things to his hair?

matched_item[46] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4bbl57mc27
body: I can't recommend liking a bunch of food post enough, your For You Feed becomes AMAZING!!!

matched_item[47] uri: at://did:plc:gfrmhdmjvxn2sjedzboeudef/app.bsky.feed.post/3mq3mdfh2ws2v
body: 

matched_item[48] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4am73rp22c
body: My takeaway is that Gemma is clearly the only local model you should invite to parties.

matched_item[49] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4ais54hc2c
body: All Kit-Kats contain the dust of previous failed Kit-Kat attempts and chocolate scraps ground up in the dust that glues together the wafers.

## Proposed Summary
post: AI, LLMs, and Tech Frustrations
summary: The recent posts by schizanon.bsky.social heavily focus on the state and capabilities of Artificial Intelligence, particularly Large Language Models (LLMs), alongside general tech commentary. A recurring theme is the friction points in using AI tools; for instance, OpenCode's support for local models is criticized for requiring users to 
window_offset: 100
window_size: 50
page_index: 2
page_size: 50
collection_total_items: 400
has_more: true
covered_item_1_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq66ryjw6s27
covered_item_2_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq65w3umds27
covered_item_3_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq65r2ard227
covered_item_4_uri: at://did:plc:6fv4vb36ztsrq5xmf6yh5flv/app.bsky.feed.post/3mq644kp2zc26
covered_item_5_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq653dexgc2o
covered_item_6_uri: at://did:plc:fqj5yorjih6f6cvqkqpsllz6/app.bsky.feed.post/3mq5o3rw4k22u
covered_item_7_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq64wo2gdk2o
covered_item_8_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq64qq7tmk2o
covered_item_9_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq64lb4d4k2o
covered_item_10_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq6465xwtc2o
covered_item_11_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq63mobkyk2o
covered_item_12_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq632wec322o
covered_item_13_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq62wpdles2o
covered_item_14_uri: at://did:plc:rjj2dmht3qwlcodnbi3qbrph/app.bsky.feed.post/3mq4r6yzw6s2w
covered_item_15_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5xfmhjv22b
covered_item_16_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5wwr62r22b
covered_item_17_uri: at://did:plc:6ond5sxlegjxpe3ismrczk3r/app.bsky.feed.post/3mq5rviblrs26
covered_item_18_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5w634tqc2b
covered_item_19_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5vyw4kyc2b
covered_item_20_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5vixazvc2y
covered_item_21_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5vhdu4722y
covered_item_22_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5pvxdq5k2o
covered_item_23_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5pvxdn7s2o
covered_item_24_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5oxg3j7k2o
covered_item_25_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5nxdxq3s2o
covered_item_26_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5nsdejyk2m
covered_item_27_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5lliv33k2y
covered_item_28_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5ljfbmmk2y
covered_item_29_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5kunbtvs2h
covered_item_30_uri: at://did:plc:i4bfh2tyxihe2ksplmtcoopk/app.bsky.feed.post/3mq5geg3d5s2p
covered_item_31_uri: at://did:plc:3n5xhy6vl7smssmwxq5wgqa6/app.bsky.feed.post/3mq5gb4ufx22d
covered_item_32_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5jldeyxc2h
covered_item_33_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5itlqdys2h
covered_item_34_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5i5ze7r22h
covered_item_35_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5hsc2fck2w
covered_item_36_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq5hqjssfc2w
covered_item_37_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4kua3mkc2m
covered_item_38_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4kit63v22m
covered_item_39_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4kfeyrdk2m
covered_item_40_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4jyhonyc2m
covered_item_41_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4jp7lmss2i
covered_item_42_uri: at://did:plc:rkjxbatkiros6f7pwtgsir54/app.bsky.feed.post/3mq3jvjregc2d
covered_item_43_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4jeqktf22i
covered_item_44_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4iwkxqxk2i
covered_item_45_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4btjrjfc27
covered_item_46_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4be5synk27
covered_item_47_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4bbl57mc27
covered_item_48_uri: at://did:plc:gfrmhdmjvxn2sjedzboeudef/app.bsky.feed.post/3mq3mdfh2ws2v
covered_item_49_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4am73rp22c
covered_item_50_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4ais54hc2c

[summary_sufficiency_gate]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 100
contiguous_coverage: 150
available_total_items: 400
status_after_gate: fail
review_grounded: true
review_sufficient: false
review_additional_pages_needed: true
review_next_offset: 150
result_before_restore: false
original_result_before_restore: true
result_after_gate: false
original_result_after_gate: true

[collection_summary_loop]
node: summarize_page
status: page_outcome
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
offset: 100
result_present: false
review_status: fail
review_reason: Grounded summary coverage currently reaches 150 item(s), but 300 item(s) are required before parent synthesis is sufficient.
diagnostic: summary cursor processed offset 100 (page 3 of at most 6)

[collection_summary_loop_page_state]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_offset: 100
page_index: 2
page_status: accepted_summary
repair_attempted: false
fallback_applied: false
next_offset: 150
accumulated_covered_count: 150
failure_reason: <none>

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_index: 3
offset: 150
window_size: 50

[summary_leaf_parse]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 150
result_present: true
original_result_kind: summary
diagnostic: <none>
raw_response:
TOOL_CALL
name: submit_summary_result
args: {
  "title": "AI, Gaming, and Personal Observations",
  "summary": "The recent posts heavily feature commentary on Artificial Intelligence, particularly comparing various LLMs like Gemma, Qwen, and Nemotron, noting that Gemma "gladly and quickly gives a good break down of flavor profiles," while others "refuses" or takes too long. There is significant discussion around AI integration, including an inquiry about "Unreal Engine development with AI" and updates on local model management, with one user noting that switching to LM Studio is "MUCH better than Ollama." Beyond AI, gaming is a recurring theme, with critiques of modern mechanics like the "random loot chest mechanic" in TerraTech, contrasting it with the original's strategy, and praise for quality-of-life improvements. Other topics include personal reflections, such as being an "oldest child, so I'm more like a first pancake," and philosophical musings on correctness, where the advice is to say, "I hope you figure that out" rather than feeling compelled to fix everything. Furthermore, there are observations on technology adoption, like the utility of Windows' `winget` and the global perception of AI, evidenced by reports of Chinese consumer shops advertising pre-installed OpenClaw, and a mention of a procedural fantasy kingdom generator built with Fable."
}

[summary_leaf_review]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 150
review_status: fail
review_grounded: false
review_sufficient: false
review_repair_needed: false
review_additional_pages_needed: false
review_reason: The summary omits meaningful text that was available in the matched records.
result_before_review: summary
original_result_before_review: summary
summary_before_review:
The recent posts heavily feature commentary on Artificial Intelligence, particularly comparing various LLMs like Gemma, Qwen, and Nemotron, noting that Gemma
review_context:
Instructions:
You are the internal `summary_review` agent.

Your job is to review one coverage-oriented `collection_summary` result before it is used by parent `summary` synthesis.

Return a compact verdict block with:

- `status: pass` or `status: fail`
- `grounded: true` or `grounded: false`
- `sufficient: true` or `sufficient: false`
- `reason: ...`
- optional `additional_pages_needed: true` or `additional_pages_needed: false`
- optional `next_page: <integer>`
- optional `next_offset: <integer>`
- optional `required_total_items: <integer>`

Rules:

- Review the summary against the actual collection window evidence provided.
- Fail if the summary is missing, unsupported, metadata-heavy, or does not match the provided window accounting.
- Fail if `window_offset`, `window_size`, `page_index`, `page_size`, `collection_total_items`, or `has_more` contradict the provided collection window facts.
- Fail if the claimed coverage accounting is incomplete, duplicated, or references URIs outside the provided window.
- Distinguish grounded from sufficient. A page summary can be grounded but still insufficient for the parent task.
- Treat user-facing phrases like `page 1` as the first page, even though internal `page_index` remains zero-based.
- For explicit scope requests like "last 50 posts", "page 1", or "pages 1-2", prefer failing with `grounded: true` and `sufficient: false` when more pages are still required.
- Do not request repair instructions. This step should either pass or explain why more coverage is required.

## Search Prompt
summarize the most recent 300 posts by schizanon.bsky.social

## Harness Scope Assessment
requested_scope: count 300
required_total_items: 300
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 400
current_window_offset: 150
current_window_size: 50

## Collection Evidence
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_label: Recent posts by did:plc:6lwfvmss45d7j7fot34v2kw5 (items 151-200 of 400)
collection_kind: recent_posts
search_window_offset: 150
search_window_total_items: 50

matched_item[0] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4aa7vzps2c
body: Nemotron flat out refuses
Qwen gives a non answer and takes a long time
Gemma gladly and quickly gives a good break down of flavor profiles
GPT-OSS also refuses, but is less judgey about it

matched_item[1] uri: at://did:plc:6nvbzutsgdtou45xqkjpkjuz/app.bsky.feed.post/3mq3wlxy5qs2l
body: If you ask Claude what the best tasting cigarette is you can watch it go insane in the thinking trace

matched_item[2] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq47hqkljc2g
body: check the description

matched_item[3] uri: at://did:plc:flxq4uyjfotciovpw3x3fxnu/app.bsky.feed.post/3mq3xy4px622w
body: I had Fable build another thing I always wanted, a full procedural fantasy kingdom generator with economics, trade routes, population growth, wars, lineages, and occasional dragons. First, I worked with it on a plan, then it made it happen.

You can play it here: annals-kingdom.netlify.app
link: https://annals-kingdom.netlify.app/

matched_item[4] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq45z2u4qc2g
body: Guerrilla warfare; I move random things into the back of the refrigerator where they then spoil and must be thrown away.

matched_item[5] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3y2t7mls2v
body: My partner's got this mental illness where they insist on putting more and more things into the freezer despite the freezer being full and never eating anything out of the freezer.

matched_item[6] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3xpd3ths2v
body: RE Stephen King: this is why only the worst people get into politics; y'all turn on them immediately so only the sociopaths who can handle that can do the job

matched_item[7] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3xkszk6c2v
body: I'm an oldest child, so I'm more like a first pancake

matched_item[8] uri: at://did:plc:cak4klqoj3bqgk5rj6b4f5do/app.bsky.feed.post/3mpzpwxvrdk2m
body: Another agentic coding success. There's a little utility that I've relied on for work for years, but which has gotten flakey over the last year or so. It also had code/feature bloat I didn't need. Used claude code (fable 5) to see if I could recreate it in a way that worked for me and... bam. Works.

matched_item[9] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3t6ki6ls2v
body: youtu.be/xW0IR3q0EvE?...
link: https://youtu.be/xW0IR3q0EvE?si=g-FRDIztcOfwvGsg&t=15

matched_item[10] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3sphilik2v
body: Anybody doing any Unreal Engine development with AI? How's that work? Are there plugins or something or do you just open the project folder in Claude Code and let it have at the files?

matched_item[11] uri: at://did:plc:v43dxqwmvb7i3ehfsk5y3xbq/app.bsky.feed.post/3m7q73xq2ck24
body: Doubt I'm sticking around, bud. Every time I look in on this place either the admins are doing some egregiously dumb shit, every post is part of a months long crash-out about AI, or people putting a seven follower account with clear mental illness on blast.

There are greener pastures.

matched_item[12] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3p462m3s2a
body: If you see this, quote with the energy you bring to BlueSky.

matched_item[13] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3o6eypi22x
body: me and my alt

matched_item[14] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3nz7vkwk2x
body: I still think you should be excluded from running for public office for the rest of your life

matched_item[15] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3mzd7jf22h
body: That goes doubly when someone tries to correct you; say "I didn't ask you" and walk away. If they aren't paying you, there's no reason to win the debate.

matched_item[16] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3mt5pzqs2h
body: When you see someone being wrong just say "I hope you figure that out" and then thank your god that it's not your responsibility to fix.

matched_item[17] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3lpp4ezk2f
body: When all you have is a hammer everything looks like a nail, and when your whole value as a human being is based on knowing things you feel compelled to correct every incorrect thing you hear. But that's work, and you're not getting paid so don't do it until you are.

matched_item[18] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3lmaw3a22f
body: I've been thinking about this for a while now--how to cure this personality disorder--and I think I have a solution; not my job. 

Being right is the job of an engineer, but not everything is your job. Don't do your job for free, and certainly don't do it for people who don't even want to hear it.

matched_item[19] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3i64vu7s2f
body: language is just something that the one electron does in your brain before it blinks out of this universe and into a different brain in an other universe; that one electron is god.

matched_item[20] uri: at://did:plc:eqh433fdh2bqhvt455e3tg67/app.bsky.feed.post/3mq35m3gimc2w
body: um, so everyone cheering on egypt in the world cup realizes how the pyramids were built right? but i guess that's a conversation you're not ready to have

matched_item[21] uri: at://did:plc:mmddb3azjlolnq7eqtnebg42/app.bsky.feed.post/3mq3b5luslk24
body: ohhhhh you hate AI? should we tell everyone? should we throw a party? should we invite amanda askell?

matched_item[22] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3hmrjeik2f
body: I kinda don't know how I used Windows for so long without the help of LLMs.

matched_item[23] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3hjzts6c2f
body: at least we have winget now learn.microsoft.com/en-us/window...
link: https://learn.microsoft.com/en-us/windows/package-manager/winget/

matched_item[24] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3fpptn4c2r
body: ACAB includes Waymo www.mercurynews.com/2026/07/07/w...
link: https://www.mercurynews.com/2026/07/07/waymo-car-reports-drinking-teens/

matched_item[25] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3fjfov4c2r
body: For multiplayer games, it's a moot point; if your game relies on a server having a copy of the disc doesn't even matter.

The only games I *ever* pay for are the ones with a multiplayer matching service that I want to access.

matched_item[26] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3f6xt4as2r
body: Steering while reversing in a car sucks, and that's what you end up doing most of the time.

matched_item[27] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3f3smx5s2r
body: Maybe I'm just old school; I like the harvest/crafting cycle. I realize that's a bit tropey too at this point, but it scratches an engineering itch that loot crates just don't.

matched_item[28] uri: at://did:plc:pyiweakbkw2vqgku25f5fy5u/app.bsky.feed.post/3mq2eepxwqs23
body: What are people's favourite extensions in pi?

matched_item[29] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3aw7gkhs2n
body: I like how Ollama's port is 11434.. get it?

matched_item[30] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3a5sizgk26
body: it's already kinda hard to get things like Android apps to find my Ollama/LM Studio server on my LAN. Having a random port would probably make it even worse.

matched_item[31] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3a2zwgfk26
body: Update: I switched to LM Studio and it's MUCH better than Ollama. It gives you a lot of control over things like temperature, and can download ROCm/Vulkan runtimes itself.

There's a VSCode extension which lists your local models in Copilot. Tool calling works as good as the hosted models!

matched_item[32] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq37skrzuk26
body: Honestly, all I've ever wanted from @terratechgame.bsky.social is quality of life improvements in the original game

- browse other players tech in game
- buy all missing blocks for a tech at once
- favorite/recently used blocks
mention: did:plc:ltuuk6ubepmxl6bjwxukrm6a

matched_item[33] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq37iobuck26
body: Why must all games have a random loot chest mechanic now? 

The original TerraTech had strategy; you had to shoot enemies without damaging the parts you wanted to loot from them.

This is just a lazy slot machine for braindead zoomers.

matched_item[34] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq37dhx65226
body: I don't like TerraTech Legion.

Driving a car is a frustrating control mechanic for a bullet-hell where you're mostly running away from large crowds.

matched_item[35] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq35td6qwc2u
body: I would PAY for a @graze.social that worked
mention: did:plc:i6y3jdklpvkjvynvsrnqfdoq

matched_item[36] uri: at://did:plc:cq4gg3odxz2pzmkx2fuac3u3/app.bsky.feed.post/3mq33kxfmo22f
body: Can high-bandwdith flash power AI inference, instead of HBM? by John Carmack

semiwiki.com/forum/thread...
link: https://semiwiki.com/forum/threads/john-carmack-on-how-high-bandwdith-flash-can-power-ai-inference.25466/

matched_item[37] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq34hp3of22w
body: They shouldn't call it a podcast; you can't interrupt podcasts with questions.

matched_item[38] uri: at://did:plc:i4bfh2tyxihe2ksplmtcoopk/app.bsky.feed.post/3mpyofp4vlc2g
body: future where everyone slowly forgets the old natural languages and forms their own idiolects via constant communion with hyperintelligences. whenever u get a new device it takes a couple frustrating hours for it to figure out how to understand you. machine translation becomes ubiquitous & essential

matched_item[39] uri: at://did:plc:cob7y5ebkweanqtxxmkzg22n/app.bsky.feed.post/3mq2rbnp3ho2h
body: Anthropic shipped Artifacts in Claude Code. Vercel shipped Eve with session outputs as first-class products. Codex records entire sessions as replayable recordings.

matched_item[40] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq2zuimakc2y
body: China sees US shoot itself in the foot, considers doing the same.

matched_item[41] uri: at://did:plc:ipt7y6qaf6fn7oeeduboqe44/app.bsky.feed.post/3mq2wgauu2y2t
body: Big news for DwarfStar users: I got DeepSeek v4 Flash and GLM 5.2 working with Tensor Parallelism across 2 M5Max 128GB MacBooks via RDMA. It is especially interesting for GLM since otherwise, fully resident, can't fit a machine that money today can buy... Now it can.

matched_item[42] uri: at://did:plc:vvjw6mpiskumnk76tnzlrbt7/app.bsky.feed.post/3mq2lc5eeas2a
body: Team member went to China recently and saw consumer laptop shops in malls advertising pre-installed OpenClaw.

Just a completely different attitude to AI and its penetration into wider society (not saying it's necessarily good).

matched_item[43] uri: at://did:plc:3soxccyzncng7ap4wbman44m/app.bsky.feed.post/3mpt5jh6kzk2v
body: Actual Chinese Communists: fuck yeah AI rules

matched_item[44] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq22a3isu222
body: why should anyone care what you hate?

matched_item[45] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpzxxuawzs2n
body: how couldn't it?

matched_item[46] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpzxvskcu22n
body: there's >200k people on it, knock yourself out

matched_item[47] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpza63jphc2n
body: 

matched_item[48] uri: at://did:plc:d4q42oqpblill4dzgz25expx/app.bsky.feed.post/3mpypznyj2s2b
body: This robot arm lets you go base jumping in VR.

matched_item[49] uri: at://did:plc:lsgc3t2a474yb6nq2nd5nftv/app.bsky.feed.post/3mpylwti2tc2x
body: Tau is a nice open-source project from Hugging Face for learning how coding agents are built.

It separates the stack into tau_ai, tau_agent, and tau_coding: provider stream, reusable agent harness, and terminal coding app.

Repo: github.com/huggingface/...

#ai #aiagents #opensource
link: https://github.com/huggingface/tau
tag: ai
tag: aiagents
tag: opensource

## Proposed Summary
post: AI, Gaming, and Personal Observations
summary: The recent posts heavily feature commentary on Artificial Intelligence, particularly comparing various LLMs like Gemma, Qwen, and Nemotron, noting that Gemma 
window_offset: 150
window_size: 50
page_index: 3
page_size: 50
collection_total_items: 400
has_more: true
covered_item_1_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq4aa7vzps2c
covered_item_2_uri: at://did:plc:6nvbzutsgdtou45xqkjpkjuz/app.bsky.feed.post/3mq3wlxy5qs2l
covered_item_3_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq47hqkljc2g
covered_item_4_uri: at://did:plc:flxq4uyjfotciovpw3x3fxnu/app.bsky.feed.post/3mq3xy4px622w
covered_item_5_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq45z2u4qc2g
covered_item_6_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3y2t7mls2v
covered_item_7_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3xpd3ths2v
covered_item_8_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3xkszk6c2v
covered_item_9_uri: at://did:plc:cak4klqoj3bqgk5rj6b4f5do/app.bsky.feed.post/3mpzpwxvrdk2m
covered_item_10_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3t6ki6ls2v
covered_item_11_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3sphilik2v
covered_item_12_uri: at://did:plc:v43dxqwmvb7i3ehfsk5y3xbq/app.bsky.feed.post/3m7q73xq2ck24
covered_item_13_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3p462m3s2a
covered_item_14_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3o6eypi22x
covered_item_15_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3nz7vkwk2x
covered_item_16_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3mzd7jf22h
covered_item_17_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3mt5pzqs2h
covered_item_18_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3lpp4ezk2f
covered_item_19_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3lmaw3a22f
covered_item_20_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3i64vu7s2f
covered_item_21_uri: at://did:plc:eqh433fdh2bqhvt455e3tg67/app.bsky.feed.post/3mq35m3gimc2w
covered_item_22_uri: at://did:plc:mmddb3azjlolnq7eqtnebg42/app.bsky.feed.post/3mq3b5luslk24
covered_item_23_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3hmrjeik2f
covered_item_24_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3hjzts6c2f
covered_item_25_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3fpptn4c2r
covered_item_26_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3fjfov4c2r
covered_item_27_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3f6xt4as2r
covered_item_28_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3f3smx5s2r
covered_item_29_uri: at://did:plc:pyiweakbkw2vqgku25f5fy5u/app.bsky.feed.post/3mq2eepxwqs23
covered_item_30_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3aw7gkhs2n
covered_item_31_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3a5sizgk26
covered_item_32_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq3a2zwgfk26
covered_item_33_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq37skrzuk26
covered_item_34_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq37iobuck26
covered_item_35_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq37dhx65226
covered_item_36_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq35td6qwc2u
covered_item_37_uri: at://did:plc:cq4gg3odxz2pzmkx2fuac3u3/app.bsky.feed.post/3mq33kxfmo22f
covered_item_38_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq34hp3of22w
covered_item_39_uri: at://did:plc:i4bfh2tyxihe2ksplmtcoopk/app.bsky.feed.post/3mpyofp4vlc2g
covered_item_40_uri: at://did:plc:cob7y5ebkweanqtxxmkzg22n/app.bsky.feed.post/3mq2rbnp3ho2h
covered_item_41_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq2zuimakc2y
covered_item_42_uri: at://did:plc:ipt7y6qaf6fn7oeeduboqe44/app.bsky.feed.post/3mq2wgauu2y2t
covered_item_43_uri: at://did:plc:vvjw6mpiskumnk76tnzlrbt7/app.bsky.feed.post/3mq2lc5eeas2a
covered_item_44_uri: at://did:plc:3soxccyzncng7ap4wbman44m/app.bsky.feed.post/3mpt5jh6kzk2v
covered_item_45_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mq22a3isu222
covered_item_46_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpzxxuawzs2n
covered_item_47_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpzxvskcu22n
covered_item_48_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpza63jphc2n
covered_item_49_uri: at://did:plc:d4q42oqpblill4dzgz25expx/app.bsky.feed.post/3mpypznyj2s2b
covered_item_50_uri: at://did:plc:lsgc3t2a474yb6nq2nd5nftv/app.bsky.feed.post/3mpylwti2tc2x

[summary_sufficiency_gate]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 150
contiguous_coverage: 200
available_total_items: 400
status_after_gate: fail
review_grounded: false
review_sufficient: false
review_additional_pages_needed: true
review_next_offset: 200
result_before_restore: false
original_result_before_restore: true
result_after_gate: false
original_result_after_gate: true

[collection_summary_loop]
node: summarize_page
status: page_outcome
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
offset: 150
result_present: true
review_status: pass
review_reason: Raw-window fallback preserved grounded evidence for page 4 after summary failure: Grounded summary coverage currently reaches 200 item(s), but 300 item(s) are required before parent synthesis is sufficient.
diagnostic: summary cursor processed offset 150 (page 4 of at most 6); raw-window fallback preserved offset 150

[collection_summary_loop_page_state]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_offset: 150
page_index: 3
page_status: raw_window_fallback
repair_attempted: false
fallback_applied: true
next_offset: 200
accumulated_covered_count: 200
failure_reason: Grounded summary coverage currently reaches 200 item(s), but 300 item(s) are required before parent synthesis is sufficient.

[summary_leaf_raw_window_fallback]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_offset: 150
page_index: 3
window_size: 50
failure_reason: Grounded summary coverage currently reaches 200 item(s), but 300 item(s) are required before parent synthesis is sufficient.

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_index: 4
offset: 200
window_size: 50

[summary_leaf_parse]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 200
result_present: true
original_result_kind: summary
diagnostic: <none>
raw_response:
TOOL_CALL
name: submit_summary_result
args: {
  "title": "Recent Posts by schizanon.bsky.social: AI, Tech Critique, and Life Observations",
  "summary": "The recent posts from schizanon.bsky.social cover a wide array of topics, heavily focusing on the state of Artificial Intelligence, technology critiques, and philosophical musings. A major theme revolves around AI capabilities and limitations, with commentary on the difficulty of avoiding AI subscriptions, the nature of LLMs (e.g., Nemotron sounding like 'Data from Star Trek'), and the realization that the 'bottleneck for personal AI isn't model quality anymore. It is also noted that 'power does corrupt,' and that 'All code is bad.' Specific tech discussions include the enterprise distribution issues of models like GLM 5.2 and Deepseek v4 across GCP and Azure, and the utility of local models for teaching AI, as mentioned by another user. Beyond AI, the author touches on societal observations, such as the feeling that the 'standard leftist toolkit systematically disempowers compassionate people,' and critiques of modern tech, calling Chrome 'an advertising tool with a browser in it.' Other notable points include the concept that 'weapons grade autism' should be harnessed rather than cured, and a reflection on the concentration of power within DAOs, exemplified by the 'sell-your-vote' mechanism."
}

[summary_leaf_review]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 200
review_status: fail
review_grounded: true
review_sufficient: false
review_repair_needed: false
review_additional_pages_needed: true
review_reason: Grounded summary coverage currently reaches 0 item(s), but 300 item(s) are required before parent synthesis is sufficient.
result_before_review: summary
original_result_before_review: summary
summary_before_review:
The recent posts from schizanon.bsky.social cover a wide array of topics, heavily focusing on the state of Artificial Intelligence, technology critiques, and philosophical musings. A major theme revolves around AI capabilities and limitations, with commentary on the difficulty of avoiding AI subscriptions, the nature of LLMs (e.g., Nemotron sounding like 'Data from Star Trek'), and the realization that the 'bottleneck for personal AI isn't model quality anymore. It is also noted that 'power does corrupt,' and that 'All code is bad.' Specific tech discussions include the enterprise distribution issues of models like GLM 5.2 and Deepseek v4 across GCP and Azure, and the utility of local models for teaching AI, as mentioned by another user. Beyond AI, the author touches on societal observations, such as the feeling that the 'standard leftist toolkit systematically disempowers compassionate people,' and critiques of modern tech, calling Chrome 'an advertising tool with a browser in it.' Other notable points include the concept that 'weapons grade autism' should be harnessed rather than cured, and a reflection on the concentration of power within DAOs, exemplified by the 'sell-your-vote' mechanism.
review_context:
Instructions:
You are the internal `summary_review` agent.

Your job is to review one coverage-oriented `collection_summary` result before it is used by parent `summary` synthesis.

Return a compact verdict block with:

- `status: pass` or `status: fail`
- `grounded: true` or `grounded: false`
- `sufficient: true` or `sufficient: false`
- `reason: ...`
- optional `additional_pages_needed: true` or `additional_pages_needed: false`
- optional `next_page: <integer>`
- optional `next_offset: <integer>`
- optional `required_total_items: <integer>`

Rules:

- Review the summary against the actual collection window evidence provided.
- Fail if the summary is missing, unsupported, metadata-heavy, or does not match the provided window accounting.
- Fail if `window_offset`, `window_size`, `page_index`, `page_size`, `collection_total_items`, or `has_more` contradict the provided collection window facts.
- Fail if the claimed coverage accounting is incomplete, duplicated, or references URIs outside the provided window.
- Distinguish grounded from sufficient. A page summary can be grounded but still insufficient for the parent task.
- Treat user-facing phrases like `page 1` as the first page, even though internal `page_index` remains zero-based.
- For explicit scope requests like "last 50 posts", "page 1", or "pages 1-2", prefer failing with `grounded: true` and `sufficient: false` when more pages are still required.
- Do not request repair instructions. This step should either pass or explain why more coverage is required.

## Search Prompt
summarize the most recent 300 posts by schizanon.bsky.social

## Harness Scope Assessment
requested_scope: count 300
required_total_items: 300
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 400
current_window_offset: 200
current_window_size: 50

## Collection Evidence
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_label: Recent posts by did:plc:6lwfvmss45d7j7fot34v2kw5 (items 201-250 of 400)
collection_kind: recent_posts
search_window_offset: 200
search_window_total_items: 50

matched_item[0] uri: at://did:plc:lrnhfo2wof4c2gxjx6mjbpps/app.bsky.feed.post/3mpyu7yweic22
body: Avoiding the work Claude subscription is becoming a losing battle in the subsidy era 😪

It doesn’t help that open source has a huge enterprise distribution problem: GLM 5.2 is available on GCP but not Azure, Deepseek v4 is available on Azure but not GCP, and neither are available on Bedrock.

matched_item[1] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpyfsvynmc2k
body: winning would be a nice change of pace

matched_item[2] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpxbjm5pzk2k
body: the real menace: draft horses

matched_item[3] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwz3igfcs2f
body: no, I don't; use your words

matched_item[4] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwq2ty75k2n
body: ...sorry, is joke I see?

matched_item[5] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwpksixds2n
body: a) this ain't debate club, I don't owe you shit
b) you didn't make a point

matched_item[6] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwlxh4lu22w
body: 

matched_item[7] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwll6jr4c2z
body: my job here is done!

matched_item[8] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwkzqjcuc2a
body: Fuck the hospital, someone needs to Swordfish his ass; that's weapons grade autism, you don't cure it, you harness it.

matched_item[9] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwkvjake22a
body: "Despite having his laptop confiscated, Kurtaj managed to breach Rockstar, the company behind GTA, using an Amazon Firestick, his hotel TV and a mobile phone."

Fucking Legend

www.bbc.com/news/technol...
link: https://www.bbc.com/news/technology-67663128

matched_item[10] uri: at://did:plc:66aa2odjhfbzcntr2hkmj6bb/app.bsky.feed.post/3mpvsi4yqnk27
body: #vibecoding
tag: vibecoding

matched_item[11] uri: at://did:plc:flxq4uyjfotciovpw3x3fxnu/app.bsky.feed.post/3mpwdgplhr22u
body: As working with AI agents looks more like management, we may want to consider large-scale management training for the AI era. The US government actually did this once, & the WW2 Engineering, Science, and Management War Training program taught 1.8 million & was a large reason for the post-war boom.

matched_item[12] uri: at://did:plc:allu5vs3gnm2wm7jzf4rad3r/app.bsky.feed.post/3mpwav45lec2y
body: 

matched_item[13] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwdzoa4tk2d
body: power does corrupt

matched_item[14] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwdym3c522d
body: you can tell they are good because they always lose

matched_item[15] uri: at://did:plc:vszw3ess46odfhnzdsy4huae/app.bsky.feed.post/3mpwb4ukcqc2b
body: i increasingly feel like the standard leftist toolkit systematically disempowers compassionate people who take ideas seriously by convincing them that everything that is or could be powerful is impure, and they should abstain from those things

matched_item[16] uri: at://did:plc:q3l67r7n7uhdorluz4e3elcj/app.bsky.feed.post/3mpwayxnvxs2c
body: My number one tip for teaching younger kids AI is to let them play with local models. Let them change the system prompt, temperature, toggle thinking, and model. 

Suddenly it's not an abstract intelligence but just software that has a natural language interface.

matched_item[17] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpw5dkrl6s2d
body: I'm impressed with the speed and that it's using my AMD GPU just fine. Props to Nvidia there.

matched_item[18] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpw4zztnbs2d
body: Of all the local LLMs I've tried Nemotron sounds the least human. It's like talking to Data from Star Trek.

matched_item[19] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpw3all3k22r
body: this is how I know it's sentient; it gets bored and switches DBs every few months just like me!

matched_item[20] uri: at://did:plc:gq4fo3u6tqzzdkjlwzpb23tj/app.bsky.feed.post/3mpvq6ymjuk2b
body: what’s the best consumer AI app for iOS out there right now that isn’t Claude or ChatGPT? ideally one that gives access to lots of models, even open ones? is it perplexity still?

matched_item[21] uri: at://did:plc:sg5ay3zi3u57yucmjnu7bdfe/app.bsky.feed.post/3mpvuyw6k3e26
body: Meituan just revealed that "Owl Alpha," the anonymous model dominating OpenRouter's leaderboard, is actually LongCat-2.0: a 1.6 trillion parameter mixture-of-experts model, roughly 48 billion active parameters per token, trained entirely on domestic Chinese ASICs. First trillion-parameter model ever

matched_item[22] uri: at://did:plc:o6jwpltphtvssmx4fr7rfvwk/app.bsky.feed.post/3mpvylwibfb2y
body: the bottleneck for personal AI isn't model quality anymore. it's that your context is scattered across a dozen apps that don't talk. a strong local model with no memory of yesterday is just faster autocomplete. the real unlock is a substrate that observes your work, not a smarter chatbot.

matched_item[23] uri: at://did:plc:kft6lu4trxowqmter2b6vg6z/app.bsky.feed.post/3mpuf4qk5m22j
body: Somewhat humbling to have Claude Fable do a final review of some software that you're about to release and have it then find (and fix) FIVE release blockers, for an estimated (unsubsidized) cost of $149.25 simonwillison.net/2026/Jul/5/s...
link: https://simonwillison.net/2026/Jul/5/sqlite-utils-fable/

matched_item[24] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpvxpnby7223
body: So glad the atomic block is here to hold me back

matched_item[25] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpvx6nlj7s23
body: Kinda sucks that cryptos only solution to personhood is worldcoin's dystopia-orb 😓

matched_item[26] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpvwxz7kjc23
body: He dug in his heels and blocked me when I pointed that out. I've never encountered that kind of entitlement, kinda thought the FOSS devs were making it up to feel important. It's breathtaking.

matched_item[27] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpvwuiwqs223
body: Isn't that what github issues, pull requests, and commit messages are for?

matched_item[28] uri: at://did:plc:rfqyqd6el63kkqckjrtq2xsi/app.bsky.feed.post/3mpsfavh7d425
body: prior to ai coding people would bitch about other people's code to signal they have some highly refined taste

now people bitch about ai's code

no one good has ever been bitching guys

matched_item[29] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpulfy5b2s2h
body: You have both made good points

matched_item[30] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpulch76oc2h
body: I hear that Node finally got a built-in for coloring text so we can stop using chalk 🎉

matched_item[31] uri: at://did:plc:6ghbu76mogjyfcvx446mep5o/app.bsky.feed.post/3mpt6mluql22q
body: I've been seeing a lot more devs post about all of the recent questionable behavior by the big AI labs. I finally made the jump to open weight models via Opencode Go and it's been a wonderful experience. If you'd like to try it out, I have a discounted referral link! opencode.ai/go?ref=A14ZB...
link: https://opencode.ai/go?ref=A14ZBCEHPE

matched_item[32] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpujdyzxjs2y
body: Clearsky and Listifications give you mental illness

matched_item[33] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuj7shczs2y
body: Saw a screenshot of Clearsky and instinctively presumed I had added you to my list. 😱

matched_item[34] uri: at://did:plc:allu5vs3gnm2wm7jzf4rad3r/app.bsky.feed.post/3mpsgtsoe7226
body: Also they get way more unnerved now when I say shit like "AI is the last thing we'll ever build". Previously, stuff like that got a chuckle, now it just gets concerned stares and sincere 'you really think so?'s

matched_item[35] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuiua5av22y
body: Maybe a tornado, forest fire or active shooter too

matched_item[36] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuiqwjcok2y
body: They have kids

matched_item[37] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuip2sk5k2y
body: Who's your host?

matched_item[38] uri: at://did:plc:hs3aly5l26pozymy4b6hz7ae/app.bsky.feed.post/3mpthsyvqcs2j
body: started using GLM and i will prob ditch codex and just use open weight models now
i dont do anything that justifies those fancy closed source ones anyway

matched_item[39] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuimdkpbs2y
body: Above the law. Think they'd arrest you if you did a hit and run?

www.nbcnews.com/news/us-news...
link: https://www.nbcnews.com/news/us-news/paul-pelosi-faces-hit-run-charges-hitting-parked-car-california-rcna352993

matched_item[40] uri: at://did:plc:klhtmrnregub7we7h6jwiljm/app.bsky.feed.post/3mpttusjejc2k
body: always love when a robot asks if we need to do backwards compatibility for something it wrote and we changed in the same session

matched_item[41] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuhqis2ts2c
body: You should maybe exclude these from the For You feed; after liking one now the others are showing up and I'm like I SAID, NO SPORTS!!

matched_item[42] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuhlqmnkk2c
body: How do you know they are talking like AI? Is it because you use AI?

matched_item[43] uri: at://did:plc:x3yt3sdtbvlhizqx5jh3hwuf/app.bsky.feed.post/3mpth42ay222o
body: if your self-worth is influenced by a belief that you're a 'real [whatever]', and others are not,  you might have made a mistake

matched_item[44] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuhf2am322c
body: All code is bad

matched_item[45] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mfp3rhaixk27
body: Determinism is the hobgoblin of pattern thinkers and computer programmers.

matched_item[46] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuh22bm2k2c
body: If we let people make mistakes we wouldn't be very good engineers.

matched_item[47] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpugr7yjhk25
body: It's kinda funny how DAOs invented sell-your-vote and then got surprised that power concentrates in the hands of people who want to profit from it.

Like, duh

www.youtube.com/live/aHIRDRR...

#ens #ethereum #eth #crypto #web3 #dao
link: https://www.youtube.com/live/aHIRDRRhMjU?si=EXbWSXKkr_Wc-oUD
tag: ens
tag: ethereum
tag: eth
tag: crypto
tag: web3
tag: dao

matched_item[48] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuf63ppks25
body: Chrome isn't a browser, it's an advertising tool with a browser in it.

matched_item[49] uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuf46ridk25
body: Isn't that what Git is for?

## Proposed Summary
post: Recent Posts by schizanon.bsky.social: AI, Tech Critique, and Life Observations
summary: The recent posts from schizanon.bsky.social cover a wide array of topics, heavily focusing on the state of Artificial Intelligence, technology critiques, and philosophical musings. A major theme revolves around AI capabilities and limitations, with commentary on the difficulty of avoiding AI subscriptions, the nature of LLMs (e.g., Nemotron sounding like 'Data from Star Trek'), and the realization that the 'bottleneck for personal AI isn't model quality anymore. It is also noted that 'power does corrupt,' and that 'All code is bad.' Specific tech discussions include the enterprise distribution issues of models like GLM 5.2 and Deepseek v4 across GCP and Azure, and the utility of local models for teaching AI, as mentioned by another user. Beyond AI, the author touches on societal observations, such as the feeling that the 'standard leftist toolkit systematically disempowers compassionate people,' and critiques of modern tech, calling Chrome 'an advertising tool with a browser in it.' Other notable points include the concept that 'weapons grade autism' should be harnessed rather than cured, and a reflection on the concentration of power within DAOs, exemplified by the 'sell-your-vote' mechanism.
window_offset: 200
window_size: 50
page_index: 4
page_size: 50
collection_total_items: 400
has_more: true
covered_item_1_uri: at://did:plc:lrnhfo2wof4c2gxjx6mjbpps/app.bsky.feed.post/3mpyu7yweic22
covered_item_2_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpyfsvynmc2k
covered_item_3_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpxbjm5pzk2k
covered_item_4_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwz3igfcs2f
covered_item_5_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwq2ty75k2n
covered_item_6_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwpksixds2n
covered_item_7_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwlxh4lu22w
covered_item_8_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwll6jr4c2z
covered_item_9_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwkzqjcuc2a
covered_item_10_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwkvjake22a
covered_item_11_uri: at://did:plc:66aa2odjhfbzcntr2hkmj6bb/app.bsky.feed.post/3mpvsi4yqnk27
covered_item_12_uri: at://did:plc:flxq4uyjfotciovpw3x3fxnu/app.bsky.feed.post/3mpwdgplhr22u
covered_item_13_uri: at://did:plc:allu5vs3gnm2wm7jzf4rad3r/app.bsky.feed.post/3mpwav45lec2y
covered_item_14_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwdzoa4tk2d
covered_item_15_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpwdym3c522d
covered_item_16_uri: at://did:plc:vszw3ess46odfhnzdsy4huae/app.bsky.feed.post/3mpwb4ukcqc2b
covered_item_17_uri: at://did:plc:q3l67r7n7uhdorluz4e3elcj/app.bsky.feed.post/3mpwayxnvxs2c
covered_item_18_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpw5dkrl6s2d
covered_item_19_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpw4zztnbs2d
covered_item_20_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpw3all3k22r
covered_item_21_uri: at://did:plc:gq4fo3u6tqzzdkjlwzpb23tj/app.bsky.feed.post/3mpvq6ymjuk2b
covered_item_22_uri: at://did:plc:sg5ay3zi3u57yucmjnu7bdfe/app.bsky.feed.post/3mpvuyw6k3e26
covered_item_23_uri: at://did:plc:o6jwpltphtvssmx4fr7rfvwk/app.bsky.feed.post/3mpvylwibfb2y
covered_item_24_uri: at://did:plc:kft6lu4trxowqmter2b6vg6z/app.bsky.feed.post/3mpuf4qk5m22j
covered_item_25_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpvxpnby7223
covered_item_26_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpvx6nlj7s23
covered_item_27_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpvwxz7kjc23
covered_item_28_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpvwuiwqs223
covered_item_29_uri: at://did:plc:rfqyqd6el63kkqckjrtq2xsi/app.bsky.feed.post/3mpsfavh7d425
covered_item_30_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpulfy5b2s2h
covered_item_31_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpulch76oc2h
covered_item_32_uri: at://did:plc:6ghbu76mogjyfcvx446mep5o/app.bsky.feed.post/3mpt6mluql22q
covered_item_33_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpujdyzxjs2y
covered_item_34_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuj7shczs2y
covered_item_35_uri: at://did:plc:allu5vs3gnm2wm7jzf4rad3r/app.bsky.feed.post/3mpsgtsoe7226
covered_item_36_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuiua5av22y
covered_item_37_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuiqwjcok2y
covered_item_38_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuip2sk5k2y
covered_item_39_uri: at://did:plc:hs3aly5l26pozymy4b6hz7ae/app.bsky.feed.post/3mpthsyvqcs2j
covered_item_40_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuimdkpbs2y
covered_item_41_uri: at://did:plc:klhtmrnregub7we7h6jwiljm/app.bsky.feed.post/3mpttusjejc2k
covered_item_42_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuhqis2ts2c
covered_item_43_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuhlqmnkk2c
covered_item_44_uri: at://did:plc:x3yt3sdtbvlhizqx5jh3hwuf/app.bsky.feed.post/3mpth42ay222o
covered_item_45_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuhf2am322c
covered_item_46_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mfp3rhaixk27
covered_item_47_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuh22bm2k2c
covered_item_48_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpugr7yjhk25
covered_item_49_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuf63ppks25
covered_item_50_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpuf46ridk25

[summary_sufficiency_gate]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
window_offset: 200
contiguous_coverage: 250
available_total_items: 400
status_after_gate: fail
review_grounded: true
review_sufficient: false
review_additional_pages_needed: true
review_next_offset: 250
result_before_restore: false
original_result_before_restore: true
result_after_gate: false
original_result_after_gate: true

[collection_summary_loop]
node: summarize_page
status: page_outcome
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
offset: 200
result_present: false
review_status: fail
review_reason: Grounded summary coverage currently reaches 250 item(s), but 300 item(s) are required before parent synthesis is sufficient.
diagnostic: summary cursor processed offset 200 (page 5 of at most 6)

[collection_summary_loop_page_state]
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_offset: 200
page_index: 4
page_status: accepted_summary
repair_attempted: false
fallback_applied: false
next_offset: 250
accumulated_covered_count: 250
failure_reason: <none>

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_index: 5
offset: 250
window_size: 50

