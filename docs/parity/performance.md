# Parity performance

- measured: 2026-10-25
- garden commit: 0ce74d6
- gh-pages commit: 5fb53fb
- lsg: 0.0.0 (1cf4a53)
- profiles: desktop, phone
- loads per page, logseq: 1
- loads per page, lsg: 3

Medians of the page loads. **Bold** marks an `lsg` value over its budget in [`scripts/parity/budgets.toml`](../../scripts/parity/budgets.toml).

## Page loads

| Page | Profile | Build | Title visible (ms) | First contentful paint (ms) | Data before title (KB, gzip) | Script before title (KB) | Peak JS heap (MB) |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| tmux | desktop | logseq | 23,823 | 23,872 | 22,630 | 8,087 | 2,010 |
| tmux | desktop | lsg | 37.8 | 48.0 | 98.5 | 14.6 | 2.0 |
| tmux | phone | logseq | 185,321 | 185,488 | 22,054 | 8,087 | 1,967 |
| tmux | phone | lsg | 237 | 248 | 98.5 | 14.6 | 1.8 |
| Foam | desktop | logseq | 20,472 | 20,484 | 22,630 | 8,087 | 1,984 |
| Foam | desktop | lsg | 20.8 | 32.0 | 12.1 | 14.6 | 1.8 |
| Foam | phone | logseq | 185,481 | 185,508 | 22,054 | 8,087 | 1,823 |
| Foam | phone | lsg | 219 | 228 | 12.1 | 14.6 | 1.9 |
| Logseq/Block/Id | desktop | logseq | 19,393 | 19,404 | 22,630 | 8,087 | 1,976 |
| Logseq/Block/Id | desktop | lsg | 21.7 | 32.0 | 10.8 | 14.6 | 1.8 |
| Logseq/Block/Id | phone | logseq | 184,100 | 184,128 | 22,054 | 8,087 | 1,947 |
| Logseq/Block/Id | phone | lsg | 243 | 256 | 10.8 | 14.6 | 1.8 |
| Logseq/Queries | desktop | logseq | 20,907 | 20,916 | 22,630 | 8,087 | 1,992 |
| Logseq/Queries | desktop | lsg | 21.8 | 28.0 | 10.4 | 14.6 | 1.8 |
| Logseq/Queries | phone | logseq | 184,349 | 184,372 | 22,054 | 8,087 | 1,951 |
| Logseq/Queries | phone | lsg | 233 | 240 | 10.4 | 14.6 | 1.8 |
| Lazygit | desktop | logseq | 22,294 | 22,312 | 22,630 | 8,087 | 1,990 |
| Lazygit | desktop | lsg | 25.3 | 36.0 | 29.9 | 14.6 | 2.1 |
| Lazygit | phone | logseq | 187,650 | 187,692 | 22,054 | 8,087 | 1,950 |
| Lazygit | phone | lsg | 242 | 260 | 29.9 | 14.6 | 2.2 |
| rw-cli | desktop | logseq | 20,537 | 20,552 | 22,945 | 9,097 | 2,008 |
| rw-cli | desktop | lsg | 28.7 | 40.0 | 14.9 | 14.6 | 1.8 |
| rw-cli | phone | logseq | 188,745 | 188,780 | 22,054 | 8,087 | 1,953 |
| rw-cli | phone | lsg | 245 | 256 | 14.9 | 14.6 | 1.8 |
| ff | desktop | logseq | 20,522 | 20,540 | 22,945 | 9,097 | 2,021 |
| ff | desktop | lsg | 24.7 | 36.0 | 13.3 | 14.6 | 1.8 |
| ff | phone | logseq | 188,398 | 188,440 | 22,054 | 8,087 | 1,942 |
| ff | phone | lsg | 237 | 252 | 13.3 | 14.6 | 1.9 |
| Contrando | desktop | logseq | 21,009 | 21,020 | 22,630 | 8,087 | 1,994 |
| Contrando | desktop | lsg | 21.3 | 28.0 | 9.6 | 14.6 | 1.8 |
| Contrando | phone | logseq | 187,978 | 188,000 | 22,054 | 8,087 | 1,956 |
| Contrando | phone | lsg | 236 | 244 | 9.6 | 14.6 | 1.8 |
| AI/E/WS/25/d1 | desktop | logseq | 20,952 | 20,968 | 22,630 | 8,087 | 2,002 |
| AI/E/WS/25/d1 | desktop | lsg | 23.6 | 32.0 | 11.2 | 14.6 | 1.8 |
| AI/E/WS/25/d1 | phone | logseq | 187,451 | 187,488 | 22,054 | 8,087 | 1,962 |
| AI/E/WS/25/d1 | phone | lsg | 246 | 260 | 11.2 | 14.6 | 1.8 |
| GitP/A/Session/24/11/19-Tue | desktop | logseq | 21,680 | 21,696 | 22,630 | 8,087 | 1,995 |
| GitP/A/Session/24/11/19-Tue | desktop | lsg | 28.7 | 40.0 | 11.7 | 14.6 | 1.8 |
| GitP/A/Session/24/11/19-Tue | phone | logseq | 188,200 | 188,260 | 22,054 | 8,087 | 1,965 |
| GitP/A/Session/24/11/19-Tue | phone | lsg | 245 | 252 | 11.7 | 14.6 | 1.9 |
| 2026-10-23 Fri | desktop | logseq | 22,222 | 22,236 | 22,630 | 8,087 | 1,971 |
| 2026-10-23 Fri | desktop | lsg | 21.2 | 32.0 | 10.8 | 14.6 | 1.8 |
| 2026-10-23 Fri | phone | logseq | 185,603 | 185,636 | 22,054 | 8,087 | 1,947 |
| 2026-10-23 Fri | phone | lsg | 234 | 248 | 10.8 | 14.6 | 1.8 |
| Home | desktop | logseq | 21,985 | 22,000 | 22,630 | 8,087 | 1,990 |
| Home | desktop | lsg | 20.9 | 32.0 | 11.3 | 14.6 | 1.7 |
| Home | phone | logseq | 188,906 | 188,944 | 22,054 | 8,087 | 1,974 |
| Home | phone | lsg | 231 | 244 | 11.3 | 14.6 | 1.8 |
| All pages | desktop | logseq | 23,357 | 23,364 | 22,630 | 8,087 | 2,019 |
| All pages | desktop | lsg | 85.1 | 108 | **270** | 14.6 | 2.4 |
| All pages | phone | logseq | 190,168 | 190,180 | 22,054 | 8,087 | 1,975 |
| All pages | phone | lsg | 224 | 232 | **270** | 14.6 | 1.7 |

## Journals feed after 10 later batches

The home page scrolled until it shows 73 journals or runs out. The phone heap budget applies to `lsg`.

| Profile | Build | Journals shown | Peak JS heap (MB) | Time to load the batches (ms) |
| --- | --- | ---: | ---: | ---: |
| desktop | logseq | 3 | 1,994.5 | 30,025 |
| desktop | lsg | 73 | 3.0 | 464 |
| phone | logseq | 3 | 1,981.2 | 30,042 |
| phone | lsg | 73 | 3.1 | 3,540 |

## Whole-garden build time

`lsg publish` with the run's options, once without and once with `--journals`.

| Build | Seconds |
| --- | ---: |
| without journals | 4.6 |
| with journals | 4.1 |

## Block zoom on tmux

Time from clicking the bullet of a nested block halfway down the page to the zoomed view.

| Profile | Build | Zoom (ms) |
| --- | --- | ---: |
| desktop | logseq | 92 |
| desktop | lsg | 18 |
| phone | logseq | 280 |
| phone | lsg | 22 |

## Search latency per keystroke

| Profile | Build | Keystrokes | Unchanged | Median (ms) | 95th percentile (ms) |
| --- | --- | ---: | ---: | ---: | ---: |
| desktop | logseq | 36 | 0 | 785 | 1838 |
| desktop | lsg | TimeoutError: Page.wait_for_selector: Timeout 30000ms exceeded. | | |
| phone | logseq | 36 | 0 | 2918 | 13252 |
| phone | lsg | TimeoutError: Page.wait_for_selector: Timeout 30000ms exceeded. | | |

## Search first open and cached reopening

| Profile | Build | First ready (ms) | Transferred body (KB) | Decoded (KB) | Heap growth (MB) | Eager requests | Warm ready (ms) | Warm requests |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| desktop | logseq | 85 | 0.0 | 0.0 | 0.0 | 0 | 0 | – |
| phone | logseq | 207 | 0.0 | 0.0 | 2.8 | 0 | 0 | – |
