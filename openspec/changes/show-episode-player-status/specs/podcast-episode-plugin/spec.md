## ADDED Requirements

### Requirement: Show the recording link and playback status

The player SHALL show a `Full recording` link to the URL of the recording it holds, opening in a new tab. Its polite live status SHALL read `Playing`, `Paused` or `Ended` as the recording starts playing, pauses or ends, and SHALL report a load or seek failure. The current chapter in the player's trail SHALL carry `aria-current="true"`, and no other trail link SHALL carry it. The player's `<audio>` element SHALL have the accessible name `Full episode recording` while it is in the player.

#### Scenario: Full recording link

- **WHEN** the player holds the recording of `GitP/A/Session/26/10/24-Sat`
- **THEN** its `Full recording` link points to the URL of `GitP___A___Session___26___10___24-Sat___Asset___Audio___Full.mp3` and opens in a new tab

#### Scenario: Another episode takes over

- **WHEN** the reader starts the recording of `GitP/A/Session/26/10/29-Thu` from its Play button while the player holds `GitP/A/Session/26/10/24-Sat`
- **THEN** the `Full recording` link points to the `29-Thu` recording

#### Scenario: Playback status

- **WHEN** a reader starts the recording, pauses it, and later lets it play to its end
- **THEN** the status reads `Playing`, then `Paused`, then `Ended`

#### Scenario: Recording fails to load

- **WHEN** the recording's request fails
- **THEN** the status says the recording could not load

#### Scenario: Current chapter in the trail

- **WHEN** a reader selects `05:15` in `05:15 - the bass comes in` on `GitP/A/Session/26/11/01-Sun`
- **THEN** the trail's link to that bullet carries `aria-current="true"` and the link to its enclosing chapter `Infini Pad as bass` does not

#### Scenario: Accessible name of the recording

- **WHEN** the player holds a recording
- **THEN** its `<audio>` element has the accessible name `Full episode recording`, and when the module is cleaned up the element returns to its page with the name the page gave it
