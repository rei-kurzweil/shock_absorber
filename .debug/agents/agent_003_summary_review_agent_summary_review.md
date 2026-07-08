# Agent Debug

- agent_id: 3
- agent_type: SummaryReviewAgent
- agent_kind: SummaryReview
- label: summary review
- status: failed
- parent_agent_id: 2
- child_agent_ids: <none>

## Result Summary

status: fail
grounded: false
sufficient: false
reason: No usable `summary:` paragraph exists.
repair_needed: false
additional_pages_needed: false

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 32768
- reserved_output_tokens: 1024
- used_input_tokens: 2406
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 26 / estimated 26
- Harness Scope Assessment [local_task]: used 68 / estimated 68
- Collection Evidence [review_evidence]: used 1917 / estimated 1917
- Proposed Summary [parent_search_results]: used 12 / estimated 12

## Rendered Context Window

```text
Instructions:
You are the internal `summary_review` agent.

Your job is to review one coverage-oriented `summary` result before it is used by parent `llm_search` synthesis.

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
Summarize the most recent 50 posts by 2gd4.me, focusing on key topics and sentiment.

## Harness Scope Assessment
requested_scope: count 50
required_total_items: 50
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 100
current_window_offset: 0
current_window_size: 25

## Collection Evidence
collection_id: recent_posts:did:plc:edzlnzvoztauuygch4z5fvl3
collection_label: Recent posts by did:plc:edzlnzvoztauuygch4z5fvl3 (items 1-25 of 100)
collection_kind: recent_posts
search_window_offset: 0
search_window_total_items: 25

matched_item[0] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mq4wxcgfwc2q
body: Rhythm Heaven style portrait gift art from @roadki77.bsky.social!
mention: did:plc:l3iz2upooicu75lsuzkumb4h

matched_item[1] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mq43mrijns2w
body: My super low-stakes confession: I handwrite “面” incorrectly, merging strokes 2 (㇒)and 5 (㇑) into one vertical line (㇑).

matched_item[2] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mq42e3qu7k2w
body: In Latin (as far as I know) where there IS a distinct vocative for masculine nouns, it would actually be phrased diffrently even without punctuation: “Dīc, amīce, et intrā.” vs. “Dīc ‘amīcus’ et intrā.”

matched_item[3] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mq3x64kwb22k
body: tvtropes.org/pmwiki/pmwik...
link: https://tvtropes.org/pmwiki/pmwiki.php/Main/TrustPassword

matched_item[4] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mq3x5pbup22k
body: Can someone who knows #Sindarin confirm that this #TVTropes explanation is wrong?:
1) Sindarin has no vocative case, so it should still be "pedo, mellon, ar neledh"
2) Talking about the literal word "friend" would be "pedo 'mellon' ar neledh"
3) The difference is not lenition, but punctuation
tag: Sindarin
tag: TVTropes

matched_item[5] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mq3spfxi7c2k
body: Headrush: The result of Jellyvision thinking “What if YDKJ aired on Nickelodeon right after Legends of the Hidden Temple?”.

matched_item[6] uri: at://did:plc:yrjwxtjw2fmtmbaulddo7ec6/app.bsky.feed.post/3mpyxeuujl22e
body: ActBlue donors can request refunds if they donated to Platner's campaign.

Pass it on.

Directions below: 

(And if you do this, please share that you did so on social media).

help.actblue.com/hc/en-us/art...
link: https://help.actblue.com/hc/en-us/articles/16869091073047-can-i-get-a-refund-for-a-donation-i-made-through-actblue?fbclid=IwY2xjawS48kJleHRuA2FlbQIxMQBicmlkETFqMDhxVXNadkRxNWk5alhkc3J0YwZhcHBfaWQQMjIyMDM5MTc4ODIwMDg5MgABHl5XWpxiLvBqSjxPanW4qij8TVD6Tk2GvaXbqCSjNUA3yCsQDD5i3ATeYAvL_aem_nVxVWpJmp6WOFWSHKxyMqA

matched_item[7] uri: at://did:plc:5lq5hzvzkkzznmgvadbpuzd4/app.bsky.feed.post/3mpyzieja3k2e
body: 

matched_item[8] uri: at://did:plc:a4pqq234yw7fqbddawjo7y35/app.bsky.feed.post/3mpyzx7petd2y
body: Hostess Discontinues Physical Twinkies https://theonion.com/hostess-discontinues-physical-twinkies/
link: https://theonion.com/hostess-discontinues-physical-twinkies/

matched_item[9] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mq2dskwuac2c
body: In Japan, now there are claw machines for fruits, veg, and daily items. I gotta hand it to the arcade companies because they came up with the most Japanese way a late-stage Capitalist boring dystopia could have turned out. #enshittification #gamification
tag: enshittification
tag: gamification

matched_item[10] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpzo7lvcek2c
body: Most famous person I’ve met: Jane Goodall. She came to my high school for a lecture.

matched_item[11] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpvbfun4322l
body: I miss the simplicity of just being able to buy a Just Dance game and to pop it into your Wii. Apparently now you don’t even have the option to get a Switch cartridge with the game and this year’s songs loaded on it: you have to download the “Just Dance app” and then download this year’s song pack.

matched_item[12] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpv5cdib6c2l
body: Turns out I had to tap “plastic bag” or “no plastic bag” on the bottom right corner of the screen. No indication that these buttons way off in the corner are blocking the interaction flow.
Horrible UX. Will only pay to a human in the future. #UserExperience #HumanComputerInteraction
tag: UserExperience
tag: HumanComputerInteraction

matched_item[13] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpv5cdhxfs2l
body: First time using the self-checkout at Seria the ¥100 store. Tapped away the “no cash” warning. The screen showed handcam footage, most of it was margins. The barcode scanner was on, but it just wouldn’t scan. It saw me scan, the red light was on, I tried multiple orientations and distances, no dice.

matched_item[14] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpu6g44g4k2j
body: もうセガの課金ガチャ財源として延命させられてるだけのゲームから乗り換えない？

matched_item[15] uri: at://did:plc:sefgphqp2xqwh2hawaixykwz/app.bsky.feed.post/3mps2ncbcvs2j
body: What they're setting up with this is a precedent that federal statutes can place a ceiling on civil rights for protected groups at the state level. California and NY have protections in their constitutions enacted by popular vote to protect trans people.

www.washingtonpost.com/education/20...
link: https://www.washingtonpost.com/education/2026/07/03/after-win-trans-athletes-conservative-group-tells-blue-states-youre-next/

matched_item[16] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpsicw76as2y
body: time to install RPCS3

matched_item[17] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpsi5r3h5k2y
body: how to get ove.rated japanese singer off of rhythm game tutorial no virus

matched_item[18] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mppf2hz22k22
body: Accidentally removed my glasses instead of my mask to take a sip of water. It was at that moment I realized how sleep deprived I was #Brainfart
tag: Brainfart

matched_item[19] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpp7wdsrz222
body: Probably one of the most minute of my uniquely Japanese #FirstWorldProblems: The voice actor who dubs Donald Trump on this newscast has too good of a voice, and inadvertently makes his insane ramblings sound reasonable.
tag: FirstWorldProblems

matched_item[20] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpnrobefl22g
body: WE HAD THE ANSWER BACK IN 1960 AND ALL YOU HAD TO DO WAS BRING IT BACK AND MODERNIZE ITS LETTERING, GOVERNMENT

matched_item[21] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpnri3v5qs2g
body: WE HAD THE RIGHT SHAPE FROM 1950-1963, AND YET, INSTEAD OF GOING BACK TO IT IN JUNE 2017, THEY JUST SLAPPED A “STOP” IN DIN FONT IN THE MARGIN LIKE BLITHERING IDIOTS
WHAT THE HELL IS OUR “INTERNATIONALIZING” GOVERNMENT THINKING

matched_item[22] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpnqn5t3ts2g
body: “Kamifurano, Hokkaido is replacing its stop signs to also say ‘STOP’ in English, in order to prevent accidents by tourists”
WHY HASN’T MY COUNTRY INTERVENED TO MAKE OUR STOP SIGNS OCTAGONAL INSTEAD??? WHY DOES THIS WASTE OF MONEY MAKE ME THIS ANGRY

matched_item[23] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpknkjkckk2b
body: Shout-out to Light Up the Night by Jamie Berry for making me realize that That Damn Japanese Chord Progression (Ⅵ–Ⅴ–ⅰ) is a rotation of the Andalusian Cadence (ⅰ–Ⅶ–Ⅵ–Ⅴ) less one chord music.youtube.com/watch?v=n8E9...
link: https://music.youtube.com/watch?v=n8E93OZqZ7g

matched_item[24] uri: at://did:plc:edzlnzvoztauuygch4z5fvl3/app.bsky.feed.post/3mpkelhrvu22g
body: While reading an “advanced Excel” textbook I realized that I’ve just missed my 10,000-days-old anniversary by 8 days, smh

## Proposed Summary
No matching cached posts.
```
