## Context

`DateFormat` in the SDK's `journals` module reads `:journal/page-title-format` and `:journal/file-name-format`. It supports `yyyy`, `MM`, `M`, `MMM`, `MMMM`, `dd`, `d`, `do`, `EEE` and `EEEE`. Any other letter run is an unsupported token, and garden settings then fall back to `MMM do, yyyy` with a diagnostic.

Logseq formats and reads journal titles with its fork of cljs-time, pinned at `5704fbf` in Logseq's [`deps.edn`](https://github.com/logseq/og/blob/6e7afa8eb040686ff057156ee877193b581dd369/deps.edn#L14-L15). In that library:

- Writing maps `E` and `EEE` to the short weekday name and `EEEE` to the full name ([`unparse.cljs`](https://github.com/logseq/cljs-time/blob/5704fbf48d3478eedcf24d458c8964b3c2fd59a9/src/cljs_time/internal/unparse.cljs#L168-L171)). `EE` and five or more `E`s have no entry, so writing a title with them fails.
- Reading maps the same three patterns to one weekday reader, which tries every full name and then every short name, whatever the pattern's width ([`parse.cljs`](https://github.com/logseq/cljs-time/blob/5704fbf48d3478eedcf24d458c8964b3c2fd59a9/src/cljs_time/internal/parse.cljs#L194-L216)). Logseq capitalizes each word of the title before reading it ([`date_time_util.cljs`](https://github.com/logseq/og/blob/6e7afa8eb040686ff057156ee877193b581dd369/deps/graph-parser/src/logseq/graph_parser/date_time_util.cljs#L20-L31)), so case does not matter.
- Reading ignores the weekday when it builds the date ([`parse.cljs`](https://github.com/logseq/cljs-time/blob/5704fbf48d3478eedcf24d458c8964b3c2fd59a9/src/cljs_time/internal/parse.cljs#L328-L358)).

Logseq's settings screen offers `E, dd-MM-yyyy`, `E, dd.MM.yyyy`, `E, MM/dd/yyyy` and `E, yyyy/MM/dd` among its title formats ([`date.cljs`](https://github.com/logseq/og/blob/6e7afa8eb040686ff057156ee877193b581dd369/src/main/frontend/date.cljs#L22-L52), [`settings.cljs`](https://github.com/logseq/og/blob/6e7afa8eb040686ff057156ee877193b581dd369/src/main/frontend/components/settings.cljs#L422)).

## Goals / Non-Goals

**Goals:**

- Write and read journal titles with `E`, `EEE` and `EEEE` as Logseq does.

**Non-Goals:**

- Read month names of either length, as cljs-time also does for `MMM` and `MMMM`.

## Decisions

`DateFormat::parse` accepts a run of one, three or four `E`s as a weekday token. One or three letters write the short name; four write the full name. A run of two, or of five or more, stays an unsupported token.

`DateFormat::read` matches a weekday token against the full names first and then the short names, without regard to case. Trying full names first keeps `Friday` from stopping after `Fri`.

## Risks / Trade-offs

[A contradicting weekday] → The SDK keeps rejecting a date whose weekday does not match it, as the journal-identity capability already requires. Logseq ignores the weekday instead. The difference affects only links whose weekday is wrong; see Resolved Question 3.

## Resolved Questions

### 1 - Does `E` write the short or the full weekday name?

Decided by the agent without codekiln: the short name, `Fri`. cljs-time maps `E` and `EEE` to the same short-name writer, and Logseq uses cljs-time for every journal title.

### 2 - Should a weekday token read both name lengths?

Decided by the agent without codekiln: yes. cljs-time's reader accepts `Friday` and `Fri` for all three patterns, so Logseq treats `[[Friday, 10/23/2026]]` as the journal titled `Fri, 10/23/2026`. Accepting both keeps the SDK's journal links equal to Logseq's.

### 3 - Should the SDK stop rejecting a contradicting weekday?

Decided by the agent without codekiln: no, not in this change. The journal-identity capability requires the rejection, and its scenario `[[2026-10-23 Mon]]` stays unresolved. Logseq would read that link as 2026-10-23 but would still keep it as its own page, separate from the journal titled `2026-10-23 Fri`, so the visible difference is small. Changing an existing requirement is outside Issue #269.

### 4 - Should `EE` and `EEEEE` be accepted as aliases?

Decided by the agent without codekiln: no. cljs-time has no pattern for them, so Logseq cannot write a title in such a format. Keeping them unsupported gives the author a diagnostic.

## Open Questions

None.
