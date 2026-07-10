# Task: Fixed 5-Row Stdout Chat Footer Via Terminal Scroll Region

## Summary

The current stdout chat renderer paints its input and status UI directly into normal terminal flow.

That keeps the app on the main screen, which is good for native scrollback and text selection, but it also means every transcript append pushes the previously painted footer upward into scrollback. Repainting the input area repeatedly leaves visible ghost bands near the bottom of the terminal even when copied text is mostly clean.

This task defines a terminal-control refactor for `src/ui/stdout_chat.rs` that keeps stdout chat on the main screen while reserving a fixed 5-row footer using terminal scroll-region control.

The target behavior is:

- transcript output scrolls only in the upper region
- the footer stays visually pinned to the bottom of the main terminal screen
- normal terminal scrollback and selection remain usable for transcript text above the footer

## Problem

### Current failure mode

Today the stdout chat renderer:

- writes transcript output into normal terminal flow
- redraws the input box and status UI in that same flow
- clears and repaints the bottom editor area after each update

That causes three visible problems:

1. New transcript output pushes previously rendered footer rows into scrollback.
2. Repeated input repaints leave visible footer ghosts or stacked input bands.
3. The bottom UI is not truly anchored, so it behaves like content instead of a fixed footer.

### What must not change

The solution should preserve the current strengths of stdout chat:

- stay on the main screen rather than using the alternate screen
- preserve ordinary terminal scrollback
- preserve terminal text selection behavior as much as possible
- avoid introducing ratatui-style full-screen ownership into stdout chat

## Target Terminal Model

Stdout chat should remain on the main terminal screen and reserve the bottom 5 rows as a fixed footer.

Use terminal top/bottom margins so only rows above the footer scroll.

Target layout:

- rows `1..scroll_bottom` form the transcript scroll region
- bottom 5 rows are reserved for footer rendering
- transcript output is emitted only inside the scroll region
- footer output is rendered only by absolute cursor positioning into the reserved rows

Compute:

- `footer_height = 5`
- `scroll_bottom = terminal_height - footer_height`

The renderer must not write transcript lines into rows below `scroll_bottom`.

## Fixed Footer Layout

The footer height is fixed at 5 rows for v1.

Footer rows from top to bottom:

- row 1: blank margin row
- rows 2-4: light-gray input box with black text
- row 5: plain status bar with one-character left and right padding and no background fill

Important layout constraints:

- the blank top footer row is intentional and is the only vertical margin
- the input area must never grow beyond 3 rows
- the status row has no background fill

### Input overflow behavior

If the composed input wraps beyond the 3 visible input rows:

- show only the last visible wrapped rows
- keep the cursor on the visible final row
- clip older wrapped rows above the visible window
- do not grow the footer beyond 5 rows

This overflow behavior must be deterministic so repeated repaints do not cause cursor drift or layout instability.

## Renderer Behavior

### Entering stdout chat mode

On entry to stdout chat mode, the renderer should:

1. query terminal dimensions
2. compute `scroll_bottom = height - 5`
3. install a scroll region covering rows `1..scroll_bottom`
4. place the cursor in the footer area for input rendering
5. render the initial footer using absolute positioning

### Transcript updates

During transcript streaming or append events:

- write transcript output only inside the configured scroll region
- allow the terminal to scroll only within that region
- never clear or repaint upward from the footer into transcript rows
- preserve append-only transcript behavior

The renderer should treat transcript rendering and footer rendering as separate responsibilities.

### Footer updates

During input/status updates:

- redraw only the 5 reserved footer rows
- use absolute cursor positioning for each footer row
- clear each footer row before repainting it
- never emit footer content into the scroll region

Footer repainting must fully clear shorter content so old characters do not remain visible after deletions or width changes.

### Leaving stdout chat mode

When switching back to TUI or tearing down the stdout renderer:

- reset the scroll region to the full screen
- clear footer rows as needed
- restore normal cursor behavior
- leave the terminal without stuck margins or misplaced cursor state

## Resize Handling

Resize must be handled as part of normal sync/render behavior rather than as a one-time setup.

On each sync or render pass, the renderer should:

- detect whether terminal dimensions changed
- recompute footer row positions
- recompute `scroll_bottom`
- reinstall the scroll region for the new height
- redraw the footer for the new width
- continue transcript rendering without rewriting old transcript content into the footer

Important resize guarantees:

- shrinking shorter or wider must still produce a valid 5-row footer
- growing taller must restore the correct scroll region immediately
- no stale margin settings should survive after a resize

## Status Row Content

The footer status row should use this content model:

- left: `<model name> · context dd% used · dddK window`
- right: `press tab to toggle modes`

Formatting requirements:

- one leading space and one trailing space inside the row
- no background fill on the status row
- when space is insufficient, preserve deterministic truncation behavior rather than wrapping

This keeps the status row single-line and stable across repaints.

## Compatibility And Safety Notes

### Terminal capability assumptions

The implementation should:

- rely only on terminal capabilities already available through `crossterm`
- avoid enabling mouse capture in stdout chat mode
- tolerate terminals that support cursor movement and clearing but may vary slightly in margin behavior

### Fallback when scroll-region control is unavailable

If scroll-region setup fails or appears unsupported, do not attempt a styled multi-row pinned footer.

Fallback behavior should be:

- remain on the main screen
- disable the styled 5-row footer
- fall back to a simpler single-line prompt/status renderer

The fallback must favor correctness and terminal recovery over visual fidelity.

## Suggested Code Scope

Primary implementation scope:

- `src/ui/stdout_chat.rs`

Assumptions:

- no ratatui migration is involved
- a new dedicated scroll-region abstraction or helper is acceptable if it keeps terminal-control logic isolated from transcript formatting
- transcript formatting and transcript compaction logic can remain conceptually separate from low-level terminal region control

## Acceptance Criteria

### Transcript streaming

- while tool or agent events stream, the footer stays visually fixed at the bottom
- no repeated input-box bands enter visible scrollback

### Text entry after completion

- typing after the root run completes leaves no visual trail
- deleting text clears previously occupied footer cells completely
- wrapped input remains clipped to the last visible 3 input rows

### Selection behavior

- the user can start selection near the bottom and drag upward through transcript text
- the terminal can scroll upward during selection as normal
- the footer does not interfere with selecting transcript text above it

### Resize behavior

- resizing shorter, taller, wider, or narrower preserves a valid transcript region and footer
- no stuck scroll margins remain after resize
- the cursor remains in a valid location after footer redraw

### Mode switching

- switching between stdout chat and TUI restores the full terminal state correctly
- leaving stdout mode removes scroll-region constraints
- no leftover footer artifacts remain after teardown

## Recommended Implementation Shape

1. Add a small renderer-owned abstraction for terminal dimensions, footer geometry, and scroll-region install/reset.
2. Split transcript writes from footer writes so transcript output never reuses footer repaint paths.
3. Rework editor rendering to produce exactly 5 footer rows, clipping wrapped input to the last 3 visible input rows.
4. Add resize-aware sync logic that reinstalls margins and redraws only the footer when dimensions change.
5. Add a capability-aware fallback path for terminals where scroll-region control cannot be relied on.
