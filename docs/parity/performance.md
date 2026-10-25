# Parity performance

- measured: 2026-10-25
- garden commit: 0ce74d6
- gh-pages commit: 8bc3391
- lsg: 0.0.0 (b1d51bc)
- profiles: desktop, phone
- loads per page, logseq: 1
- loads per page, lsg: 3

Medians of the page loads. **Bold** marks an `lsg` value over its budget in [`scripts/parity/budgets.toml`](../../scripts/parity/budgets.toml).

## Page loads

| Page | Profile | Build | Title visible (ms) | First contentful paint (ms) | Data before title (KB, gzip) | Script before title (KB) | Peak JS heap (MB) |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| tmux | desktop | logseq | 17,874 | 17,928 | 18,703 | 8,087 | 2,575 |
| tmux | desktop | lsg | 36.0 | 48.0 | 96.0 | 14.0 | 2.0 |
| tmux | phone | logseq | 149,825 | 150,016 | 18,127 | 8,087 | 2,547 |
| tmux | phone | lsg | 233 | 244 | 96.0 | 14.0 | 1.8 |
| Foam | desktop | logseq | 11,930 | 11,944 | 18,703 | 8,087 | 2,559 |
| Foam | desktop | lsg | 21.3 | 32.0 | 10.6 | 14.0 | 1.8 |
| Foam | phone | logseq | 152,732 | 152,772 | 18,127 | 8,087 | 2,529 |
| Foam | phone | lsg | 227 | 240 | 10.6 | 14.0 | 1.9 |
| Logseq/Block/Id | desktop | logseq | 16,856 | 16,868 | 18,703 | 8,087 | 2,579 |
| Logseq/Block/Id | desktop | lsg | 22.2 | 32.0 | 9.5 | 14.0 | 1.8 |
| Logseq/Block/Id | phone | logseq | 158,010 | 158,040 | 18,127 | 8,087 | 1,551 |
| Logseq/Block/Id | phone | lsg | 243 | 256 | 9.5 | 14.0 | 1.8 |
| Logseq/Queries | desktop | logseq | 14,603 | 14,612 | 18,703 | 8,087 | 2,550 |
| Logseq/Queries | desktop | lsg | 21.6 | 28.0 | 9.1 | 14.0 | 1.8 |
| Logseq/Queries | phone | logseq | 141,518 | 141,544 | 18,127 | 8,087 | 1,557 |
| Logseq/Queries | phone | lsg | 238 | 244 | 9.1 | 14.0 | 1.8 |
| Lazygit | desktop | logseq | 14,947 | 14,964 | 18,703 | 8,087 | 2,552 |
| Lazygit | desktop | lsg | 23.5 | 36.0 | 28.2 | 14.0 | 2.1 |
| Lazygit | phone | logseq | 151,910 | 151,956 | 18,127 | 8,087 | 2,544 |
| Lazygit | phone | lsg | 243 | 260 | 28.2 | 14.0 | 2.2 |
| rw-cli | desktop | logseq | 14,513 | 14,528 | 19,018 | 9,097 | 2,594 |
| rw-cli | desktop | lsg | 26.6 | 36.0 | 13.5 | 14.0 | 1.8 |
| rw-cli | phone | logseq | 144,966 | 145,004 | 18,127 | 8,087 | 2,538 |
| rw-cli | phone | lsg | 242 | 256 | 13.5 | 14.0 | 1.8 |
| ff | desktop | logseq | 15,110 | 15,132 | 19,018 | 9,097 | 2,605 |
| ff | desktop | lsg | 23.7 | 32.0 | 11.9 | 14.0 | 1.8 |
| ff | phone | logseq | 145,240 | 145,280 | 18,127 | 8,087 | 2,543 |
| ff | phone | lsg | 247 | 264 | 11.9 | 14.0 | 1.9 |
| Contrando | desktop | logseq | 14,370 | 14,380 | 18,703 | 8,087 | 2,568 |
| Contrando | desktop | lsg | 20.8 | 28.0 | 8.3 | 14.0 | 1.8 |
| Contrando | phone | logseq | 144,054 | 144,076 | 18,127 | 8,087 | 1,554 |
| Contrando | phone | lsg | 233 | 244 | 8.3 | 14.0 | 1.8 |
| AI/E/WS/25/d1 | desktop | logseq | 14,917 | 14,932 | 18,703 | 8,087 | 2,590 |
| AI/E/WS/25/d1 | desktop | lsg | 24.7 | 36.0 | 9.9 | 14.0 | 1.8 |
| AI/E/WS/25/d1 | phone | logseq | 146,089 | 146,128 | 18,127 | 8,087 | 2,537 |
| AI/E/WS/25/d1 | phone | lsg | 238 | 252 | 9.9 | 14.0 | 1.8 |
| GitP/A/Session/24/11/19-Tue | desktop | logseq | 15,109 | 15,128 | 18,703 | 8,087 | 2,561 |
| GitP/A/Session/24/11/19-Tue | desktop | lsg | 28.5 | 40.0 | 10.3 | 14.0 | 1.8 |
| GitP/A/Session/24/11/19-Tue | phone | logseq | 145,248 | 145,304 | 18,127 | 8,087 | 1,558 |
| GitP/A/Session/24/11/19-Tue | phone | lsg | 250 | 264 | 10.3 | 14.0 | 1.8 |
| 2026-10-23 Fri | desktop | logseq | 14,760 | 14,772 | 18,703 | 8,087 | 2,561 |
| 2026-10-23 Fri | desktop | lsg | 19.8 | 28.0 | 9.4 | 14.0 | 1.8 |
| 2026-10-23 Fri | phone | logseq | 143,096 | 143,128 | 18,127 | 8,087 | 1,560 |
| 2026-10-23 Fri | phone | lsg | 234 | 248 | 9.4 | 14.0 | 1.8 |
| Home | desktop | logseq | 14,015 | 14,028 | 18,703 | 8,087 | 2,576 |
| Home | desktop | lsg | 21.1 | 32.0 | 9.8 | 14.0 | 1.7 |
| Home | phone | logseq | 142,881 | 142,912 | 18,127 | 8,087 | 2,544 |
| Home | phone | lsg | 239 | 252 | 9.8 | 14.0 | 1.8 |
| All pages | desktop | logseq | 15,595 | 15,600 | 18,703 | 8,087 | 2,592 |
| All pages | desktop | lsg | 90.8 | 116 | **269** | 14.0 | 2.5 |
| All pages | phone | logseq | 145,273 | 145,284 | 18,127 | 8,087 | 2,558 |
| All pages | phone | lsg | 231 | 240 | **269** | 14.0 | 2.5 |

## Journals feed after 10 later batches

The home page scrolled until it shows 73 journals or runs out. The phone heap budget applies to `lsg`.

| Profile | Build | Journals shown | Peak JS heap (MB) | Time to load the batches (ms) |
| --- | --- | ---: | ---: | ---: |
| desktop | logseq | 10 | 2,586.9 | 30,255 |
| desktop | lsg | 73 | 3.3 | 259 |
| phone | logseq | 3 | 1,586.1 | 30,454 |
| phone | lsg | 73 | 3.0 | 3,228 |

## Whole-garden build time

`lsg publish` with the run's options, once without and once with `--journals`.

| Build | Seconds |
| --- | ---: |
| without journals | 4.9 |
| with journals | 4.1 |

## Block zoom on tmux

Time from clicking the bullet of a nested block halfway down the page to the zoomed view.

| Profile | Build | Zoom (ms) |
| --- | --- | ---: |
| desktop | logseq | 165 |
| desktop | lsg | 16 |
| phone | logseq | 258 |
| phone | lsg | 25 |

Measured on 2026-10-25 with one load per build and profile (`parity:perf --fixtures` naming `tmux`, run `20261025T065337Z`); the other sections are from the last full run.

## Search latency per keystroke

| Profile | Build | Keystrokes | Median (ms) | 95th percentile (ms) |
| --- | --- | ---: | ---: | ---: |
| desktop | logseq | 36 | 652 | 1516 |
| desktop | lsg | no search | | |
| phone | logseq | 36 | 2262 | 11280 |
| phone | lsg | no search | | |
