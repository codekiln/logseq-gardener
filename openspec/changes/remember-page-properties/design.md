## Context

Properties use native details and a build-time configured open state. The shared fragment helper opens necessary details, and hosts replace main content without reexecuting embedded scripts.

## Goals / Non-Goals

Goals: site-wide reader choice, configured first view, sliding expiry, early restoration and reusable host initialization.

Non-goals: none.

## Decisions

Use one localStorage entry containing a boolean open choice and last-seen time. Scope its key to the directory of the generated properties asset, resolved from a head link. This keeps readable nested host routes in the same site while separating gardens on different base paths. Expire entries after 30 days without visits; invalid or future timestamps also fall back to the configured default.

A small synchronous head reader validates the entry; the controller renews it after paint and on each host visit. An inline call inside each generated Properties disclosure applies the choice before its contents are parsed. Native activation while the module is loading is queued and saved when the controller arrives. The full controller loads after paint and provides idempotent `logseqGardenerInitProperties(root=document)` for replaced content. A host calls this after adopting its URL and before fragment reveal. Repeated initialization leaves already initialized disclosures alone. Native user activation writes the choice; programmatic fragment opening leaves the remembered preference unchanged.

Both initializers apply the latest accepted pending reader choice before consulting older storage. The activation queue remains available until its deferred writes finish, including during controller adoption. Canceled events and programmatic openings do not supply preferences.

## Risks / Trade-offs

Storage failures → keep the configured initial state and a working native disclosure. Reader choices can still apply within the current document.

Dynamic controller load latency → early restoration remains available from the small head reader; native controls remain usable while it loads.

## Resolved Questions

### 1 - Which default applies without a remembered choice?

The graph's existing explicit collapse setting remains authoritative, including its existing default when absent.

### 2 - Does fragment reveal change the saved preference?

Fragment reveal opens the target for navigation. Only a reader's disclosure activation changes the saved site preference.

## Open Questions

None.
