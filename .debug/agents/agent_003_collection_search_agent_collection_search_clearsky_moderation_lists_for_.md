# Agent Debug

- agent_id: 3
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Clearsky moderation lists for did:plc:6lwfvmss45d7j7fot34v2kw5
- status: completed
- parent_agent_id: 1
- child_agent_ids: <none>
- collection_id: clearsky_lists:did:plc:6lwfvmss45d7j7fot34v2kw5

## Result Summary

post: LLM-selected post in Clearsky moderation lists for did:plc:6lwfvmss45d7j7fot34v2kw5 (reduced retry view)
summary: Grounded evidence centers on: Generative ai users/defenders; Anyone who uses generative ai. Whether it's posting ai images, having an ai profile picture, an ai banner, shares ai generated songs, anything. Also anyone who defends the use of ai.; Fascist trash heap.
search_result_1_uri: https://bsky.app/profile/did:plc:23tc33rlb7lsuv3hoo5zqxqw/lists/3m2vj4z4nbq2m
search_result_1_source_collection_id: clearsky_lists:did:plc:6lwfvmss45d7j7fot34v2kw5
search_result_2_uri: https://bsky.app/profile/did:plc:26ah4gdh7exq73vcjth6ecfn/lists/3ltjpf3dhsz2j
search_result_2_source_collection_id: clearsky_lists:did:plc:6lwfvmss45d7j7fot34v2kw5
search_result_3_uri: https://bsky.app/profile/did:plc:477rnpqffrg4vayxgmu22v5u/lists/3mirvqcbojn2i
search_result_3_source_collection_id: clearsky_lists:did:plc:6lwfvmss45d7j7fot34v2kw5
search_result_4_uri: https://bsky.app/profile/did:plc:25fzh7yrbl5col4b7mqa3cvl/lists/3mik5rixqfd2i
search_result_4_source_collection_id: clearsky_lists:did:plc:6lwfvmss45d7j7fot34v2kw5

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 2163
- truncated: false

## Rendered Context Window

```text
Instructions:
Inspect the provided collection carefully.

Return a grounded result block with `title:`, `summary:`, and up to four `uri:` lines for the chosen search results.

Every `uri:` must be a real item from the collection.

The `summary:` field is required. It should be dense and grounded: include the main repeated themes, the strongest exact phrases or list names, and any meaningful split or ambiguity inside the collection.

Use the `uri:` lines to point at the strongest supporting records.

If fewer than four real search results are relevant, return fewer.

Your evidence is constrained to the collection:
- quote exact short snippets, list names, list descriptions, or other text taken from the collection
- note repeated themes across multiple items when relevant

For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context.

Do not invent a separate label field unless it appears explicitly in the collection text.

Do not add higher-level interpretation beyond grouping repeated evidence and short contrasts that are directly supported by the collection text.

Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.

## Collection
collection_id: clearsky_lists:did:plc:6lwfvmss45d7j7fot34v2kw5
label: Clearsky moderation lists for did:plc:6lwfvmss45d7j7fot34v2kw5 (reduced retry view)
collection_kind: clearsky_lists
item_count: 24
last_refreshed_at: 0
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
refresh_ttl_seconds: 900
metadata.collection_kind: clearsky_lists
metadata.actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5

item[0]
uri: https://bsky.app/profile/did:plc:23tc33rlb7lsuv3hoo5zqxqw/lists/3m2vj4z4nbq2m
author: clearsky
type: moderation_list
list_name: Generative ai users/defenders
list_description: Anyone who uses generative ai. Whether it's posting ai images, having an ai profile picture, an ai banner, shares ai generated songs, anything. Also anyone who defends the use of ai.

item[1]
uri: https://bsky.app/profile/did:plc:25fzh7yrbl5col4b7mqa3cvl/lists/3mik5rixqfd2i
author: clearsky
type: moderation_list
list_name: pieces de merde
list_description: people i do not know who have me blocked for no reason despite we share the same views.. sad that assholes exist in some spaces..

item[2]
uri: https://bsky.app/profile/did:plc:26ah4gdh7exq73vcjth6ecfn/lists/3ltjpf3dhsz2j
author: clearsky
type: moderation_list
list_name: Fascist trash heap
list_description: General anti-fascist blocklist. Nazis, Zionists, blue MAGA, anticommunists, transphobes, racists, AI shills, people who use "tankie" as an insult unironically, subscribers to anticommunist blocklists, etc etc. all the same shit different ass.

item[3]
uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfwqzzsx2n
author: clearsky
type: moderation_list
list_name: Follows of @godoglyness.bsky.social
list_description: Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts.

item[4]
uri: https://bsky.app/profile/did:plc:2bxfqrquzicluosmz24ctjgs/lists/3mlj2prxx3x26
author: clearsky
type: moderation_list
list_name: pro israel
list_description: fuck democrats

item[5]
uri: https://bsky.app/profile/did:plc:2ekf2iwtp3ttwkfelu43pjmv/lists/3lbhojn2cqu2i
author: clearsky
type: moderation_list
list_name: Blockity block
list_description: Block

item[6]
uri: https://bsky.app/profile/did:plc:2f2hj2uzsrkcjlr4a6crcf3v/lists/3mdghabjujm2b
author: clearsky
type: moderation_list
list_name: Aggressive …holes, just no 🙂‍↔️
list_description: 

item[7]
uri: https://bsky.app/profile/did:plc:2ksyekjzm6vw5izbacdzyovb/lists/3kwf6phnfd52p
author: clearsky
type: moderation_list
list_name: Into the Sun
list_description: generic block list

item[8]
uri: https://bsky.app/profile/did:plc:2lsmqs5q6kheypemqzwbkqp7/lists/3lutm2ekegk2l
author: clearsky
type: moderation_list
list_name: AI Slop
list_description: Users that frequently upload content "created" via generative AI.

item[9]
uri: https://bsky.app/profile/did:plc:2nkqpzxjmgr7zzjm6edvp4gk/lists/3lrnizklvmz2i
author: clearsky
type: moderation_list
list_name: Little beetches
list_description: Cowards who put people on lists and then block

item[10]
uri: https://bsky.app/profile/did:plc:2nkqpzxjmgr7zzjm6edvp4gk/lists/3lvu3unyedj2v
author: clearsky
type: moderation_list
list_name: The unwashed a$$ez of bluesky
list_description: 

item[11]
uri: https://bsky.app/profile/did:plc:2qm3rdn7xv3un7nfcvjhwv4u/lists/3matisxukmu2m
author: clearsky
type: moderation_list
list_name: AI Bros
list_description: I won’t add you to the list if you only like AI for the science, but stop dick riding AI

item[12]
uri: https://bsky.app/profile/did:plc:2tikdjtnugm4lpbazjghtlll/lists/3malktudc6f2r
author: clearsky
type: moderation_list
list_name: AI slop
list_description: List to mass block accounts spreading generative AI.

item[13]
uri: https://bsky.app/profile/did:plc:2xvsh5wxxrd54u53ipdygvlp/lists/3lb3bp4gaah2r
author: clearsky
type: moderation_list
list_name: Problematic accounts
list_description: Accounts to block (or mute if you're too kind) because they were negative, engagement farming, or just bringing the other site vibes. I'm also adding spam bots / abusive list makers too

item[14]
uri: https://bsky.app/profile/did:plc:35c6odv2t3m3kacfizakyni6/lists/3lc6b7dukv52t
author: clearsky
type: moderation_list
list_name: MAGA PUKES
list_description: Another list of MAGA deplorables for your blocking pleasure. I looked at each individual profile before adding them.

item[15]
uri: https://bsky.app/profile/did:plc:37eafgax67jm3v7apim5vy4s/lists/3kaby5buhor2t
author: clearsky
type: moderation_list
list_name: Menschenfeindlichkeit
list_description: Hier werden Accounts mit queer- und transfeindlichen, misogynen, rassistischen oder ableistischen Inhalten erfasst. Das schließt Follower*innen insbesondere großer Accounts, die mit solchen Aussagen oder Handlungen aufgefallen sind, explizit ein.

item[16]
uri: https://bsky.app/profile/did:plc:3dp67jb5li3aiut32fvxxija/lists/3lb5e6se2qc2m
author: clearsky
type: moderation_list
list_name: Plagiadores, suplantadores, timadores
list_description: Personas a las que yo o uno de mis contactos directos hemos pillado plagiando (es decir, copiando contenido ajeno sin atribución), suplantando a otros usuarios, usando IA generativa de forma recurrente o intentando timar.

item[17]
uri: https://bsky.app/profile/did:plc:3hzsbqgzgjq55yidqovjpeg5/lists/3lv6b4xoztx2s
author: clearsky
type: moderation_list
list_name: QP's Blocklist #1
list_description: Personal blocklist, I don't recommend others use them.

item[18]
uri: https://bsky.app/profile/did:plc:3mvwwv4q3aehb46yk7zgrzsh/lists/3l74tlw33742t
author: clearsky
type: moderation_list
list_name: The IndieGame Devs List
list_description: An ever-growing list of people in the indie gamedev industry, showcasing creativity and innovation in the gaming community on Bluesky.

item[19]
uri: https://bsky.app/profile/did:plc:3scildjwchglpd6boxm723fi/lists/3k6tcbziupa2k
author: clearsky
type: moderation_list
list_name: Anti-Artist
list_description: Theft; AI; people who defraud artists or art commissioners; scammer sympathizers; people who hate the arts and attack artists for sharing their work. Mostly combination of other lists - if you're wrongfully added, DM lectronyx.bsky.app with appeals.

item[20]
uri: https://bsky.app/profile/did:plc:3y6un7djfzvac2t7xscipbkl/lists/3moj6hxw7ay2v
author: clearsky
type: moderation_list
list_name: Zionist
list_description: 

item[21]
uri: https://bsky.app/profile/did:plc:44mceatx27xkborhr44q7blo/lists/3m5co5rvv2z2h
author: clearsky
type: moderation_list
list_name: blocklist chuds
list_description: people who randomly add me to blocklists or block me for no reason, be it because of following someone they don't like, having views they don't like, false allegations, etc. You're a fat lil chud

item[22]
uri: https://bsky.app/profile/did:plc:477rnpqffrg4vayxgmu22v5u/lists/3mirvqcbojn2i
author: clearsky
type: moderation_list
list_name: Zionists / Israel
list_description: Michael Bloomberg controls Bluesky Israel decides what you see on here due to massive amounts of ZIONISM

item[23]
uri: https://bsky.app/profile/did:plc:4e7mo6o5xf2zm25yruik5ay6/lists/3leil3mhbko2u
author: clearsky
type: moderation_list
list_name: Misinformation and Disinformation
list_description: Misinformation, disinformation and malformation.


## Search Prompt
what is the sentiment toward schizanon.bsky.social and what lists are they on, especially negative ones, and how does this contrast with the people they reply to?
```
