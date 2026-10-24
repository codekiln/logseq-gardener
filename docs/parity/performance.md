# Parity performance

- measured: 2026-10-24
- garden commit: 0ce74d6
- gh-pages commit: 1ec008c
- lsg: 0.0.0 (58b35fb with the app shell change)
- profiles: desktop, phone
- loads per page, logseq: 1
- loads per page, lsg: 3

Medians of the page loads. **Bold** marks an `lsg` value over its budget in [`scripts/parity/budgets.toml`](../../scripts/parity/budgets.toml).

## Page loads

| Page | Profile | Build | Title visible (ms) | First contentful paint (ms) | Data before title (KB, gzip) | Script before title (KB) | Peak JS heap (MB) |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| tmux | desktop | logseq | 14,936 | 14,980 | 18,679 | 8,087 | 2,577 |
| tmux | desktop | lsg | 34.7 | 48.0 | 89.2 | 13.5 | 2.3 |
| tmux | phone | logseq | 146,691 | 146,864 | 18,069 | 7,956 | 1,551 |
| tmux | phone | lsg | 236 | 248 | 89.2 | 13.5 | 1.8 |
| Foam | desktop | logseq | 13,696 | 13,708 | 18,679 | 8,087 | 2,561 |
| Foam | desktop | lsg | 20.4 | 28.0 | 10.2 | 13.5 | 1.8 |
| Foam | phone | logseq | 145,959 | 145,988 | 18,069 | 7,956 | 1,538 |
| Foam | phone | lsg | 232 | 244 | 10.2 | 13.5 | 1.9 |
| Logseq/Block/Id | desktop | logseq | 13,641 | 13,652 | 18,679 | 8,087 | 2,560 |
| Logseq/Block/Id | desktop | lsg | 19.7 | 28.0 | 9.0 | 13.5 | 1.8 |
| Logseq/Block/Id | phone | logseq | 139,713 | 139,740 | 18,069 | 7,956 | 1,558 |
| Logseq/Block/Id | phone | lsg | 244 | 256 | 9.0 | 13.5 | 1.8 |
| Logseq/Queries | desktop | logseq | 14,947 | 14,960 | 18,679 | 8,087 | 2,547 |
| Logseq/Queries | desktop | lsg | 16.7 | 24.0 | 8.2 | 13.5 | 1.8 |
| Logseq/Queries | phone | logseq | 150,014 | 150,040 | 18,069 | 7,956 | 2,537 |
| Logseq/Queries | phone | lsg | 242 | 252 | 8.2 | 13.5 | 1.8 |
| Lazygit | desktop | logseq | 14,163 | 14,180 | 18,679 | 8,087 | 2,549 |
| Lazygit | desktop | lsg | 23.5 | 36.0 | 25.6 | 13.5 | 2.1 |
| Lazygit | phone | logseq | 144,348 | 144,392 | 18,069 | 7,956 | 2,539 |
| Lazygit | phone | lsg | 238 | 252 | 25.6 | 13.5 | 2.1 |
| rw-cli | desktop | logseq | 14,371 | 14,384 | 18,994 | 9,096 | 2,592 |
| rw-cli | desktop | lsg | 23.7 | 32.0 | 12.9 | 13.5 | 1.8 |
| rw-cli | phone | logseq | 146,250 | 146,288 | 18,069 | 7,956 | 1,531 |
| rw-cli | phone | lsg | 238 | 248 | 12.9 | 13.5 | 1.8 |
| ff | desktop | logseq | 13,948 | 13,964 | 18,994 | 9,096 | 2,578 |
| ff | desktop | lsg | 23.6 | 32.0 | 11.1 | 13.5 | 1.8 |
| ff | phone | logseq | 144,596 | 144,644 | 18,069 | 7,956 | 1,545 |
| ff | phone | lsg | 244 | 260 | 11.1 | 13.5 | 1.8 |
| Contrando | desktop | logseq | 14,534 | 14,544 | 18,679 | 8,087 | 2,541 |
| Contrando | desktop | lsg | 22.1 | 28.0 | 7.8 | 13.5 | 1.7 |
| Contrando | phone | logseq | 141,579 | 141,604 | 18,069 | 7,956 | 1,551 |
| Contrando | phone | lsg | 238 | 248 | 7.8 | 13.5 | 1.8 |
| AI/E/WS/25/d1 | desktop | logseq | 13,532 | 13,548 | 18,679 | 8,087 | 2,575 |
| AI/E/WS/25/d1 | desktop | lsg | 24.2 | 36.0 | 9.1 | 13.5 | 1.7 |
| AI/E/WS/25/d1 | phone | logseq | 144,838 | 144,880 | 18,069 | 7,956 | 1,562 |
| AI/E/WS/25/d1 | phone | lsg | 245 | 260 | 9.1 | 13.5 | 1.8 |
| GitP/A/Session/24/11/19-Tue | desktop | logseq | 14,418 | 14,436 | 18,679 | 8,087 | 2,569 |
| GitP/A/Session/24/11/19-Tue | desktop | lsg | 26.4 | 36.0 | 9.3 | 13.5 | 1.8 |
| GitP/A/Session/24/11/19-Tue | phone | logseq | 143,013 | 143,072 | 18,069 | 7,956 | 1,551 |
| GitP/A/Session/24/11/19-Tue | phone | lsg | 251 | 260 | 9.3 | 13.5 | 1.8 |
| 2026-10-23 Fri | desktop | logseq | 13,768 | 13,780 | 18,679 | 8,087 | 2,558 |
| 2026-10-23 Fri | desktop | lsg | missing from this build | | | | |
| 2026-10-23 Fri | phone | logseq | 141,410 | 141,440 | 18,069 | 7,956 | 1,442 |
| 2026-10-23 Fri | phone | lsg | missing from this build | | | | |
| Home | desktop | logseq | 14,374 | 14,384 | 18,679 | 8,087 | 2,555 |
| Home | desktop | lsg | 108 | 136 | **261** | 13.5 | 2.5 |
| Home | phone | logseq | 142,175 | 142,196 | 18,069 | 7,956 | 1,552 |
| Home | phone | lsg | 231 | 244 | **261** | 13.5 | 2.4 |
| All pages | desktop | logseq | 15,250 | 15,256 | 18,679 | 8,087 | 2,585 |
| All pages | desktop | lsg | 95.1 | 124 | **261** | 13.5 | 2.5 |
| All pages | phone | logseq | 147,783 | 147,796 | 18,069 | 7,956 | 1,576 |
| All pages | phone | lsg | 223 | 236 | **261** | 13.5 | 2.3 |

## Search latency per keystroke

| Profile | Build | Keystrokes | Median (ms) | 95th percentile (ms) |
| --- | --- | ---: | ---: | ---: |
| desktop | logseq | 36 | 651 | 1517 |
| desktop | lsg | no search | | |
| phone | logseq | 36 | 2279 | 11164 |
| phone | lsg | no search | | |
