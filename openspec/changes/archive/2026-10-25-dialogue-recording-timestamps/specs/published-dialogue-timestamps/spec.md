## ADDED Requirements

### Requirement: Recognize original dialogue timestamps

The renderer SHALL recognize a plain or bold/italic plain speaker prefix, a middle dot, an M:SS or H:MM:SS token, and a dialogue dash at the beginning of an original non-heading outline entry. It SHALL preserve the speaker, timestamp label, speech, formatting and nesting. Trailing time components SHALL contain two digits in the range 00–59, and the absolute seconds SHALL fit a JavaScript safe integer. Dates, prose times, written responses, code, authored link labels and copied transclusion/backlink content SHALL remain literal.

#### Scenario: Speaker and time forms
- **WHEN** original dialogue uses a plain or emphasized speaker with zero, M:SS or H:MM:SS
- **THEN** valid timestamps retain their labels and produce the corresponding absolute seconds

#### Scenario: Literal and invalid content
- **WHEN** content is a date, prose time, written response, code, heading, link label or malformed/out-of-range time
- **THEN** automatic dialogue controls are absent and the source text remains readable

#### Scenario: Copied dialogue
- **WHEN** the same dialogue appears through a transclusion or backlink copy
- **THEN** only the original dialogue can supply automatic controls

### Requirement: Associate one supported original recording

Automatic dialogue controls SHALL bind only when the document renders exactly one distinct supported original audio source through existing media/asset resolution. Repeated original uses of that source SHALL share its identity. Copied audio SHALL not participate. The source may occur after the dialogue. Controls SHALL use ordinary recording URLs with integer #t fragments, preserving URL queries and source audio options. Source files SHALL remain unchanged.

#### Scenario: Later original recording and repeated download
- **WHEN** a page's unique supported recording occurs after its dialogue and is also linked as a download
- **THEN** dialogue controls target that original recording at the authored absolute times

#### Scenario: Ambiguous or copied-only recordings
- **WHEN** a page has no original recording, several distinct originals, or audio only in copied content
- **THEN** implicit dialogue times stay literal

### Requirement: Coordinate accessible inline playback

Only pages with rendered dialogue controls SHALL load the shared controller. It SHALL support native anchor keyboard activation, preserve modified-click navigation, install idempotently and perform no playback during setup. On activation it SHALL offer a bubbling cancelable lsg:media-seek event carrying source, seconds and anchor before inline fallback. Cancellation SHALL leave playback entirely to the host. Fallback SHALL use the associated existing original audio, retain the latest pending metadata seek, clamp against known duration, and handle failed playback without creating another player. Unhandled controls SHALL retain ordinary media-fragment navigation.

#### Scenario: Host-owned playback
- **WHEN** a host cancels the media-seek event
- **THEN** the helper prevents the original handled navigation and performs no inline seek or play

#### Scenario: Delayed metadata and duration bounds
- **WHEN** a reader activates several timestamps before metadata arrives
- **THEN** the latest requested seek applies when possible, bounded by known duration, while playback starts only from activation

#### Scenario: Native fallback and repeat setup
- **WHEN** scripts are absent, playback is unavailable, activation is modified, or the helper initializes repeatedly
- **THEN** ordinary links/native media controls remain available and duplicate players or activation listeners are absent
