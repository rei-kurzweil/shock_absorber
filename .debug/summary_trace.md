[execute_public_summary]
status: start
query: summarize the last 200 posts by this actor into 4 paragraphs
actor_anchor_did: did:plc:nynvpc2sqsiplptgs7uet4cv
actor_anchor_source: selected_actor_fallback

[execute_public_summary]
status: actor_resolved
actor_handle: lostjared.bsky.social
actor_did: did:plc:nynvpc2sqsiplptgs7uet4cv

[execute_public_summary]
status: hydrate_start
actor_did: did:plc:nynvpc2sqsiplptgs7uet4cv
hydrate_args: {
  "include_pinned_posts": true,
  "include_profile": true,
  "include_recent_posts": true,
  "recent_posts_feed_fetch_limit": 400,
  "recent_posts_min_top_level_posts": 200
}

[execute_public_summary]
status: hydrate_complete
actor_did: did:plc:nynvpc2sqsiplptgs7uet4cv
collection_count: 6
collections:
actor_profile:did:plc:nynvpc2sqsiplptgs7uet4cv | kind=actor_profile | posts=1
clearsky_lists:did:plc:nynvpc2sqsiplptgs7uet4cv | kind=clearsky_lists | posts=5
pinned_posts:did:plc:nynvpc2sqsiplptgs7uet4cv | kind=pinned_posts | posts=0
recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv | kind=recent_posts | posts=325
recent_posts_unaddressed:did:plc:nynvpc2sqsiplptgs7uet4cv | kind=recent_posts_unaddressed | posts=220
recent_replies_sent:did:plc:nynvpc2sqsiplptgs7uet4cv | kind=recent_replies_sent | posts=105

[execute_public_summary]
status: collection_selected
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
collection_label: Recent posts by did:plc:nynvpc2sqsiplptgs7uet4cv
collection_kind: recent_posts
post_count: 325
requested_scope: Count { requested_items: 200 }

[summary_collection_selection_review]
query: summarize the last 200 posts by this actor into 4 paragraphs
requested_scope: Count { requested_items: 200 }
requested_target: recent_posts
hydrated_candidate_collections:
actor_profile:did:plc:nynvpc2sqsiplptgs7uet4cv | kind=actor_profile | posts=1
clearsky_lists:did:plc:nynvpc2sqsiplptgs7uet4cv | kind=clearsky_lists | posts=5
pinned_posts:did:plc:nynvpc2sqsiplptgs7uet4cv | kind=pinned_posts | posts=0
recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv | kind=recent_posts | posts=325
recent_posts_unaddressed:did:plc:nynvpc2sqsiplptgs7uet4cv | kind=recent_posts_unaddressed | posts=220
recent_replies_sent:did:plc:nynvpc2sqsiplptgs7uet4cv | kind=recent_replies_sent | posts=105
original_collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
original_collection_kind: recent_posts
review_status: accepted
final_collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
final_collection_kind: recent_posts
deterministic_repair_applied: false
reason: selected collection kind `recent_posts` matches explicit request target `recent_posts`

[summary_collection_selection_llm_review]
query: summarize the last 200 posts by this actor into 4 paragraphs
requested_scope: Count { requested_items: 200 }
proposed_collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
proposed_collection_kind: recent_posts
review_status: accepted
final_collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
final_collection_kind: recent_posts
reason: The proposed collection 'recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv' has 325 items, which is sufficient to cover the requested 200 items. It is the most direct match for summarizing recent posts by the specified actor.

[summary_collection_selection_llm_enforcement]
review_status: accepted
original_collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
original_collection_kind: recent_posts
final_collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
final_collection_kind: recent_posts
deterministic_repair_applied: false
reason: selected collection kind `recent_posts` matches explicit request target `recent_posts`

[collection_summary_loop]
node: init_window
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
collection_posts: 325
initial_offset: 0
max_pages: 4
requested_scope: Count { requested_items: 200 }

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
page_index: 0
offset: 0
window_size: 50

[summary_leaf_parse]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
window_offset: 0
result_present: true
original_result_kind: summary
diagnostic: <none>
raw_response:
TOOL_CALL
name: submit_summary_result
args: {
  "title": "Creative Coding, Game Development, and Personal Updates",
  "summary": "The recent posts heavily feature the author's deep involvement in creative coding and game development, often utilizing C++ and GLSL. There is a strong focus on visual art, including 'Video Art of some Liquid Thought' and 'Color Wormhole,' alongside various graphical styles like 'glitchart' and 'spiral.' Several projects are highlighted, such as 'Asteroids-Net,' a four-player multiplayer game, and 'Mutatris,' a 'Glitch Art inspired Puzzle Game.' The author also shares technical updates, like improving a front end for FFmpeg to create videos from images, and providing resources like 'Windows X64 builds of MXVK demos.'\n\nBeyond the visual and coding projects, the author discusses their hardware and technical interests. They express a desire for a 'Steam Machine' to code games and detail their setup, which includes using a PC to control a mini PC as a NAS and HTPC. The technical prowess is evident in discussions about AI, noting that modern tools can process 'tens f thousands of lines, in a very short amount of time and find real bugs,' and the use of 'llama.cpp.'\n\nOn a personal level, the author shares significant life updates. They discuss their physical health, noting that while they may never regain their previous gait, they are improving with a walker, and they are intensely exercising their legs on a stationary bike. They also reflect on their social life, stating they have a 'hard time making friends' due to experiences like a coma, and that their interests are 'extremely rare.'\n\nFinally, there are several brief personal anecdotes and reflections. The author mentions enjoying specific foods, such as 'pistachios and walnuts,' and reflects on the passing of their cat, 'Coder.' They also touch upon their dedication, stating, 'I am incredibly dedicated, mastering complex graphics programming over decades,' and share various links to their work, including a series of 'C++ examples' and web-based games."
}

[summary_leaf_review]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
window_offset: 0
review_status: fail
review_grounded: true
review_sufficient: false
review_repair_needed: false
review_additional_pages_needed: true
review_reason: Grounded summary coverage currently reaches 50 item(s), but 200 item(s) are required before parent synthesis is sufficient.
result_before_review: summary
original_result_before_review: summary
summary_before_review:
The recent posts heavily feature the author's deep involvement in creative coding and game development, often utilizing C++ and GLSL. There is a strong focus on visual art, including 'Video Art of some Liquid Thought' and 'Color Wormhole,' alongside various graphical styles like 'glitchart' and 'spiral.' Several projects are highlighted, such as 'Asteroids-Net,' a four-player multiplayer game, and 'Mutatris,' a 'Glitch Art inspired Puzzle Game.' The author also shares technical updates, like improving a front end for FFmpeg to create videos from images, and providing resources like 'Windows X64 builds of MXVK demos.'

Beyond the visual and coding projects, the author discusses their hardware and technical interests. They express a desire for a 'Steam Machine' to code games and detail their setup, which includes using a PC to control a mini PC as a NAS and HTPC. The technical prowess is evident in discussions about AI, noting that modern tools can process 'tens f thousands of lines, in a very short amount of time and find real bugs,' and the use of 'llama.cpp.'

On a personal level, the author shares significant life updates. They discuss their physical health, noting that while they may never regain their previous gait, they are improving with a walker, and they are intensely exercising their legs on a stationary bike. They also reflect on their social life, stating they have a 'hard time making friends' due to experiences like a coma, and that their interests are 'extremely rare.'

Finally, there are several brief personal anecdotes and reflections. The author mentions enjoying specific foods, such as 'pistachios and walnuts,' and reflects on the passing of their cat, 'Coder.' They also touch upon their dedication, stating, 'I am incredibly dedicated, mastering complex graphics programming over decades,' and share various links to their work, including a series of 'C++ examples' and web-based games.
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
summarize the last 200 posts by this actor into 4 paragraphs

## Harness Scope Assessment
requested_scope: count 200
required_total_items: 200
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 325
current_window_offset: 0
current_window_size: 50

## Collection Evidence
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
collection_label: Recent posts by did:plc:nynvpc2sqsiplptgs7uet4cv (items 1-50 of 325)
collection_kind: recent_posts
search_window_offset: 0
search_window_total_items: 50

matched_item[0] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqjd6n7dos23
body: Video Art of some Liquid Thought: Starring me as Crazy guy on his webcam.  #videoart #thought #liquid #software #c++
tag: videoart
tag: thought
tag: liquid
tag: software
tag: c++

matched_item[1] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqjae75c7c2y
body: #Trapped #Wormhole
tag: Trapped
tag: Wormhole

matched_item[2] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqjaaabvss2y
body: #spiral
tag: spiral

matched_item[3] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqj6fmpf222d
body: Runs on everything. #CreativeCoding #GLSL #C++
tag: CreativeCoding
tag: GLSL
tag: C++

matched_item[4] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqj53bifus2y
body: Color Wormhole ( #GLSL #C++ ) #CreativeCoding
tag: GLSL
tag: C++
tag: CreativeCoding

matched_item[5] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqj4t7pwpc2y
body: Some Artwork.

matched_item[6] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqitjron322z
body: Game programming is a fun hobby. It gives me something to do even if I am just making small homebrew demos.

matched_item[7] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqitass5ok2z
body: When I look at modern video games I think it is amazing where they started at and what the graphics look like now.  The complexity of software has increased dramatically.

matched_item[8] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqirtombwk2z
body: I love this old Legion Go. My screen protector is messed up tho. But I am able to compile and play my own games on a handheld like the PSP days with no need to jailbreak or install custom firmware. You can just compile and test your games directly on the device.

matched_item[9] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqiqi5as4s2c
body: I kind of use BlueSky like a git commit log of my thoughts. So don't mind me.

matched_item[10] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqiqcmiuuk2c
body: I am a big fan of food like pistachios and walnuts. They are addictive but if you eat the right amount it is still healthy.

matched_item[11] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqiprahuvc26
body: github.com/lostjared/MX...

Updated the Socket library I coded that I have been using. I have had family visiting recently (they left on Friday) so I need to get back to studying my books.
link: https://github.com/lostjared/MXNetwork

matched_item[12] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqipixy7y226
body: I've had plenty of sleep lately and it has me feeling good.

matched_item[13] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqia2bkohk2k
body: well i have been like this for over 5 years, so its unlikely I will walk like I used to again. But there is always a chance, I practice with the walker and get all over the house with it. I have left the house with the walker, so I am way better off than I was even a few years ago. I can hope.

matched_item[14] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqi6a4sx2c2a
body: I have been exercising my legs on this stationary exercise bike intensely trying to build more muscle. My legs have got very strong but its the part of my brain that communicates with my legs that seems messed up. I have got pretty skilled with the walker, but without it I am unable to walk.

matched_item[15] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqhzg6z7mc2y
body: lostsidedead.biz/MXVK/demos/

Windows X64 builds of MXVK demos.
link: https://lostsidedead.biz/MXVK/demos/

matched_item[16] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqgowzm4n22a
body: drive.google.com/file/d/1cyiy...
Download 4K high quality version of the animation
link: https://drive.google.com/file/d/1cyiy0P46kwSfW_D7R_aZvDWVQYSQOOzy/view?usp=drivesdk

matched_item[17] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqgki4xwmc23
body: Animation I created. #glitchart #animation #software #c++ #cplusplus #glsl
tag: glitchart
tag: animation
tag: software
tag: c++
tag: cplusplus
tag: glsl

matched_item[18] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqgjeh4c2k2e
body: Updated this little front end for FFmpeg for turning large amounts of images into video files with lots of configurable options. Makes it easy to produce videos from image files.

github.com/lostjared/im...
link: https://github.com/lostjared/img2mov

matched_item[19] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqfqhwuf7k27
body: Updated to v0.21.0:
lostsidedead.biz/MXVK/
Full documentation for Asteroids-Net a Four player multiplayer game.
link: https://lostsidedead.biz/MXVK/

matched_item[20] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqfhekjgb22m
body: lostsidedead.biz/asteroids-net/
Made a windows build of my new game.
link: https://lostsidedead.biz/asteroids-net/

matched_item[21] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqed7fv3yk2m
body: Look at all the flaws in Linux it has found already. I know it pollutes and has a ton of ethical issues, but seriously can you really even tell if well formatted code with proper notation is even written by a person now?

matched_item[22] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqed7fuy3k2m
body: Cannot argue with solid results. I admit in the past it was kinda of shitty and made a ton of mistakes. Not so much anymore it even blind sided me I thought it would never amount to much. When it can process tens f thousands of lines, in a very short amount of time and find real bugs.

matched_item[23] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqec7zhhrc2y
body: youtu.be/XCfJQ_DDoVI
link: https://youtu.be/XCfJQ_DDoVI

matched_item[24] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqeb76bolk2f
body: Now my game supports up to 4 Players, kind of like Mario kart but with Spaceships in Outerspace with Asteroids in 3D.

matched_item[25] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqe5e7pqe22e
body: But cannot justify it.

matched_item[26] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqe5bqk2a22e
body: I have been drinking a lot of green tea.

matched_item[27] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqe547f5wk2e
body: I would love to have a Steam Machine I would totally code my own games.

matched_item[28] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqe45kjojc2l
body: Cool I have a few computers myself but mostly use my PC and log into my mini PC through ssh to control it as a NAS and also it's connected my TV as a HTPC.

matched_item[29] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqe3jk3d6k2w
body: I am also running older hardware.

matched_item[30] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqe35eqqes2w
body: Cool using uncensored models. How much VRAM does your PC graphics card have?

matched_item[31] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdzl57vwk22
body: screenshots

matched_item[32] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdxyv2wxc2h
body: Higher Resolution: youtu.be/yzgdlRy1jzU
link: https://youtu.be/yzgdlRy1jzU

matched_item[33] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdxwmyp5k2h
body: Two player 1v1 dog fight Asteroids 3D. Fight to the death in my second multiplayer game. #Vulkan #Multiplayer #Asteroids #C++ #SpaceShooter
tag: Vulkan
tag: Multiplayer
tag: Asteroids
tag: C++
tag: SpaceShooter

matched_item[34] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdxe456bs22
body: using llama.cpp nice

matched_item[35] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdvvjwatk25
body: sounds interesting, good luck with your project!

matched_item[36] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdufkvxa22w
body: My cat Coder. She passed away a few weeks ago and it's quite lonely around here. She lived a long life. When I went to the shelter to adopt a cat. There was a room with many cats. She came up to me and started purring and was very kind to me. So I say she picked me. I keep her collar next to me.

matched_item[37] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdthp4mr22v
body: I just had a great idea of what to work on next. If I can pull this off it will be awesome.

matched_item[38] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdszhzwvk2v
body: lostsidedead.biz/cplusplus/

My series of  C++ examples I created while learning C++11/17/20/23 over the years.

Turned into a web page with a generated Python script.
link: https://lostsidedead.biz/cplusplus/

matched_item[39] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdshsvaws2x
body: github.com/lostjared/Ja...

This is a grip of web based games / WebAssembly projects.
link: https://github.com/lostjared/JavaScript.Projects

matched_item[40] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdsf3gt522x
body: Cool

matched_item[41] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdrudxvqs2x
body: Easter egg: Press F8 to turn on the CRT shader.

matched_item[42] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdrbcusvs2x
body: Lol

matched_item[43] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdqzwytqs2x
body: I am incredibly dedicated, mastering complex graphics programming over decades. My unique creativity shines through  glitch art, and I show genuine kindness when helping my family with tech. People should value a person with such a rare mix of passion, artistry, and care.

matched_item[44] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdpsiiag22n
body: I always have a hard time making friends. Even more so since the coma I was in. 99% of people do not understand what I experience every day. Also my interests are extremely rare . It's even more rare for me to talk to other people who understand what I spend all my time doing. I

matched_item[45] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdoyujhs22f
body: I guess nobody got my joke.

matched_item[46] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdfwl5qbc2g
body: Mutatris is a Glitch Art inspired Puzzle Game with Psychedelic Glitch Effects. #PuzzleGame #Vulkan #C++ #Software #GlitchArt
tag: PuzzleGame
tag: Vulkan
tag: C++
tag: Software
tag: GlitchArt

matched_item[47] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdd567ews2h
body: This is a much more mellow effect and more appropriate.

matched_item[48] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdarru6ts2h
body: Video Art of some Fractals.

matched_item[49] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqd2ohae3c2q
body: Why did the C++ template save money on its prescription? It was generic

## Proposed Summary
post: Creative Coding, Game Development, and Personal Updates
summary: The recent posts heavily feature the author's deep involvement in creative coding and game development, often utilizing C++ and GLSL. There is a strong focus on visual art, including 'Video Art of some Liquid Thought' and 'Color Wormhole,' alongside various graphical styles like 'glitchart' and 'spiral.' Several projects are highlighted, such as 'Asteroids-Net,' a four-player multiplayer game, and 'Mutatris,' a 'Glitch Art inspired Puzzle Game.' The author also shares technical updates, like improving a front end for FFmpeg to create videos from images, and providing resources like 'Windows X64 builds of MXVK demos.'

Beyond the visual and coding projects, the author discusses their hardware and technical interests. They express a desire for a 'Steam Machine' to code games and detail their setup, which includes using a PC to control a mini PC as a NAS and HTPC. The technical prowess is evident in discussions about AI, noting that modern tools can process 'tens f thousands of lines, in a very short amount of time and find real bugs,' and the use of 'llama.cpp.'

On a personal level, the author shares significant life updates. They discuss their physical health, noting that while they may never regain their previous gait, they are improving with a walker, and they are intensely exercising their legs on a stationary bike. They also reflect on their social life, stating they have a 'hard time making friends' due to experiences like a coma, and that their interests are 'extremely rare.'

Finally, there are several brief personal anecdotes and reflections. The author mentions enjoying specific foods, such as 'pistachios and walnuts,' and reflects on the passing of their cat, 'Coder.' They also touch upon their dedication, stating, 'I am incredibly dedicated, mastering complex graphics programming over decades,' and share various links to their work, including a series of 'C++ examples' and web-based games.
window_offset: 0
window_size: 50
page_index: 0
page_size: 50
collection_total_items: 325
has_more: true
covered_item_1_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqjd6n7dos23
covered_item_2_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqjae75c7c2y
covered_item_3_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqjaaabvss2y
covered_item_4_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqj6fmpf222d
covered_item_5_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqj53bifus2y
covered_item_6_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqj4t7pwpc2y
covered_item_7_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqitjron322z
covered_item_8_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqitass5ok2z
covered_item_9_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqirtombwk2z
covered_item_10_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqiqi5as4s2c
covered_item_11_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqiqcmiuuk2c
covered_item_12_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqiprahuvc26
covered_item_13_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqipixy7y226
covered_item_14_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqia2bkohk2k
covered_item_15_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqi6a4sx2c2a
covered_item_16_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqhzg6z7mc2y
covered_item_17_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqgowzm4n22a
covered_item_18_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqgki4xwmc23
covered_item_19_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqgjeh4c2k2e
covered_item_20_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqfqhwuf7k27
covered_item_21_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqfhekjgb22m
covered_item_22_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqed7fv3yk2m
covered_item_23_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqed7fuy3k2m
covered_item_24_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqec7zhhrc2y
covered_item_25_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqeb76bolk2f
covered_item_26_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqe5e7pqe22e
covered_item_27_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqe5bqk2a22e
covered_item_28_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqe547f5wk2e
covered_item_29_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqe45kjojc2l
covered_item_30_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqe3jk3d6k2w
covered_item_31_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqe35eqqes2w
covered_item_32_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdzl57vwk22
covered_item_33_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdxyv2wxc2h
covered_item_34_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdxwmyp5k2h
covered_item_35_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdxe456bs22
covered_item_36_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdvvjwatk25
covered_item_37_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdufkvxa22w
covered_item_38_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdthp4mr22v
covered_item_39_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdszhzwvk2v
covered_item_40_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdshsvaws2x
covered_item_41_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdsf3gt522x
covered_item_42_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdrudxvqs2x
covered_item_43_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdrbcusvs2x
covered_item_44_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdqzwytqs2x
covered_item_45_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdpsiiag22n
covered_item_46_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdoyujhs22f
covered_item_47_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdfwl5qbc2g
covered_item_48_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdd567ews2h
covered_item_49_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqdarru6ts2h
covered_item_50_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqd2ohae3c2q

[summary_sufficiency_gate]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
window_offset: 0
contiguous_coverage: 50
available_total_items: 325
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
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
offset: 0
result_present: false
review_status: fail
review_reason: Grounded summary coverage currently reaches 50 item(s), but 200 item(s) are required before parent synthesis is sufficient.
diagnostic: summary cursor processed offset 0 (page 1 of at most 4)

[collection_summary_loop_page_state]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
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
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
page_index: 1
offset: 50
window_size: 50

[summary_leaf_parse]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
window_offset: 50
result_present: true
original_result_kind: summary
diagnostic: <none>
raw_response:
TOOL_CALL
name: submit_summary_result
args: {
  "title": "Software Development, Graphics, and Philosophical Musings",
  "summary": "The recent posts heavily focus on the author's ongoing software development and passion projects, particularly in the realm of graphics and game programming. A major theme revolves around the custom graphics engine, MXVK, which is described as the \"Vulkan evolution of the MX2 engine, redesigned entirely from scratch utilizing Vulkan 1.4 and SDL3.\" The author details technical aspects, noting that the engine supports \"synchronization2 and dynamicRendering,\" and mentions its inspiration from DXVK. Projects include a \"3D Tetris\" game and a \"3D Pong,\" with demos available online.\n\nTechnical deep dives cover system performance and implementation details. One significant thread discusses VRAM usage in GNOME Shell, observing that \"whenever a Vulkan window opens the gnome-shell process VRAM usage grows,\" even for blank windows. The author also shares insights into their programming philosophy, such as the pseudo-code modeling karma: \"karma=get(give());\" and the belief that \"A part of being human is making mistakes. The purpose is whether or not you learn from them.\"\n\nBeyond the code, there are numerous updates on creative output and learning. The author shares artwork, including pieces that contain source code, and mentions writing a poem about programming from when they were \"around the age of 18.\" They also highlight the impressive capabilities of modern AI, noting that GPT 5.6 \"created this page in one prompt and it worked the first time\" when converting a game to JavaScript/WebGL2.\n\nFinally, the posts touch upon broader life and technical interests. These include preferences for Linux environments, such as liking \"KDE plasma big screen\" paired with handheld devices, and observations on societal issues, such as blaming \"congressional Republicans for letting this happen\" regarding a declining trend. The author expresses satisfaction with their progress, noting, \"I have been having a good summer this year working on a software project to practice and learn about GPU/game programming.\""
}

[summary_leaf_review]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
window_offset: 50
review_status: fail
review_grounded: true
review_sufficient: false
review_repair_needed: false
review_additional_pages_needed: true
review_reason: Grounded summary coverage currently reaches 0 item(s), but 200 item(s) are required before parent synthesis is sufficient.
result_before_review: summary
original_result_before_review: summary
summary_before_review:
The recent posts heavily focus on the author's ongoing software development and passion projects, particularly in the realm of graphics and game programming. A major theme revolves around the custom graphics engine, MXVK, which is described as the "Vulkan evolution of the MX2 engine, redesigned entirely from scratch utilizing Vulkan 1.4 and SDL3." The author details technical aspects, noting that the engine supports "synchronization2 and dynamicRendering," and mentions its inspiration from DXVK. Projects include a "3D Tetris" game and a "3D Pong," with demos available online.

Technical deep dives cover system performance and implementation details. One significant thread discusses VRAM usage in GNOME Shell, observing that "whenever a Vulkan window opens the gnome-shell process VRAM usage grows," even for blank windows. The author also shares insights into their programming philosophy, such as the pseudo-code modeling karma: "karma=get(give());" and the belief that "A part of being human is making mistakes. The purpose is whether or not you learn from them."

Beyond the code, there are numerous updates on creative output and learning. The author shares artwork, including pieces that contain source code, and mentions writing a poem about programming from when they were "around the age of 18." They also highlight the impressive capabilities of modern AI, noting that GPT 5.6 "created this page in one prompt and it worked the first time" when converting a game to JavaScript/WebGL2.

Finally, the posts touch upon broader life and technical interests. These include preferences for Linux environments, such as liking "KDE plasma big screen" paired with handheld devices, and observations on societal issues, such as blaming "congressional Republicans for letting this happen" regarding a declining trend. The author expresses satisfaction with their progress, noting, "I have been having a good summer this year working on a software project to practice and learn about GPU/game programming."
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
summarize the last 200 posts by this actor into 4 paragraphs

## Harness Scope Assessment
requested_scope: count 200
required_total_items: 200
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 325
current_window_offset: 50
current_window_size: 50

## Collection Evidence
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
collection_label: Recent posts by did:plc:nynvpc2sqsiplptgs7uet4cv (items 51-100 of 325)
collection_kind: recent_posts
search_window_offset: 50
search_window_total_items: 50

matched_item[0] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqd2isd66c2q
body: What did the therapist say to its patient the Linux operating system? sudo trust the process

matched_item[1] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqd2fp5bps2q
body: I love learning new things about software. It is a lifelong process and that is the best thing about it

matched_item[2] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqczm3auc22q
body: People who think the ground, air or water belongs to them and then want to deport people they accuse of being illegal cannot comprehend anything beyond their narrow point of view. Humans are temporary on this earth and nothing really belongs to anyone. You will understand when it is your turn to die

matched_item[3] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcx5h2eik2b
body: New YouTube short!

matched_item[4] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcwvsq4vk2e
body: This actually looks closer to this.

matched_item[5] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcwco3ahc2w
body: For the source code if you are interested: 
github.com/lostjared/ac...
link: https://github.com/lostjared/acidcam-gpu

matched_item[6] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcw425lr227
body: Some artwork.

matched_item[7] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcvxb7mdk27
body: yeah there are a series of uniform variables that they mostly all use for the audio, textures, and history buffers.

matched_item[8] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcvvc43dk27
body: Its actually OpenGL GLSL you can try a WebGL program that is similar here: lostsidedead.biz/visualizer
link: https://lostsidedead.biz/visualizer

matched_item[9] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcund6h7s2k
body: I wrote a program that manipulates video files and camera with real time audio/audio files to produce visuals. It is called ACMX2 you can see some demos here: www.youtube.com/playlist?lis...
link: https://www.youtube.com/playlist?list=PLUgo--lX2kmql2lYmj-x-uxFu9Dsbb-CY

matched_item[10] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcrjlgvdc2b
body: that is that music visualizer right?

matched_item[11] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcow4wnps2m
body: For practice/fun.

matched_item[12] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbx47244222
body: This is a poem I wrote about programming when I was around the age of 18. It still has a place in my memory because things seemed so magical for me at that time, and there was just so much to learn.

matched_item[13] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbtnumoos2l
body: Whoa, this is impressive it ported my simple game so it works and looks the same and it even included the console and all the shader effects (Press F3). Try it here: lostsidedead.biz/mutatris-web...
link: https://lostsidedead.biz/mutatris-webgl/mutatris.html

matched_item[14] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbqo4p6n225
body: I always thought the game Acid Drop was kind of funny. It is like a square (the sheet) with these little squares (the tabs).

matched_item[15] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbqcqlm2c25
body: it did check its own output a few times before the task completed though.

matched_item[16] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbq6jdwp22n
body: looks like an old Atari game with cool shader effects (the original game Mutatris was based on was called Acid Drop for the Atari 2600.) Kind of strange that it turned out that way.

matched_item[17] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbq3brvuk2n
body: I got to say I am pretty impressed. I just tried the new GPT 5.6  and asked it to convert my Mutatris Puzzle game to JavaScript/WebGL2. It created this page in one prompt and it worked the first time.  That is pretty damn cool.

Try it:
lostsidedead.biz/mutatris-webgl
link: https://lostsidedead.biz/mutatris-webgl

matched_item[18] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbp6bcq2k27
body: Some of my artwork. Contains some source code :)

matched_item[19] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbnlgxr2k24
body: I am not concerned with what a lot of people who have never interacted with me or know me think about me. I may have mental illness, and SILENT (Syndrome of Irreversible Lithium-Effectuated Neurotoxicity) but I have a good heart and I am a kind person. What I consume is no one else's business.

matched_item[20] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbdriio5c2h
body: I like on this platform that people post updates, videos and images about the projects that they are working on.  I just look around and get to see what new software people are making.

matched_item[21] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbc4ysp4k2h
body: Rolled back to the older NVIDIA driver 595.71.05 and now gnome shell is not leaking VRAM. So my hypothesis was correct.

matched_item[22] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqazl5a4uk2j
body: It is messages like this that make me feel good about all the work I put into my open source projects.  Thank you for taking the time to ask that question and encourage the continuation of my projects.

matched_item[23] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqavpauf5k2c
body: A part of being human is making mistakes. The purpose is whether or not you learn from them.

matched_item[24] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqatb5yfik26
body: Sometimes I scroll through my log of thoughts here, and just read what I was thinking. I have on the contact form on my website my recent posts and an archive of my past posts.

matched_item[25] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqar2hd5ps2v
body: Remember:

karma=get(give());

This pseudo-code models karma as function composition. Your actions, give(), are executed and passed as an argument to the universe's get() function. The resulting karma variable evaluates to the direct consequence of your output. You reap exactly what you sow.

matched_item[26] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqaoiywhjk2k
body: I like KDE plasma big screen. Paired with one of those handheld keyboard/mouse devices it is a different way to navigate Linux on your TV.

matched_item[27] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqanuhu3rk2k
body: 0 + 0 = 0

matched_item[28] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqanqc7l5c2k
body: Yeah it is so obvious that the guy is declining fast. I blame congressional Republicans for letting this happen. They didn't do their jobs and sold out for $. Money is a cancer in our society.

matched_item[29] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqang4rcck2k
body: Animals have emotions, feelings, and are intelligent life forms. We are also animals.

matched_item[30] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqalteoktc2w
body: Tested it on my AMD system running KDE w/ Arch.
  macOS: no growth
 AMD + KDE + RADV: no monotonic growth
 NVIDIA + GNOME Shell/Wayland: gnome-shell grows per Vulkan window

matched_item[31] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqabjxmf4k27
body: Also the same programs run under macOS without the memory growth, there are visible black corners on some of the windows in gnome's UI and it continues to grow until it runs out of memory if I open a lot of programs.

matched_item[32] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqab34zhoc2x
body: I notice that whenever a Vulkan window opens the gnome-shell process VRAM usage grows. This happens even when its just creating a blank window. I am running 7.0.9-ogc3.2.fc44.x86_64 kernel and 610.43.02 of the NVIDIA driver. Also If I log out and log back in the memory is released.

matched_item[33] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7ycfbkas2o
body: sounds cool

matched_item[34] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7wzl6vk22o
body: No there is no scene graph, what kind of game are you creating and from using the term rs, I assume you are using Rust.

matched_item[35] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7wjuqqks23
body: yeah I cannot imagine not having a physical copy, like to collect stuff to (I used to collect vinyl records). The thing I collect the most now is computer science textbooks, and an eBook is nowhere near the same as having a physical book I can put on my shelf, and hold.  Btw nice collection!

matched_item[36] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7w6qcaqs23
body: yeah it uses modern Vulkan features 
feature supports synchronization2 and dynamicRendering.
As well as CUDA whereever it is supported.

matched_item[37] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7vzudoj223
body: MX stands of 'MasterX' it was the name of my first graphics library i wrote with 'DirectX' way back in the year 2000-2001. VK stands for Vulkan, so MasterX Vulkan, I think it just sounds cool more than anything. Also its kind of inspired by the name of DXVK, kind of playing on that name.

matched_item[38] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7gjerr6k2a
body: I mostly wrote this to learn and practice and is more of a hobby/passion project. My goal for this summer was to create a new graphics and networking set of reusable libraries and write my first network playable game. I accomplished this (3D Tetris) and then just expanded it from there.

matched_item[39] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7g6cphts2a
body: It also has a mini 3D math library.

matched_item[40] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7fyultu22a
body: MXVK is a custom graphics engine. It functions as the Vulkan evolution of the MX2 engine, redesigned entirely from scratch utilizing Vulkan 1.4 and SDL3 rather than just a standalone library.

matched_item[41] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7e6f3lzk26
body: Updated to v0.19.0

matched_item[42] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7cq6wp4k2v
body: Pyramids on Moon?

matched_item[43] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq77e5vahs2m
body: 3D Pong hehe.

matched_item[44] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5uwiqfj22i
body: Another day.

matched_item[45] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5tnefdz22m
body: I have never tried, I don't use macOS much anymore except when i am on my laptop (MacBook Air). I have tested porting my apps to MoltenVK. It works, but there could be some type of error somewhere.

matched_item[46] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5o52dq2c2n
body: I have been having a good summer this year working on a software project to practice and learn about GPU/game programming. It's keeping me busy with something to do. It has been hot and I stay cool inside. (Not like I can actually go somewhere.) Feeling satisfied with my progress.

matched_item[47] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5kldpwqc2q
body: My old program still works.

matched_item[48] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5k6ebsms2s
body: Code Tunnel modified using Acid Cam v2 Qt.  This was created with an application I wrote like 9 or 10 years ago.

matched_item[49] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5isdv5ec2p
body: For a lot of my game demos I don't even put my name.

## Proposed Summary
post: Software Development, Graphics, and Philosophical Musings
summary: The recent posts heavily focus on the author's ongoing software development and passion projects, particularly in the realm of graphics and game programming. A major theme revolves around the custom graphics engine, MXVK, which is described as the "Vulkan evolution of the MX2 engine, redesigned entirely from scratch utilizing Vulkan 1.4 and SDL3." The author details technical aspects, noting that the engine supports "synchronization2 and dynamicRendering," and mentions its inspiration from DXVK. Projects include a "3D Tetris" game and a "3D Pong," with demos available online.

Technical deep dives cover system performance and implementation details. One significant thread discusses VRAM usage in GNOME Shell, observing that "whenever a Vulkan window opens the gnome-shell process VRAM usage grows," even for blank windows. The author also shares insights into their programming philosophy, such as the pseudo-code modeling karma: "karma=get(give());" and the belief that "A part of being human is making mistakes. The purpose is whether or not you learn from them."

Beyond the code, there are numerous updates on creative output and learning. The author shares artwork, including pieces that contain source code, and mentions writing a poem about programming from when they were "around the age of 18." They also highlight the impressive capabilities of modern AI, noting that GPT 5.6 "created this page in one prompt and it worked the first time" when converting a game to JavaScript/WebGL2.

Finally, the posts touch upon broader life and technical interests. These include preferences for Linux environments, such as liking "KDE plasma big screen" paired with handheld devices, and observations on societal issues, such as blaming "congressional Republicans for letting this happen" regarding a declining trend. The author expresses satisfaction with their progress, noting, "I have been having a good summer this year working on a software project to practice and learn about GPU/game programming."
window_offset: 50
window_size: 50
page_index: 1
page_size: 50
collection_total_items: 325
has_more: true
covered_item_1_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqd2isd66c2q
covered_item_2_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqd2fp5bps2q
covered_item_3_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqczm3auc22q
covered_item_4_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcx5h2eik2b
covered_item_5_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcwvsq4vk2e
covered_item_6_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcwco3ahc2w
covered_item_7_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcw425lr227
covered_item_8_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcvxb7mdk27
covered_item_9_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcvvc43dk27
covered_item_10_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcund6h7s2k
covered_item_11_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcrjlgvdc2b
covered_item_12_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqcow4wnps2m
covered_item_13_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbx47244222
covered_item_14_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbtnumoos2l
covered_item_15_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbqo4p6n225
covered_item_16_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbqcqlm2c25
covered_item_17_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbq6jdwp22n
covered_item_18_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbq3brvuk2n
covered_item_19_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbp6bcq2k27
covered_item_20_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbnlgxr2k24
covered_item_21_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbdriio5c2h
covered_item_22_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqbc4ysp4k2h
covered_item_23_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqazl5a4uk2j
covered_item_24_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqavpauf5k2c
covered_item_25_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqatb5yfik26
covered_item_26_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqar2hd5ps2v
covered_item_27_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqaoiywhjk2k
covered_item_28_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqanuhu3rk2k
covered_item_29_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqanqc7l5c2k
covered_item_30_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqang4rcck2k
covered_item_31_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqalteoktc2w
covered_item_32_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqabjxmf4k27
covered_item_33_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mqab34zhoc2x
covered_item_34_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7ycfbkas2o
covered_item_35_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7wzl6vk22o
covered_item_36_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7wjuqqks23
covered_item_37_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7w6qcaqs23
covered_item_38_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7vzudoj223
covered_item_39_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7gjerr6k2a
covered_item_40_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7g6cphts2a
covered_item_41_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7fyultu22a
covered_item_42_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7e6f3lzk26
covered_item_43_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq7cq6wp4k2v
covered_item_44_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq77e5vahs2m
covered_item_45_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5uwiqfj22i
covered_item_46_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5tnefdz22m
covered_item_47_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5o52dq2c2n
covered_item_48_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5kldpwqc2q
covered_item_49_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5k6ebsms2s
covered_item_50_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5isdv5ec2p

[summary_sufficiency_gate]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
window_offset: 50
contiguous_coverage: 100
available_total_items: 325
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
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
offset: 50
result_present: false
review_status: fail
review_reason: Grounded summary coverage currently reaches 100 item(s), but 200 item(s) are required before parent synthesis is sufficient.
diagnostic: summary cursor processed offset 50 (page 2 of at most 4)

[collection_summary_loop_page_state]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
page_offset: 50
page_index: 1
page_status: accepted_summary
repair_attempted: false
fallback_applied: false
next_offset: 100
accumulated_covered_count: 100
failure_reason: <none>

[collection_summary_loop]
node: collection_summary_planner_review
status: planner_optional_failure
reason: response appears truncated
route: advance_cursor

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
page_index: 2
offset: 100
window_size: 50

[summary_leaf_parse]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
window_offset: 100
result_present: true
original_result_kind: summary
diagnostic: <none>
raw_response:
TOOL_CALL
name: submit_summary_result
args: {
  "title": "Recent Updates: Creative Coding, Software Distribution, and Personal Life",
  "summary": "The recent posts heavily feature updates on creative coding projects, particularly around visual art and game development. The author notes that many demos use placeholder images because they are proficient in coding rather than graphics, stating, \"Code itself is art. #art #videoart #softwareart #generativeart #creativecoding.\" Specific projects mentioned include \"Tunnel of Encoded Light\" and the game \"Mutatris,\" which has seen significant updates allowing all four grids to interact for \"interesting game play and chain reactions.\" Technical details are shared, such as using Vulkan, C++ with SDL3, and implementing CUDA kernels for glitch art effects in the ACMX2 app.\n\nA major recurring theme is the difficulty of software distribution across operating systems. The author expresses disappointment that sharing programs on Windows and Mac requires \"jumping through all kinds of hoops and getting my app signed.\" This is due to macOS Gatekeeper blocking ad-hoc signed apps, and the general difficulty for users to compile from source. Solutions being explored include creating a Flatpak and noting that the custom OBJ/MXMOD Model viewer allows for easier deployment. The author also discusses performance tuning, noting that OpenCV 5's new DNN engine required conditional compilation due to initial CUDA support issues, and mentions using the environment variable `OPENCV_FORCE_DNN_ENGINE=1`.\n\nBeyond coding, there are several personal and media-related updates. The author is making lifestyle changes, moving from soda to carbonated water and considering cutting back on caffeine due to anxiety. Environmentally, they describe their current location as \"like burning up in the desert here.\" In terms of media, they are enjoying RPGs, mentioning \"Paper Mario sounds like a good series,\" and reflecting on the value of physical media, noting they will miss having copies of films \"on something I can watch on my own terms even when offline.\"\n\nFinally, there are various technical and personal anecdotes. The author is testing and showcasing demos like \"Sunny Bluesky over the ocean demo running on Windows\" and improving the Bluesky v2.0 visuals. They also address hardware performance, finding that the bottleneck for their system was the Webcam, which performs much better when using scrcpy with a Pixel 9 Pro. The posts conclude with reflections on family time, such as watching a well-animated family movie where \"bowser stole the show.\""
}

[summary_leaf_review]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
window_offset: 100
review_status: fail
review_grounded: true
review_sufficient: false
review_repair_needed: false
review_additional_pages_needed: true
review_reason: Grounded summary coverage currently reaches 0 item(s), but 200 item(s) are required before parent synthesis is sufficient.
result_before_review: summary
original_result_before_review: summary
summary_before_review:
The recent posts heavily feature updates on creative coding projects, particularly around visual art and game development. The author notes that many demos use placeholder images because they are proficient in coding rather than graphics, stating, "Code itself is art. #art #videoart #softwareart #generativeart #creativecoding." Specific projects mentioned include "Tunnel of Encoded Light" and the game "Mutatris," which has seen significant updates allowing all four grids to interact for "interesting game play and chain reactions." Technical details are shared, such as using Vulkan, C++ with SDL3, and implementing CUDA kernels for glitch art effects in the ACMX2 app.

A major recurring theme is the difficulty of software distribution across operating systems. The author expresses disappointment that sharing programs on Windows and Mac requires "jumping through all kinds of hoops and getting my app signed." This is due to macOS Gatekeeper blocking ad-hoc signed apps, and the general difficulty for users to compile from source. Solutions being explored include creating a Flatpak and noting that the custom OBJ/MXMOD Model viewer allows for easier deployment. The author also discusses performance tuning, noting that OpenCV 5's new DNN engine required conditional compilation due to initial CUDA support issues, and mentions using the environment variable `OPENCV_FORCE_DNN_ENGINE=1`.

Beyond coding, there are several personal and media-related updates. The author is making lifestyle changes, moving from soda to carbonated water and considering cutting back on caffeine due to anxiety. Environmentally, they describe their current location as "like burning up in the desert here." In terms of media, they are enjoying RPGs, mentioning "Paper Mario sounds like a good series," and reflecting on the value of physical media, noting they will miss having copies of films "on something I can watch on my own terms even when offline."

Finally, there are various technical and personal anecdotes. The author is testing and showcasing demos like "Sunny Bluesky over the ocean demo running on Windows" and improving the Bluesky v2.0 visuals. They also address hardware performance, finding that the bottleneck for their system was the Webcam, which performs much better when using scrcpy with a Pixel 9 Pro. The posts conclude with reflections on family time, such as watching a well-animated family movie where "bowser stole the show."
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
summarize the last 200 posts by this actor into 4 paragraphs

## Harness Scope Assessment
requested_scope: count 200
required_total_items: 200
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 325
current_window_offset: 100
current_window_size: 50

## Collection Evidence
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
collection_label: Recent posts by did:plc:nynvpc2sqsiplptgs7uet4cv (items 101-150 of 325)
collection_kind: recent_posts
search_window_offset: 100
search_window_total_items: 50

matched_item[0] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5iopvyjk2z
body: A lot of my demos use place holder images I had generated for me because I have no talent at that kind of art. I am good at coding not using a graphics editor, or drawing really. However I have it in my projects so all you have to do is replace the images in the games folder and I release the code.

matched_item[1] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5gypomjc2q
body: Higher quality.

matched_item[2] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5g4wa5os2y
body: Tunnel of Encoded Light #CreativeCoding #Cplusplus #C++
tag: CreativeCoding
tag: Cplusplus
tag: C++

matched_item[3] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq4qdzz3vs2j
body: Updated to v0.17.0 Full Doxygen documentation.

matched_item[4] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3nizjwak2r
body: Artwork.

matched_item[5] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3nbp7c6s27
body: Some artwork.

matched_item[6] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3mga4fpk2m
body: Variation of the video that looks much better in 4K.

matched_item[7] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3lluinhc2k
body: The compression kind of makes this look dull, but the idea is there.

matched_item[8] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3lcngync2c
body: Code itself is art. #art #videoart #softwareart #generativeart #creativecoding
tag: art
tag: videoart
tag: softwareart
tag: generativeart
tag: creativecoding

matched_item[9] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3kyyzcks2c
body: yeah I was impressed by it, really cool.

matched_item[10] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3gsz5gc22r
body: Hey thanks a lot!

matched_item[11] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq37jtlgnc2j
body: Custom OBJ/MXMOD Model viewer.

matched_item[12] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq33j2f37s2b
body: I was more just mentioning this because I would love to share my apps I create with people like I used to, but most people are locked into those ecosystems, and it can be difficult for most people to compile software or just don't want to.

matched_item[13] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq334oaxi22b
body: but does this let other people run your programs without the requirements for gatekeeper on mac, or the limitations on windows?

matched_item[14] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq32xy5rdc2b
body: My program installed on my system with an icon and running it from the system menu.

matched_item[15] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq32mwwr222e
body: cool

matched_item[16] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq32czuq6s2v
body: Started playing two different RPG styled games to start out with. One classic, one modern both were enjoyable.

matched_item[17] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2zapfv222u
body: or you can use the environment variable 
OPENCV_FORCE_DNN_ENGINE=1

matched_item[18] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2yeeqjck2u
body: Came across an issue with OpenCV 5. The new Deep Neural Net  (DNN) engine does not support CUDA yet, and the CPU fallback was giving me like 2 to 3 FPS, so had to disable the new graph engine until its fully supported. So  I used conditional compilation so it works across OpenCV 4 and 5.

matched_item[19] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2wxwkjxs2u
body: So my I updated my system's distrobox arch container opencv-cuda package to OpenCV 5 and recompiled my apps. I have not really noticed any difference but its supposed to be a big step forward. OpenCV makes it easy to write cross-platform computer vision applications.

matched_item[20] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2w44oy722u
body: Yesterday my friend was showing me some stuff he made with Godot, it looked good. GDScript runs in a VM, so massive loops and stuff will run slower than a compiled language, but for most projects that is not going to disrupt your game. Godot is mostly written in C++.

matched_item[21] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2vpmttuc2u
body: I am actually feeling pretty good today. I woke up around like 6 am had a cup of coffee and then started working on my projects. Thinking about making a flatpak  for one of my games.

matched_item[22] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2uiggamc2u
body: You cannot realistically use ad-hoc signing to distribute your software to the general public.

Gatekeeper Blocks It: If another user downloads your ad-hoc signed app from the internet, macOS Gatekeeper will block it with a severe "developer cannot be verified" or "malware" warning.

matched_item[23] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2tyu5zrc2u
body: problem is how many normal mac users actually know how to do that? If you try to open it from the DMG file it does not work, and you have to open your security settings and explicitly let the app through gatekeeper, but I guess that is a security feature.

matched_item[24] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2tqg57vs2u
body: yeah you have to sign it with - and the let it past gatekeeper right?

matched_item[25] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2qxkcg2s2f
body: So the most viable way to use my programs is to compile from source, and  99% of people won't or can't do that. On windows I had to disable some feature that can only be turned back on by reinstalling Windows, to even test my software. On macOS apple wants to extort you for like $100 a year.

matched_item[26] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2quejw2k2f
body: It really is disappointing that I cannot share my programs with people on Windows/Mac without jumping through all kinds of hoops and getting my app signed. On macOS you have to pay Apple, and I believe it the same for Windows. So much bad software has been developed they locked those systems down.

matched_item[27] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpzqc3gh2c2k
body: I don't know if I shared this here, this is a layout  of the CUDA kernel used in my app ACMX2 for the glitch art effects.

matched_item[28] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpznskube223
body: Source Code: github.com/lostjared/MX...
link: https://github.com/lostjared/MXVK/tree/main/examples/mutatris

matched_item[29] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpznqz7lhk23
body: Updated the game play in my game, check it out. #Vulkan #CPLUSPLUS #C++ #SDL3 #Shaders #Puzzle #PuzzleGame
tag: Vulkan
tag: CPLUSPLUS
tag: C++
tag: SDL3
tag: Shaders
tag: Puzzle
tag: PuzzleGame

matched_item[30] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpznea637c2q
body: updated my game Mutatris, so now all four grids can interact with each and matches can occur across boundaries which creates some interesting game play and chain reactions.

This game is like my dream game and I finally was able to put all the 'pieces together' and it has come full circle.

matched_item[31] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpz4q643pc2w
body: My Sunny Bluesky over the ocean demo running on Windows.
Source Code:

github.com/lostjared/MX...
link: https://github.com/lostjared/MXVK/tree/main/examples/bluesky

matched_item[32] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpywy3eqbs2l
body: I already don't drink soda anymore, only carbonated water, so that is a step in the right direction.

matched_item[33] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpywtqpk2c22
body: I think I am going to cut down on the caffeine or stop it completely. I am getting (is it causing ?) anxiety and I think it makes my shaking worse. Hopefully that will help, and I will adjust.

matched_item[34] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpyvhbde2k2x
body: This how it feels where I live right now, its like burning up in the desert here.

matched_item[35] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpyosrmtl22c
body: Paper Mario sounds like a good series.

matched_item[36] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpyogbn6pc2s
body: Hey thanks for the info I think I can find one that I can vibe with. I will have to explore.

matched_item[37] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpyjd5v7bc2v
body: youtu.be/jemhF-XVojE
link: https://youtu.be/jemhF-XVojE

matched_item[38] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpyj7ogy4s2v
body: Bluesky v2.0 (Updated). The ocean looks better now.

matched_item[39] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxs7tyyek2k
body: Hehe, I was bored so I created this with my new library. It is a nice sunny day overlooking the ocean with a Bluesky and puffy white clouds.

matched_item[40] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxmx5pdzc2s
body: That is the best way to do it, makes it easier to not have to switch the discs but still have backup copy.

matched_item[41] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxiv2r7i227
body: I wonder if the original FF7 is on the Internet archive. It could be.

matched_item[42] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpximvhr3227
body: Whoa that's impressive.

matched_item[43] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxfh5qkrc2c
body: I used to collect DVD/Blu-ray and have a small collection. When they stop making physical discs I will miss being able to have a copy of my favorite films on something I can watch on my own terms even when offline.

matched_item[44] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxdxentks25
body: My nieces are visiting their Grandmother for the summer for a few days. So I ordered this off Amazon and redeemed the digital code. We all watched it in the family room and they loved it. I thought the animation was very well done and bowser stole the show. All around good time with my family.

matched_item[45] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxdehmyos25
body: So you keep them in the box? That's cool might increase in value that way. I wonder if you can still use it in the game it is kept in box?

matched_item[46] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxbkiidqk2v
body: I have never really been interested in RPGs. I did try Mario RPG for SNES and I thought it was a good game. But never played final fantasy or anything of that sort. Was thinking about trying Cyberpunk 2077.

matched_item[47] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpx5f47ku22c
body: Fractals.

matched_item[48] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpx3tiwsos24
body: Well I figured out the bottleneck for my system was more on the Webcam than the application itself. It runs fine when I use scrcpy and my Pixel 9 Pro as the camera. The image quality is also a ton better. I would share a video but on my system I am having trouble recording in 4K so screenshots.

matched_item[49] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpwyvgwdsk2i
body: Capture on 4K Resolution Webcam applying fragment shaders on an older device (RTX 2070) in native 4K resolution.

## Proposed Summary
post: Recent Updates: Creative Coding, Software Distribution, and Personal Life
summary: The recent posts heavily feature updates on creative coding projects, particularly around visual art and game development. The author notes that many demos use placeholder images because they are proficient in coding rather than graphics, stating, "Code itself is art. #art #videoart #softwareart #generativeart #creativecoding." Specific projects mentioned include "Tunnel of Encoded Light" and the game "Mutatris," which has seen significant updates allowing all four grids to interact for "interesting game play and chain reactions." Technical details are shared, such as using Vulkan, C++ with SDL3, and implementing CUDA kernels for glitch art effects in the ACMX2 app.

A major recurring theme is the difficulty of software distribution across operating systems. The author expresses disappointment that sharing programs on Windows and Mac requires "jumping through all kinds of hoops and getting my app signed." This is due to macOS Gatekeeper blocking ad-hoc signed apps, and the general difficulty for users to compile from source. Solutions being explored include creating a Flatpak and noting that the custom OBJ/MXMOD Model viewer allows for easier deployment. The author also discusses performance tuning, noting that OpenCV 5's new DNN engine required conditional compilation due to initial CUDA support issues, and mentions using the environment variable `OPENCV_FORCE_DNN_ENGINE=1`.

Beyond coding, there are several personal and media-related updates. The author is making lifestyle changes, moving from soda to carbonated water and considering cutting back on caffeine due to anxiety. Environmentally, they describe their current location as "like burning up in the desert here." In terms of media, they are enjoying RPGs, mentioning "Paper Mario sounds like a good series," and reflecting on the value of physical media, noting they will miss having copies of films "on something I can watch on my own terms even when offline."

Finally, there are various technical and personal anecdotes. The author is testing and showcasing demos like "Sunny Bluesky over the ocean demo running on Windows" and improving the Bluesky v2.0 visuals. They also address hardware performance, finding that the bottleneck for their system was the Webcam, which performs much better when using scrcpy with a Pixel 9 Pro. The posts conclude with reflections on family time, such as watching a well-animated family movie where "bowser stole the show."
window_offset: 100
window_size: 50
page_index: 2
page_size: 50
collection_total_items: 325
has_more: true
covered_item_1_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5iopvyjk2z
covered_item_2_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5gypomjc2q
covered_item_3_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq5g4wa5os2y
covered_item_4_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq4qdzz3vs2j
covered_item_5_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3nizjwak2r
covered_item_6_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3nbp7c6s27
covered_item_7_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3mga4fpk2m
covered_item_8_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3lluinhc2k
covered_item_9_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3lcngync2c
covered_item_10_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3kyyzcks2c
covered_item_11_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq3gsz5gc22r
covered_item_12_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq37jtlgnc2j
covered_item_13_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq33j2f37s2b
covered_item_14_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq334oaxi22b
covered_item_15_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq32xy5rdc2b
covered_item_16_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq32mwwr222e
covered_item_17_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq32czuq6s2v
covered_item_18_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2zapfv222u
covered_item_19_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2yeeqjck2u
covered_item_20_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2wxwkjxs2u
covered_item_21_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2w44oy722u
covered_item_22_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2vpmttuc2u
covered_item_23_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2uiggamc2u
covered_item_24_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2tyu5zrc2u
covered_item_25_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2tqg57vs2u
covered_item_26_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2qxkcg2s2f
covered_item_27_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mq2quejw2k2f
covered_item_28_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpzqc3gh2c2k
covered_item_29_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpznskube223
covered_item_30_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpznqz7lhk23
covered_item_31_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpznea637c2q
covered_item_32_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpz4q643pc2w
covered_item_33_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpywy3eqbs2l
covered_item_34_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpywtqpk2c22
covered_item_35_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpyvhbde2k2x
covered_item_36_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpyosrmtl22c
covered_item_37_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpyogbn6pc2s
covered_item_38_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpyjd5v7bc2v
covered_item_39_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpyj7ogy4s2v
covered_item_40_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxs7tyyek2k
covered_item_41_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxmx5pdzc2s
covered_item_42_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxiv2r7i227
covered_item_43_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpximvhr3227
covered_item_44_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxfh5qkrc2c
covered_item_45_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxdxentks25
covered_item_46_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxdehmyos25
covered_item_47_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpxbkiidqk2v
covered_item_48_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpx5f47ku22c
covered_item_49_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpx3tiwsos24
covered_item_50_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpwyvgwdsk2i

[summary_sufficiency_gate]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
window_offset: 100
contiguous_coverage: 150
available_total_items: 325
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
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
offset: 100
result_present: false
review_status: fail
review_reason: Grounded summary coverage currently reaches 150 item(s), but 200 item(s) are required before parent synthesis is sufficient.
diagnostic: summary cursor processed offset 100 (page 3 of at most 4)

[collection_summary_loop_page_state]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
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
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
page_index: 3
offset: 150
window_size: 50

[summary_leaf_parse]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
window_offset: 150
result_present: true
original_result_kind: summary
diagnostic: <none>
raw_response:
TOOL_CALL
name: submit_summary_result
args: {
  "title": "Recent Updates: Software Development, Gaming, and Personal Reflections",
  "summary": "The recent posts heavily focus on software development, particularly updates and showcases related to the MXVK project, which utilizes Vulkan. There are multiple mentions of version releases, such as v0.16.0, v0.15.0, v0.14.0, v0.13.0, and v0.12.0, alongside new features like an 'Object Viewer' and 'Better Speed Boost.' Specific projects highlighted include a four-sided falling-block puzzle game named Mutatris, which is built with MXVK, Vulkan, and SDL3, and a 3D Breakout game demo, which has been ported and polished using Vulkan/C++.\n\nBeyond MXVK, the actor shares insights into coding tools and experiences. There is a strong preference for terminal-based tools, with mentions of using 'vim a lot and KDevelop' instead of Visual Studio Code. The actor also discusses the rapid advancement of AI tools, acknowledging the ethical concerns but praising the accessibility of answers. Furthermore, there is a clear theme of software distribution philosophy, advocating that 'All software should be free and come with the source code,' suggesting that physical media should be phased out to eliminate DRM.\n\nPersonal reflections and lifestyle updates are also present. The actor shares feelings of being 'kind of low this morning' but improving after working on projects, and notes physical efforts like using a 'stationary exercise bike to try and strengthen my legs.' In terms of technology performance, the actor notes that with an older graphics card, performance is 'nowhere near as good' without CUDA, especially in 4K.\n\nFinally, there are several miscellaneous updates covering gaming, media, and general thoughts. These include enjoying a game that has a 'modern twist' reminiscent of NES titles, appreciating the wisdom found in a book after reading the second edition, and sharing technical snippets like 'Fireworks in less than 250 lines with MXVK.' The posts also touch upon the importance of knowledge, stating that 'what is really valuable is knowledge,' and the beauty of digital vs. physical media."
}

[summary_leaf_review]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
window_offset: 150
review_status: fail
review_grounded: true
review_sufficient: false
review_repair_needed: false
review_additional_pages_needed: true
review_reason: Grounded summary coverage currently reaches 0 item(s), but 200 item(s) are required before parent synthesis is sufficient.
result_before_review: summary
original_result_before_review: summary
summary_before_review:
The recent posts heavily focus on software development, particularly updates and showcases related to the MXVK project, which utilizes Vulkan. There are multiple mentions of version releases, such as v0.16.0, v0.15.0, v0.14.0, v0.13.0, and v0.12.0, alongside new features like an 'Object Viewer' and 'Better Speed Boost.' Specific projects highlighted include a four-sided falling-block puzzle game named Mutatris, which is built with MXVK, Vulkan, and SDL3, and a 3D Breakout game demo, which has been ported and polished using Vulkan/C++.

Beyond MXVK, the actor shares insights into coding tools and experiences. There is a strong preference for terminal-based tools, with mentions of using 'vim a lot and KDevelop' instead of Visual Studio Code. The actor also discusses the rapid advancement of AI tools, acknowledging the ethical concerns but praising the accessibility of answers. Furthermore, there is a clear theme of software distribution philosophy, advocating that 'All software should be free and come with the source code,' suggesting that physical media should be phased out to eliminate DRM.

Personal reflections and lifestyle updates are also present. The actor shares feelings of being 'kind of low this morning' but improving after working on projects, and notes physical efforts like using a 'stationary exercise bike to try and strengthen my legs.' In terms of technology performance, the actor notes that with an older graphics card, performance is 'nowhere near as good' without CUDA, especially in 4K.

Finally, there are several miscellaneous updates covering gaming, media, and general thoughts. These include enjoying a game that has a 'modern twist' reminiscent of NES titles, appreciating the wisdom found in a book after reading the second edition, and sharing technical snippets like 'Fireworks in less than 250 lines with MXVK.' The posts also touch upon the importance of knowledge, stating that 'what is really valuable is knowledge,' and the beauty of digital vs. physical media.
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
summarize the last 200 posts by this actor into 4 paragraphs

## Harness Scope Assessment
requested_scope: count 200
required_total_items: 200
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 325
current_window_offset: 150
current_window_size: 50

## Collection Evidence
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
collection_label: Recent posts by did:plc:nynvpc2sqsiplptgs7uet4cv (items 151-200 of 325)
collection_kind: recent_posts
search_window_offset: 150
search_window_total_items: 50

matched_item[0] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpwvwjyhik2c
body: With the older graphics card in my computer if I don't use CUDA the performance is nowhere near as good. Especially in 4K. But when it's running in 4K on a 4K monitor it looks nice.

matched_item[1] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpw7m5ylqs23
body: Was feeling kind of low this morning. After resting and working on my projects I am feeling much better. Have a good day.

matched_item[2] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpvzuxke7222
body: This game is starting to get difficult.  Still a good experience that reminds me of the old games for NES but with a modern twist.

matched_item[3] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpvmqrxiyk2d
body: but its alright, I do not need validation, I do code because it is something I love to do, and the process keeps me motivated and engaged in life. Even tho it does produce a lot of solitude.

matched_item[4] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpvmojnxs22d
body: I guess I just must be unpopular or people just are not interested in me and what I do. I spend all my time working on all this software and a lot of the time people just ignore what I post or they don't see it. Honestly, it is a little disappointing because I put all my heart and soul into it.

matched_item[5] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpvmdfmzl22d
body: Released v0.16.0 updated the docs, and also published a zip file with all the different vulkan shaders I use for my different projects.

matched_item[6] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpvivccor22i
body: On YouTube.

matched_item[7] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpviqyueo22i
body: Post processing shaders with MXVK (Vulkan/SDL3).

matched_item[8] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mputdsxois27
body: Mutatris VK - Win X64 port

matched_item[9] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mputcxqvbc27
body: Looks great!

matched_item[10] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpugxeio622d
body: Fireworks in less than 250 lines with MXVK.

github.com/lostjared/MX...
link: https://github.com/lostjared/MXVK/blob/main/examples/fireworks/fireworks.cpp

matched_item[11] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpu4p7x7l22n
body: The country is going through some serious drama. But anyway have a good fourth of July from your buddy Jared. Don't drink too much and be respectful of animals hearing.

matched_item[12] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptypesz7k2t
body: I remember when I tried to read the first edition of this book, I was not ready for it. When I got a hold of the second edition as a hardback It really hit home. At that point I was ready to really listen to the wisdom in this book. I recommend it for anyone who is seriously interested in software.

matched_item[13] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptxzdy7ck2t
body: I like terminal based coding tools the best. It is quite amazing how fast these AI tools have developed. I know a lot of people don't like the ethics involved in them, or the data centers and I totally agree with that.  However it is quite amazing to have most of the answers at your fingertips.

matched_item[14] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptx4lj2u225
body: Ninja is also pretty cool.

matched_item[15] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptvzbdlwk2v
body: I no longer use Visual Studio Code, I use vim a lot and KDevelop.

matched_item[16] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptvxyi6zc2v
body: I can see a lot of issues from people publishing software they had generated for them without knowing how to code at all.  That to me sounds a little dangerous. If nobody is reviewing it, how do you know if its correct or not?

matched_item[17] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptv5atob22v
body: Updated MXVK to v0.15.0 with new Mutatris example project, and other improvements.

matched_item[18] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptqgf5crs2j
body: Running on Apple Silicon #Vulkan #C++ #MoltenVK
#GPU #Graphics #CPlusPlus #HomeBrew
tag: Vulkan
tag: C++
tag: MoltenVK
tag: GPU
tag: Graphics
tag: CPlusPlus
tag: HomeBrew

matched_item[19] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpt7ufkz7s22
body: Example of some of the shader effects for the new version of  Mutatris.

matched_item[20] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpt7ccasks2v
body:  Ported and enhanced my puzzle Game Mutatris to Vulkan and made the game background have animated shader effects. This is such a cool game, i t just it takes a lot of time to play because it can get pretty complex. Good exercise for my mind. Even if nobody else cares or wants to play it. I love it.

matched_item[21] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mps5oucwfk25
body: Started working on this port of my old game.

Mutatris is a four-sided falling-block puzzle game built with MXVK, Vulkan, and SDL3. Three-block pieces enter one side of the playfield at a time, then the active side rotates around the board after a piece locks.

matched_item[22] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprvl4ihu224
body: Feeling tired early today. I spent most of the day working on my projects.

matched_item[23] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprilgxcg22m
body: I use a stationary exercise bike to try and strengthen my legs. I have built muscle but my balance is still not functioning correctly. It's kind of difficult to deal with thinking I might be like this the rest of my life. I am going to keep pushing forward.

matched_item[24] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprhlvjpzk2m
body: Don't get me wrong I find digital versions very convenient. But nothing beats having a real book in your hands.

matched_item[25] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprhefrfes2m
body: I think what is really valuable is knowledge. If it is gated behind a paywall then only the people with access will be able to learn and have the same opportunities. I was lucky enough to be able to acquire a large amount of computer science textbooks to teach myself.

matched_item[26] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprgenloxs2b
body: just my opinion.

matched_item[27] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprgbweyac2b
body: How about if they get rid of physical media, they get rid of the DRM garbage on digital versions and share it with everyone for free and stop capitalizing off stuff that is just strings of data put onto something you should already own. All software should be free and come with the source code.

matched_item[28] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprfmjmjyk2b
body: Dancing Linux Penguin Point Sprites.
#C++ #Vulkan #GPU
tag: C++
tag: Vulkan
tag: GPU

matched_item[29] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpr3dra5zc23
body: Updated to version v0.14.0 with new Object Viewer.

github.com/lostjared/MXVK
lostsidedead.biz/MXVK
link: https://github.com/lostjared/MXVK
link: https://lostsidedead.biz/MXVK

matched_item[30] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpqw7fvr6s2p
body: Made a version of my Model viewer that uses MXVK. It can be useful to inspect the files in the project if you just want to see what they look like real quick. BTW: I love mushrooms, especially in spaghetti.

matched_item[31] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpqcnv3ogs2o
body: Better Speed Boost. (Warp Speed?).

matched_item[32] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpqc6gdog22u
body: lol i was just joking anyway.

matched_item[33] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpq3ggjvws2j
body: It is illogical for people to spend their lives collecting wealth just to realize in the end none of that matters and you cease to exist in this dimension. Ironic and the poetry of life. All pain of this life will no longer exist in the path of time.

matched_item[34] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mppanaa6422k
body: youtu.be/8SvGU0D2Ypc
link: https://youtu.be/8SvGU0D2Ypc

matched_item[35] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpp77t3hnc24
body: Eyes are the window to the soul.

matched_item[36] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpp4rj652c2s
body: Released version v0.13.0
github.com/lostjared/MXVK

and Documentation:
lostsidedead.biz/MXVK
link: https://github.com/lostjared/MXVK
link: https://lostsidedead.biz/MXVK

matched_item[37] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpp3zrjruc2a
body: Nice!

matched_item[38] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpoxuzhqbs2y
body: Blue Terminal for BlueSky

matched_item[39] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpoxqhcxzs2y
body: Warp Speed with Vulkan/C++.
Engage!

matched_item[40] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpoumbj7vc2y
body: Updated the breakout clone and used a texture mapped sphere of the moon as the ball instead of a cube, looks much better now. #Vulkan #C++ #Cplusplus #demo
tag: Vulkan
tag: C++
tag: Cplusplus
tag: demo

matched_item[41] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpogjq5kzs2g
body: Updated MXVK to version v0.12.0 with a lot of new features, bug fixes, and improvements.

lostsidedead.biz/MXVK

github.com/lostjared/MXVK
link: https://lostsidedead.biz/MXVK
link: https://github.com/lostjared/MXVK

matched_item[42] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpof4ohb6k2g
body: My breakout clone.

matched_item[43] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpocwaumac22
body: Screenshots for this little program.

matched_item[44] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpocpm6n7s22
body: Added a custom video recorder to my fractal zoom  demo.

matched_item[45] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpo6ii2f4c2w
body: Fell asleep on my computer last night and did that crap where I wake up and the terminal is full of random characters from my hands going heavy. I need to stop falling asleep at my computer.

matched_item[46] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpnwpl5bq22r
body: Put some polish on my 3D Breakout game demo with Vulkan/C++ (MXVK).

matched_item[47] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpn7cbis7k2j
body: I love these movies. Just got the 4K collectors edition.

matched_item[48] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpmzbce7js2d
body: Ported my 3D Breakout game over to MXVK and improved it.

matched_item[49] uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpmunwc4kk2t
body:  I am trying to be funny

## Proposed Summary
post: Recent Updates: Software Development, Gaming, and Personal Reflections
summary: The recent posts heavily focus on software development, particularly updates and showcases related to the MXVK project, which utilizes Vulkan. There are multiple mentions of version releases, such as v0.16.0, v0.15.0, v0.14.0, v0.13.0, and v0.12.0, alongside new features like an 'Object Viewer' and 'Better Speed Boost.' Specific projects highlighted include a four-sided falling-block puzzle game named Mutatris, which is built with MXVK, Vulkan, and SDL3, and a 3D Breakout game demo, which has been ported and polished using Vulkan/C++.

Beyond MXVK, the actor shares insights into coding tools and experiences. There is a strong preference for terminal-based tools, with mentions of using 'vim a lot and KDevelop' instead of Visual Studio Code. The actor also discusses the rapid advancement of AI tools, acknowledging the ethical concerns but praising the accessibility of answers. Furthermore, there is a clear theme of software distribution philosophy, advocating that 'All software should be free and come with the source code,' suggesting that physical media should be phased out to eliminate DRM.

Personal reflections and lifestyle updates are also present. The actor shares feelings of being 'kind of low this morning' but improving after working on projects, and notes physical efforts like using a 'stationary exercise bike to try and strengthen my legs.' In terms of technology performance, the actor notes that with an older graphics card, performance is 'nowhere near as good' without CUDA, especially in 4K.

Finally, there are several miscellaneous updates covering gaming, media, and general thoughts. These include enjoying a game that has a 'modern twist' reminiscent of NES titles, appreciating the wisdom found in a book after reading the second edition, and sharing technical snippets like 'Fireworks in less than 250 lines with MXVK.' The posts also touch upon the importance of knowledge, stating that 'what is really valuable is knowledge,' and the beauty of digital vs. physical media.
window_offset: 150
window_size: 50
page_index: 3
page_size: 50
collection_total_items: 325
has_more: true
covered_item_1_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpwvwjyhik2c
covered_item_2_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpw7m5ylqs23
covered_item_3_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpvzuxke7222
covered_item_4_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpvmqrxiyk2d
covered_item_5_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpvmojnxs22d
covered_item_6_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpvmdfmzl22d
covered_item_7_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpvivccor22i
covered_item_8_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpviqyueo22i
covered_item_9_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mputdsxois27
covered_item_10_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mputcxqvbc27
covered_item_11_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpugxeio622d
covered_item_12_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpu4p7x7l22n
covered_item_13_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptypesz7k2t
covered_item_14_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptxzdy7ck2t
covered_item_15_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptx4lj2u225
covered_item_16_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptvzbdlwk2v
covered_item_17_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptvxyi6zc2v
covered_item_18_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptv5atob22v
covered_item_19_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mptqgf5crs2j
covered_item_20_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpt7ufkz7s22
covered_item_21_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpt7ccasks2v
covered_item_22_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mps5oucwfk25
covered_item_23_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprvl4ihu224
covered_item_24_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprilgxcg22m
covered_item_25_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprhlvjpzk2m
covered_item_26_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprhefrfes2m
covered_item_27_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprgenloxs2b
covered_item_28_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprgbweyac2b
covered_item_29_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mprfmjmjyk2b
covered_item_30_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpr3dra5zc23
covered_item_31_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpqw7fvr6s2p
covered_item_32_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpqcnv3ogs2o
covered_item_33_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpqc6gdog22u
covered_item_34_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpq3ggjvws2j
covered_item_35_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mppanaa6422k
covered_item_36_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpp77t3hnc24
covered_item_37_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpp4rj652c2s
covered_item_38_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpp3zrjruc2a
covered_item_39_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpoxuzhqbs2y
covered_item_40_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpoxqhcxzs2y
covered_item_41_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpoumbj7vc2y
covered_item_42_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpogjq5kzs2g
covered_item_43_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpof4ohb6k2g
covered_item_44_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpocwaumac22
covered_item_45_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpocpm6n7s22
covered_item_46_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpo6ii2f4c2w
covered_item_47_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpnwpl5bq22r
covered_item_48_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpn7cbis7k2j
covered_item_49_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpmzbce7js2d
covered_item_50_uri: at://did:plc:nynvpc2sqsiplptgs7uet4cv/app.bsky.feed.post/3mpmunwc4kk2t

[summary_sufficiency_gate]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
window_offset: 150
contiguous_coverage: 200
available_total_items: 325
status_after_gate: pass
review_grounded: true
review_sufficient: true
review_additional_pages_needed: false
review_next_offset: <none>
result_before_restore: false
original_result_before_restore: true
result_after_gate: true
original_result_after_gate: true

[collection_summary_loop]
node: summarize_page
status: page_outcome
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
offset: 150
result_present: true
review_status: pass
review_reason: Grounded summary coverage reaches 200 item(s), satisfying the requested 200 item scope.
diagnostic: summary cursor processed offset 150 (page 4 of at most 4)

[collection_summary_loop_page_state]
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
page_offset: 150
page_index: 3
page_status: accepted_summary
repair_attempted: false
fallback_applied: false
next_offset: <none>
accumulated_covered_count: 200
failure_reason: <none>

[collection_summary_loop]
node: collection_summary_planner_review
status: planner_optional_failure
reason: response appears truncated
route: collection_summary_notes

[execute_public_summary]
status: loop_finished
outcome_count: 1
rendered:
tool_name: collection_summary
collection_id: recent_posts:did:plc:nynvpc2sqsiplptgs7uet4cv
collection_label: Recent posts by did:plc:nynvpc2sqsiplptgs7uet4cv
status: ok
diagnostic: collection_summary_planner accepted 4 page summaries and 0 raw-window fallbacks; final_notes_summary_accepted: false; planner_summary_accepted: false
covered_window_offsets: 0, 50, 100, 150
covered_post_count: 200
planner_updates: 2
coherent_pages: 4
review_status: pass
review_grounded: true
review_sufficient: true
review_reason: No final planner or notes synthesis was accepted after considering 200 posts.
review_repair_needed: false
review_additional_pages_needed: false
review_required_total_items: 200
post: Summary of Recent posts by did:plc:nynvpc2sqsiplptgs7uet4cv
summary: The recent posts heavily feature the author's deep involvement in creative coding and game development, often utilizing C++ and GLSL. There is a strong focus on visual art, including 'Video Art of some Liquid Thought' and 'Color Wormhole,' alongside various graphical styles like 'glitchart' and 'spiral.' Several projects are highlighted, such as 'Asteroids-Net,' a four-player multiplayer game, and 'Mutatris,' a 'Glitch Art inspired Puzzle Game.' The author also shares technical updates, like improving a front end for FFmpeg to create videos from images, and providing resources like 'Windows X64 builds of MXVK demos.'

Beyond the visual and coding projects, the author discusses their hardware and technical interests. They express a desire for a 'Steam Machine' to code games and detail their setup, which includes using a PC to control a mini PC as a NAS and HTPC. The technical prowess is evident in discussions about AI, noting that modern tools can process 'tens f thousands of lines, in a very short amount of time and find real bugs,' and the use of 'llama.cpp.'

On a personal level, the author shares significant life updates. They discuss their physical health, noting that while they may never regain their previous gait, they are improving with a walker, and they are intensely exercising their legs on a stationary bike. They also reflect on their social life, stating they have a 'hard time making friends' due to experiences like a coma, and that their interests are 'extremely rare.'

Finally, there are several brief personal anecdotes and reflections. The author mentions enjoying specific foods, such as 'pistachios and walnuts,' and reflects on the passing of their cat, 'Coder.' They also touch upon their dedication, stating, 'I am incredibly dedicated, mastering complex graphics programming over decades,' and share various links to their work, including a series of 'C++ examples' and web-based games.

The recent posts heavily focus on the author's ongoing software development and passion projects, particularly in the realm of graphics and game programming. A major theme revolves around the custom graphics engine, MXVK, which is described as the "Vulkan evolution of the MX2 engine, redesigned entirely from scratch utilizing Vulkan 1.4 and SDL3." The author details technical aspects, noting that the engine supports "synchronization2 and dynamicRendering," and mentions its inspiration from DXVK. Projects include a "3D Tetris" game and a "3D Pong," with demos available online.

Technical deep dives cover system performance and implementation details. One significant thread discusses VRAM usage in GNOME Shell, observing that "whenever a Vulkan window opens the gnome-shell process VRAM usage grows," even for blank windows. The author also shares insights into their programming philosophy, such as the pseudo-code modeling karma: "karma=get(give());" and the belief that "A part of being human is making mistakes. The purpose is whether or not you learn from them."

Beyond the code, there are numerous updates on creative output and learning. The author shares artwork, including pieces that contain source code, and mentions writing a poem about programming from when they were "around the age of 18." They also highlight the impressive capabilities of modern AI, noting that GPT 5.6 "created this page in one prompt and it worked the first time" when converting a game to JavaScript/WebGL2.

Finally, the posts touch upon broader life and technical interests. These include preferences for Linux environments, such as liking "KDE plasma big screen" paired with handheld devices, and observations on societal issues, such as blaming "congressional Republicans for letting this happen" regarding a declining trend. The author expresses satisfaction with their progress, noting, "I have been having a good summer this year working on a software project to practice and learn about GPU/game programming."

The recent posts heavily feature updates on creative coding projects, particularly around visual art and game development. The author notes that many demos use placeholder images because they are proficient in coding rather than graphics, stating, "Code itself is art. #art #videoart #softwareart #generativeart #creativecoding." Specific projects mentioned include "Tunnel of Encoded Light" and the game "Mutatris," which has seen significant updates allowing all four grids to interact for "interesting game play and chain reactions." Technical details are shared, such as using Vulkan, C++ with SDL3, and implementing CUDA kernels for glitch art effects in the ACMX2 app.

A major recurring theme is the difficulty of software distribution across operating systems. The author expresses disappointment that sharing programs on Windows and Mac requires "jumping through all kinds of hoops and getting my app signed." This is due to macOS Gatekeeper blocking ad-hoc signed apps, and the general difficulty for users to compile from source. Solutions being explored include creating a Flatpak and noting that the custom OBJ/MXMOD Model viewer allows for easier deployment. The author also discusses performance tuning, noting that OpenCV 5's new DNN engine required conditional compilation due to initial CUDA support issues, and mentions using the environment variable `OPENCV_FORCE_DNN_ENGINE=1`.

Beyond coding, there are several personal and media-related updates. The author is making lifestyle changes, moving from soda to carbonated water and considering cutting back on caffeine due to anxiety. Environmentally, they describe their current location as "like burning up in the desert here." In terms of media, they are enjoying RPGs, mentioning "Paper Mario sounds like a good series," and reflecting on the value of physical media, noting they will miss having copies of films "on something I can watch on my own terms even when offline."

Finally, there are various technical and personal anecdotes. The author is testing and showcasing demos like "Sunny Bluesky over the ocean demo running on Windows" and improving the Bluesky v2.0 visuals. They also address hardware performance, finding that the bottleneck for their system was the Webcam, which performs much better when using scrcpy with a Pixel 9 Pro. The posts conclude with reflections on family time, such as watching a well-animated family movie where "bowser stole the show."

The recent posts heavily focus on software development, particularly updates and showcases related to the MXVK project, which utilizes Vulkan. There are multiple mentions of version releases, such as v0.16.0, v0.15.0, v0.14.0, v0.13.0, and v0.12.0, alongside new features like an 'Object Viewer' and 'Better Speed Boost.' Specific projects highlighted include a four-sided falling-block puzzle game named Mutatris, which is built with MXVK, Vulkan, and SDL3, and a 3D Breakout game demo, which has been ported and polished using Vulkan/C++.

Beyond MXVK, the actor shares insights into coding tools and experiences. There is a strong preference for terminal-based tools, with mentions of using 'vim a lot and KDevelop' instead of Visual Studio Code. The actor also discusses the rapid advancement of AI tools, acknowledging the ethical concerns but praising the accessibility of answers. Furthermore, there is a clear theme of software distribution philosophy, advocating that 'All software should be free and come with the source code,' suggesting that physical media should be phased out to eliminate DRM.

Personal reflections and lifestyle updates are also present. The actor shares feelings of being 'kind of low this morning' but improving after working on projects, and notes physical efforts like using a 'stationary exercise bike to try and strengthen my legs.' In terms of technology performance, the actor notes that with an older graphics card, performance is 'nowhere near as good' without CUDA, especially in 4K.

Finally, there are several miscellaneous updates covering gaming, media, and general thoughts. These include enjoying a game that has a 'modern twist' reminiscent of NES titles, appreciating the wisdom found in a book after reading the second edition, and sharing technical snippets like 'Fireworks in less than 250 lines with MXVK.' The posts also touch upon the importance of knowledge, stating that 'what is really valuable is knowledge,' and the beauty of digital vs. physical media.
window_offset: 0
window_size: 200
page_index: 0
page_size: 50
collection_total_items: 325
has_more: false
source_exhausted: false
concatenated_window_summaries:
The recent posts heavily feature the author's deep involvement in creative coding and game development, often utilizing C++ and GLSL. There is a strong focus on visual art, including 'Video Art of some Liquid Thought' and 'Color Wormhole,' alongside various graphical styles like 'glitchart' and 'spiral.' Several projects are highlighted, such as 'Asteroids-Net,' a four-player multiplayer game, and 'Mutatris,' a 'Glitch Art inspired Puzzle Game.' The author also shares technical updates, like improving a front end for FFmpeg to create videos from images, and providing resources like 'Windows X64 builds of MXVK demos.'

Beyond the visual and coding projects, the author discusses their hardware and technical interests. They express a desire for a 'Steam Machine' to code games and detail their setup, which includes using a PC to control a mini PC as a NAS and HTPC. The technical prowess is evident in discussions about AI, noting that modern tools can process 'tens f thousands of lines, in a very short amount of time and find real bugs,' and the use of 'llama.cpp.'

On a personal level, the author shares significant life updates. They discuss their physical health, noting that while they may never regain their previous gait, they are improving with a walker, and they are intensely exercising their legs on a stationary bike. They also reflect on their social life, stating they have a 'hard time making friends' due to experiences like a coma, and that their interests are 'extremely rare.'

Finally, there are several brief personal anecdotes and reflections. The author mentions enjoying specific foods, such as 'pistachios and walnuts,' and reflects on the passing of their cat, 'Coder.' They also touch upon their dedication, stating, 'I am incredibly dedicated, mastering complex graphics programming over decades,' and share various links to their work, including a series of 'C++ examples' and web-based games.

The recent posts heavily focus on the author's ongoing software development and passion projects, particularly in the realm of graphics and game programming. A major theme revolves around the custom graphics engine, MXVK, which is described as the "Vulkan evolution of the MX2 engine, redesigned entirely from scratch utilizing Vulkan 1.4 and SDL3." The author details technical aspects, noting that the engine supports "synchronization2 and dynamicRendering," and mentions its inspiration from DXVK. Projects include a "3D Tetris" game and a "3D Pong," with demos available online.

Technical deep dives cover system performance and implementation details. One significant thread discusses VRAM usage in GNOME Shell, observing that "whenever a Vulkan window opens the gnome-shell process VRAM usage grows," even for blank windows. The author also shares insights into their programming philosophy, such as the pseudo-code modeling karma: "karma=get(give());" and the belief that "A part of being human is making mistakes. The purpose is whether or not you learn from them."

Beyond the code, there are numerous updates on creative output and learning. The author shares artwork, including pieces that contain source code, and mentions writing a poem about programming from when they were "around the age of 18." They also highlight the impressive capabilities of modern AI, noting that GPT 5.6 "created this page in one prompt and it worked the first time" when converting a game to JavaScript/WebGL2.

Finally, the posts touch upon broader life and technical interests. These include preferences for Linux environments, such as liking "KDE plasma big screen" paired with handheld devices, and observations on societal issues, such as blaming "congressional Republicans for letting this happen" regarding a declining trend. The author expresses satisfaction with their progress, noting, "I have been having a good summer this year working on a software project to practice and learn about GPU/game programming."

The recent posts heavily feature updates on creative coding projects, particularly around visual art and game development. The author notes that many demos use placeholder images because they are proficient in coding rather than graphics, stating, "Code itself is art. #art #videoart #softwareart #generativeart #creativecoding." Specific projects mentioned include "Tunnel of Encoded Light" and the game "Mutatris," which has seen significant updates allowing all four grids to interact for "interesting game play and chain reactions." Technical details are shared, such as using Vulkan, C++ with SDL3, and implementing CUDA kernels for glitch art effects in the ACMX2 app.

A major recurring theme is the difficulty of software distribution across operating systems. The author expresses disappointment that sharing programs on Windows and Mac requires "jumping through all kinds of hoops and getting my app signed." This is due to macOS Gatekeeper blocking ad-hoc signed apps, and the general difficulty for users to compile from source. Solutions being explored include creating a Flatpak and noting that the custom OBJ/MXMOD Model viewer allows for easier deployment. The author also discusses performance tuning, noting that OpenCV 5's new DNN engine required conditional compilation due to initial CUDA support issues, and mentions using the environment variable `OPENCV_FORCE_DNN_ENGINE=1`.

Beyond coding, there are several personal and media-related updates. The author is making lifestyle changes, moving from soda to carbonated water and considering cutting back on caffeine due to anxiety. Environmentally, they describe their current location as "like burning up in the desert here." In terms of media, they are enjoying RPGs, mentioning "Paper Mario sounds like a good series," and reflecting on the value of physical media, noting they will miss having copies of films "on something I can watch on my own terms even when offline."

Finally, there are various technical and personal anecdotes. The author is testing and showcasing demos like "Sunny Bluesky over the ocean demo running on Windows" and improving the Bluesky v2.0 visuals. They also address hardware performance, finding that the bottleneck for their system was the Webcam, which performs much better when using scrcpy with a Pixel 9 Pro. The posts conclude with reflections on family time, such as watching a well-animated family movie where "bowser stole the show."

The recent posts heavily focus on software development, particularly updates and showcases related to the MXVK project, which utilizes Vulkan. There are multiple mentions of version releases, such as v0.16.0, v0.15.0, v0.14.0, v0.13.0, and v0.12.0, alongside new features like an 'Object Viewer' and 'Better Speed Boost.' Specific projects highlighted include a four-sided falling-block puzzle game named Mutatris, which is built with MXVK, Vulkan, and SDL3, and a 3D Breakout game demo, which has been ported and polished using Vulkan/C++.

Beyond MXVK, the actor shares insights into coding tools and experiences. There is a strong preference for terminal-based tools, with mentions of using 'vim a lot and KDevelop' instead of Visual Studio Code. The actor also discusses the rapid advancement of AI tools, acknowledging the ethical concerns but praising the accessibility of answers. Furthermore, there is a clear theme of software distribution philosophy, advocating that 'All software should be free and come with the source code,' suggesting that physical media should be phased out to eliminate DRM.

Personal reflections and lifestyle updates are also present. The actor shares feelings of being 'kind of low this morning' but improving after working on projects, and notes physical efforts like using a 'stationary exercise bike to try and strengthen my legs.' In terms of technology performance, the actor notes that with an older graphics card, performance is 'nowhere near as good' without CUDA, especially in 4K.

Finally, there are several miscellaneous updates covering gaming, media, and general thoughts. These include enjoying a game that has a 'modern twist' reminiscent of NES titles, appreciating the wisdom found in a book after reading the second edition, and sharing technical snippets like 'Fireworks in less than 250 lines with MXVK.' The posts also touch upon the importance of knowledge, stating that 'what is really valuable is knowledge,' and the beauty of digital vs. physical media.

