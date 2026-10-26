# Parity performance

- measured: 2026-10-26
- garden commit: 0ce74d6
- gh-pages commit: 8132c95
- lsg: 0.0.0 (b490d68)
- profiles: desktop, phone
- loads per page, logseq: 1
- loads per page, lsg: 3

Medians of the page loads. **Bold** marks an `lsg` value over its budget in [`scripts/parity/budgets.toml`](../../scripts/parity/budgets.toml).

## Page loads

| Page | Profile | Build | Title visible (ms) | First contentful paint (ms) | Data before title (KB, gzip) | Script before title (KB) | Peak JS heap (MB) |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| tmux | desktop | logseq | 21,952 | 22,000 | 22,643 | 8,087 | 2,013 |
| tmux | desktop | lsg | 35.9 | 48.0 | **101** | 14.6 | 2.0 |
| tmux | phone | logseq | 204,376 | 204,568 | 22,067 | 8,087 | 1,989 |
| tmux | phone | lsg | 234 | 200 | **101** | 14.6 | 1.9 |
| Foam | desktop | logseq | 23,614 | 23,628 | 22,643 | 8,087 | 1,988 |
| Foam | desktop | lsg | 21.6 | 32.0 | 13.3 | 14.6 | 1.8 |
| Foam | phone | logseq | 196,789 | 196,820 | 22,067 | 8,087 | 1,975 |
| Foam | phone | lsg | 244 | 208 | 13.3 | 14.6 | 1.9 |
| Logseq/Block/Id | desktop | logseq | 17,716 | 17,736 | 22,643 | 8,087 | 1,976 |
| Logseq/Block/Id | desktop | lsg | 22.9 | 32.0 | 11.8 | 14.6 | 1.8 |
| Logseq/Block/Id | phone | logseq | 184,889 | 184,916 | 22,067 | 8,087 | 1,949 |
| Logseq/Block/Id | phone | lsg | 239 | 248 | 11.8 | 14.6 | 1.8 |
| Logseq/Queries | desktop | logseq | 19,760 | 19,772 | 22,643 | 8,087 | 1,969 |
| Logseq/Queries | desktop | lsg | 21.8 | 28.0 | 11.5 | 14.6 | 1.8 |
| Logseq/Queries | phone | logseq | 185,344 | 185,372 | 22,067 | 8,087 | 1,974 |
| Logseq/Queries | phone | lsg | 243 | 244 | 11.5 | 14.6 | 1.8 |
| Lazygit | desktop | logseq | 23,806 | 23,824 | 22,643 | 8,087 | 1,997 |
| Lazygit | desktop | lsg | 27.2 | 40.0 | 30.9 | 14.6 | 2.1 |
| Lazygit | phone | logseq | 194,248 | 194,292 | 22,067 | 8,087 | 1,974 |
| Lazygit | phone | lsg | 226 | 240 | 30.9 | 14.6 | 2.2 |
| rw-cli | desktop | logseq | 21,022 | 21,036 | 22,958 | 9,096 | 2,008 |
| rw-cli | desktop | lsg | 27.7 | 40.0 | 16.0 | 14.6 | 1.8 |
| rw-cli | phone | logseq | 192,159 | 192,204 | 22,067 | 8,087 | 1,983 |
| rw-cli | phone | lsg | 252 | 212 | 16.0 | 14.6 | 1.9 |
| ff | desktop | logseq | 17,395 | 17,412 | 22,958 | 9,096 | 2,021 |
| ff | desktop | lsg | 27.9 | 40.0 | 14.5 | 14.6 | 1.8 |
| ff | phone | logseq | 196,794 | 196,844 | 22,067 | 8,087 | 1,981 |
| ff | phone | lsg | 238 | 200 | 14.5 | 14.6 | 1.9 |
| Contrando | desktop | logseq | 22,494 | 22,504 | 22,643 | 8,087 | 2,007 |
| Contrando | desktop | lsg | 22.0 | 28.0 | 10.7 | 14.6 | 1.8 |
| Contrando | phone | logseq | 191,461 | 191,484 | 22,067 | 8,087 | 1,959 |
| Contrando | phone | lsg | 253 | 212 | 10.7 | 14.6 | 1.8 |
| AI/E/WS/25/d1 | desktop | logseq | 17,595 | 17,612 | 22,643 | 8,087 | 2,002 |
| AI/E/WS/25/d1 | desktop | lsg | 25.4 | 36.0 | 12.3 | 14.6 | 1.8 |
| AI/E/WS/25/d1 | phone | logseq | 190,372 | 190,412 | 22,067 | 8,087 | 1,961 |
| AI/E/WS/25/d1 | phone | lsg | 240 | 204 | 12.3 | 14.6 | 1.8 |
| GitP/A/Session/24/11/19-Tue | desktop | logseq | 18,885 | 18,904 | 22,643 | 8,087 | 1,989 |
| GitP/A/Session/24/11/19-Tue | desktop | lsg | 26.7 | 36.0 | 12.7 | 14.6 | 1.8 |
| GitP/A/Session/24/11/19-Tue | phone | logseq | 186,104 | 186,160 | 22,067 | 8,087 | 1,978 |
| GitP/A/Session/24/11/19-Tue | phone | lsg | 252 | 256 | 12.7 | 14.6 | 1.9 |
| 2026-10-23 Fri | desktop | logseq | 22,266 | 22,280 | 22,643 | 8,087 | 1,972 |
| 2026-10-23 Fri | desktop | lsg | 21.6 | 32.0 | 11.8 | 14.6 | 1.8 |
| 2026-10-23 Fri | phone | logseq | 188,766 | 188,796 | 22,067 | 8,087 | 1,948 |
| 2026-10-23 Fri | phone | lsg | 234 | 200 | 11.8 | 14.6 | 1.8 |
| AI/Coding/History | desktop | logseq | 19,211 | 19,264 | 22,643 | 8,087 | 1,982 |
| AI/Coding/History | desktop | lsg | 23.7 | 36.0 | 15.4 | 14.6 | 1.8 |
| AI/Coding/History | phone | logseq | 188,363 | 188,568 | 22,067 | 8,087 | 1,960 |
| AI/Coding/History | phone | lsg | 231 | 240 | 15.4 | 14.6 | 1.8 |
| VSCode/Proj/24t3/LearnVSCode/03 Basic Python | desktop | logseq | 23,196 | 23,208 | 23,062 | 9,096 | 2,028 |
| VSCode/Proj/24t3/LearnVSCode/03 Basic Python | desktop | lsg | 35.3 | 48.0 | 32.1 | 14.6 | 2.0 |
| VSCode/Proj/24t3/LearnVSCode/03 Basic Python | phone | logseq | 188,662 | 188,684 | 22,172 | 8,087 | 1,948 |
| VSCode/Proj/24t3/LearnVSCode/03 Basic Python | phone | lsg | 235 | 248 | 32.1 | 14.6 | 1.8 |
| Home | desktop | logseq | 20,937 | 20,952 | 22,643 | 8,087 | 1,991 |
| Home | desktop | lsg | 21.0 | 32.0 | 12.3 | 14.6 | 1.7 |
| Home | phone | logseq | 204,698 | 204,736 | 22,067 | 8,087 | 1,838 |
| Home | phone | lsg | 242 | 256 | 12.3 | 14.6 | 1.8 |
| All pages | desktop | logseq | 19,820 | 19,828 | 22,643 | 8,087 | 2,019 |
| All pages | desktop | lsg | 85.6 | 112 | **271** | 14.6 | 2.4 |
| All pages | phone | logseq | 197,866 | 197,876 | 22,067 | 8,087 | 1,982 |
| All pages | phone | lsg | 233 | 244 | **271** | 14.6 | 1.7 |

## Journals feed after 10 later batches

The home page scrolled until it shows 73 journals or runs out. The phone heap budget applies to `lsg`.

| Profile | Build | Journals shown | Peak JS heap (MB) | Time to load the batches (ms) |
| --- | --- | ---: | ---: | ---: |
| desktop | logseq | 3 | 1,998.3 | 30,046 |
| desktop | lsg | 73 | 4.6 | 331 |
| phone | logseq | 3 | 1,988.8 | 30,045 |
| phone | lsg | 73 | 3.1 | 3,601 |

## Whole-garden build time

`lsg publish` with the run's options, once without and once with `--journals`.

| Build | Seconds |
| --- | ---: |
| without journals | 5.6 |
| with journals | 4.7 |

## Block zoom on tmux

Time from clicking the bullet of a nested block halfway down the page to the zoomed view.

| Profile | Build | Zoom (ms) |
| --- | --- | ---: |
| desktop | logseq | 77 |
| desktop | lsg | 20 |
| phone | logseq | 612 |
| phone | lsg | 23 |

## Search latency per keystroke

| Profile | Build | Keystrokes | Unchanged | Median (ms) | 95th percentile (ms) |
| --- | --- | ---: | ---: | ---: | ---: |
| desktop | logseq | 36 | 0 | 776 | 1836 |
| desktop | lsg | TimeoutError: Page.wait_for_selector: Timeout 30000ms exceeded. | | |
| phone | logseq | 36 | 0 | 2950 | 13675 |
| phone | lsg | TimeoutError: Page.wait_for_selector: Timeout 30000ms exceeded. | | |

## Search first open and cached reopening

| Profile | Build | First ready (ms) | Transferred body (KB) | Decoded (KB) | Heap growth (MB) | Eager requests | Warm ready (ms) | Warm requests |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| desktop | logseq | 95 | 0.0 | 0.0 | 0.0 | 0 | 0 | – |
| phone | logseq | 204 | 0.0 | 0.0 | 2.8 | 0 | 0 | – |
