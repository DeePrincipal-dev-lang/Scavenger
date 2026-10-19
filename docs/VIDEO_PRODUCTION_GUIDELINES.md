# Video Production Guidelines

Standards for creating Scavngr platform tutorial videos.

## Recording Setup

- **Resolution**: 1920×1080 (1080p) minimum; 2560×1440 preferred
- **Frame rate**: 30 fps
- **Audio**: External USB microphone; quiet room; -12 dBFS nominal level
- **Browser zoom**: 125% in Chrome/Firefox for readability
- **Screen capture**: OBS Studio (free, cross-platform)

## Script Structure

Each video must follow this structure:

```
1. Hook (0:00–0:20)
   - One sentence: what you'll learn
   - Why it matters

2. Prerequisites (0:20–0:45)
   - List what viewers need before starting

3. Walkthrough (bulk of the video)
   - Step-by-step, no skipping steps
   - Narrate every action ("I'm clicking the Submit button...")

4. Summary (last 30 sec)
   - Recap what was covered
   - Link to next video in the series
```

## Visual Standards

- Use the production app at https://app.scavngr.io for all recordings
- Testnet wallet with pre-funded test accounts (see `demo/demo-accounts.json.example`)
- Highlight clicks using a screen annotation tool (e.g., Epic Pen)
- Blur or crop out any personal wallet addresses
- Show terminal in a dark theme (VS Code Dark+ or Dracula)

## Narration

- Speak at a measured pace — aim for ~130 words/minute
- Use plain language; avoid jargon without explanation
- Spell out acronyms on first use ("XLM — the Stellar Lumens native token")
- Record narration in one continuous take per section to simplify editing

## Editing

- Trim dead air longer than 2 seconds
- Add zoom-in cuts for small UI elements
- Chapters: add YouTube chapter markers matching the script sections
- Add intro/outro cards (provided in `docs/videos/assets/`)
- Export: H.264, AAC audio, MP4 container

## Captions and Transcripts

- Export auto-captions from YouTube, then manually correct errors
- Upload corrected `.srt` file back to YouTube
- Place the plain-text transcript alongside the video embed in the docs

## Review Process

1. Author records and self-reviews the video.
2. Author opens a PR adding the transcript and metadata YAML to `docs/videos/`.
3. A maintainer reviews for accuracy and clarity.
4. On approval, the video is made public on YouTube and the docs PR is merged.

## Updating Existing Videos

If a UI change makes a video outdated:

1. Create a new recording using the update sections only.
2. Add a note to the old video description: "Updated version: <link>"
3. Update the docs embed to point to the new video.
4. Archive the old video (set to unlisted, do not delete — old links may still circulate).
