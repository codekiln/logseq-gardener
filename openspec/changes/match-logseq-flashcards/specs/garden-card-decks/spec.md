## ADDED Requirements

### Requirement: Select cards in the SDK

The SDK SHALL return as cards the blocks of selected visible documents whose references include the page `card`, in any case and written as `#card`, `#[[card]]`, `[[card]]` or `[[Card]]`, ignoring code blocks and inline code. Each card SHALL carry its block, page, ancestors and children, and its schedule from `card-repeats`, `card-next-schedule`, `card-last-interval`, `card-ease-factor`, `card-last-reviewed` and `card-last-score`. Withheld sources SHALL contribute no cards.

#### Scenario: Encode garden cards
- **WHEN** the SDK selects cards over the encode garden at commit `083106d`
- **THEN** it returns 432 cards and none from code

#### Scenario: Capitalised reference
- **WHEN** a block on `tmux/Keyshort/Copy Mode` contains `[[Card]]`
- **THEN** that block is a card

### Requirement: Select decks with the shared query evaluator

The SDK SHALL list the argument of every `{{cards}}` macro in the selected documents as a deck query, treating an empty argument as `All` and a bare name as a page reference. It SHALL select a deck's cards by evaluating `(and <deck query> [[card]])` with the SDK query evaluator, and `All` as every card.

#### Scenario: Path references
- **WHEN** the SDK selects the deck `[[yazi]]` over the encode garden at commit `083106d`
- **THEN** it holds 18 cards, including cards under a parent block tagged `[[yazi]]`

#### Scenario: Combined deck
- **WHEN** the SDK selects `(and [[tmux]] (not [[oh-my-tmux]]))` over the same garden
- **THEN** it holds 71 cards

#### Scenario: Deck list
- **WHEN** the SDK lists the encode garden's deck queries
- **THEN** it returns 29 queries

### Requirement: Order and count due cards for a given time

Given a time, the SDK SHALL treat a card as due when it has no `card-repeats` or fewer than 1, no `card-next-schedule`, or a `card-next-schedule` before that time. It SHALL order a deck's due cards by the `card-next-schedule` text with unscheduled cards first, keeping cards with equal text in page-title then outline order, and SHALL give each deck's due and total counts.

#### Scenario: Counts on a date
- **WHEN** the SDK counts decks over the encode garden at commit `083106d` for 2026-10-27T04:33Z
- **THEN** `All` is `430/432`, `[[VSCode]]` is `71/73`, `(and [[VSCode/Keyshort/Git]] )` is `9/10`, `[[yazi]]` is `18/18`, and the first due card of `All` is the first unscheduled card in page-title order, on `1Password/Quick Access`

### Requirement: Hide the scheduling properties everywhere

The SDK SHALL treat `card-last-interval`, `card-repeats`, `card-ease-factor`, `card-next-schedule`, `card-last-reviewed` and `card-last-score` as hidden built-in properties, so no page, reference, embed, search record or review shows them, whatever other block properties are shown.

#### Scenario: Whole-garden build
- **WHEN** the encode garden is published
- **THEN** no published HTML shows any of the six `card-*` keys as visible text
