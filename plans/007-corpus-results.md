# Essential-mac cask mechanism matrix

Classification pinned 2026-08-13. Operational differential result: **PENDING
PLAN 018**. Nothing in this file is a real-brew parity PASS yet.

Source identity for every row:

- API: `https://formulae.brew.sh/api/cask/<token>.json`
- tap: `homebrew/cask`
- tap commit: `9a1a1d1caa8dd0b1f40107205cb3776120130245`
- Homebrew behavior reference: `6.0.17`, commit
  `4dacfe77a24dead72de749c0876028b77b99cd04`

Legend: `A` moved app, `B` binary, `G` generated completion, `M` manpage,
`F` font, `C` declared completion, `W` command wrapper, `P` pkg installer.
Uninstall actions are `L` launchctl, `Q` quit, `S` signal, `R` script, `U`
pkgutil, and `D` delete. `Z` means zap metadata exists but mise never runs zap
implicitly and exposes no zap operation. `implemented / oracle pending` is code
support only. `UNSUPPORTED` means fresh availability must fail before mutation.

## Pinned mechanism accounting

| Cask                          | Artifacts | Steps             | Uninstall | Auto | Versioned | Privileged/system | Zap | Engine support                                           |
| ----------------------------- | --------- | ----------------- | --------- | ---- | --------- | ----------------- | --- | -------------------------------------------------------- |
| 1password                     | A         | —                 | L,Q       | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| 1password-cli                 | B,G       | —                 | —         | —    | —         | —                 | Z   | implemented / oracle pending                             |
| bartender                     | A         | —                 | L,Q,D     | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| chatgpt                       | A         | —                 | Q         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| claude                        | A         | —                 | Q         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| claude-code                   | B         | —                 | —         | —    | —         | —                 | Z   | implemented / oracle pending                             |
| cleanshot                     | A         | —                 | Q         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| cloudflare-warp               | P         | —                 | L,Q,R,U,D | ✓    | —         | ✓                 | Z   | **UNSUPPORTED: pkg/BOM + script**                        |
| codex                         | B,G       | —                 | —         | —    | —         | —                 | Z   | implemented / oracle pending                             |
| codexbar                      | A,B       | —                 | Q         | —    | —         | —                 | Z   | implemented / oracle pending                             |
| font-jetbrains-mono           | F×34      | —                 | —         | —    | —         | —                 | —   | implemented / oracle pending                             |
| font-jetbrains-mono-nerd-font | F×96      | —                 | —         | —    | —         | —                 | —   | implemented / oracle pending                             |
| ghostty                       | A,M×2,C×3 | —                 | —         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| google-chrome                 | A         | —                 | —         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| grammarly-desktop             | A         | —                 | L         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| grok-build                    | B×2,G     | —                 | —         | —    | —         | —                 | Z   | implemented / oracle pending                             |
| handbrake-app                 | A         | —                 | —         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| jetbrains-toolbox             | A         | —                 | L,Q,S     | ✓    | —         | —                 | Z   | **UNSUPPORTED: signal**                                  |
| kimi                          | A         | —                 | —         | —    | —         | —                 | Z   | implemented / oracle pending                             |
| little-snitch                 | A         | —                 | —         | ✓    | —         | ✓                 | Z   | implemented artifacts; privileged runtime oracle pending |
| notion                        | A         | —                 | Q         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| opencode-desktop              | A         | —                 | —         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| orbstack                      | A,B×2,C×4 | run               | R         | ✓    | —         | ✓                 | Z   | **UNSUPPORTED: unconfined run + script**                 |
| plex-media-server             | A,B       | —                 | L,Q       | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| sketch                        | A         | —                 | —         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| speechify-voice-ai            | A         | —                 | —         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| sublime-text                  | A,B       | —                 | Q         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| superwhisper                  | A         | —                 | Q         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| surge                         | A,B×3     | —                 | L,D       | ✓    | —         | ✓                 | Z   | implemented / oracle pending                             |
| tableplus                     | A         | —                 | —         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| tor-browser                   | A         | —                 | —         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| transmission                  | A         | —                 | —         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| visualvm                      | A         | —                 | —         | —    | —         | —                 | Z   | implemented / oracle pending                             |
| vlc                           | A,W       | —                 | —         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| yaak                          | A         | —                 | —         | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| zed@preview                   | A,B,G     | —                 | Q         | ✓    | ✓         | —                 | Z   | implemented / oracle pending                             |
| zoom                          | P         | terminate_process | L,S,U,D   | ✓    | —         | ✓                 | Z   | **UNSUPPORTED: pkg/BOM + signal**                        |

Ghostty is deliberately a mixed-artifact row. Hidden Bar is not evidence for
its app + two manpages + three declared completions.

## Exact API payload identities

The SHA-256 is over canonical sorted JSON after removing only the live
`analytics` and `generated_date` fields. All structural cask data, versions,
artifact URLs/checksums, mechanisms, and tap identity remain pinned.

| Cask                          | Version                                            | SHA-256                                                            |
| ----------------------------- | -------------------------------------------------- | ------------------------------------------------------------------ |
| 1password                     | 8.12.33                                            | `bb82a1030961864607ed871cafdd5c6baaa725713f0e6aaab1ba071a808b90c7` |
| 1password-cli                 | 2.38.1                                             | `d819694c6428760d4b539913b5962d8e1f7ac2310253a9167eb20d4a6b1a8f14` |
| bartender                     | 6.6.2                                              | `b71788109bb5053abab6be69656ab53f5a3bc74dfd5bd5b5877435791bfa9388` |
| chatgpt                       | 26.803.81509                                       | `0d669a3bdd87e0013771a3ac6e4372ae1d890200ac9a8cc4526293a8c8e419e7` |
| claude                        | 1.28929.0,d1a6bcd4ef8627d603a8290548a984220b6701cf | `bd2960c186692f39746b76337a8187edbf8ba3f2ba0f781196ef86d8b731b15a` |
| claude-code                   | 2.1.222                                            | `4bff89d3240e4b736f30d65c52e3c11ba682387f058713f18c06515b0bfa49a9` |
| cleanshot                     | 4.8.10                                             | `653391f8eeac4d3f5841e01aac540e21339f8a0c3d602a92ec9531384d2d3e65` |
| cloudflare-warp               | 2026.6.880.0                                       | `dd2d62c16ae5e05d6021c34626d6b8a972a5b0da9cf0ac7196f63a279354576a` |
| codex                         | 0.147.0                                            | `dedcbcbea960912cac5302703b403f8cea3f481830340fc622e17f384d3fe380` |
| codexbar                      | 0.49.3                                             | `17a8ff2d3ab8b143ff76d3cdbc19ac950d2c7a3a45789c38e451a1fdddeb4224` |
| font-jetbrains-mono           | 2.304                                              | `759100caac0dbb17c0a3d54a050584a89893ed787ed84a821afedd6b99e1c8e0` |
| font-jetbrains-mono-nerd-font | 3.5.0                                              | `da91b0312efbaae14a73dc2318ba799a6aa48ddc2d03eecac3a65477fd1c69cf` |
| ghostty                       | 1.3.1                                              | `628dfe93794ad00c7025925832727f71aaea676feafa68138a2e64323a88d0aa` |
| google-chrome                 | 151.0.7922.138                                     | `755cfd18efc0258e600f27b59ed0ae819bafcfcc33c33e6c750b2d4e54235023` |
| grammarly-desktop             | 1.183.1.0                                          | `bb22709dced0a0aa8db186eb60f495f4fe6f87e9d6ccbc7e927e05e046f91fe2` |
| grok-build                    | 1.0.3                                              | `38d4a7f61366d7fbbddc524018fd4a7f055f5029aba07830769dc0e31a5189c5` |
| handbrake-app                 | 1.11.2                                             | `40a912f7302220faa78adea4d36c2aa167264c3f7f9a32056b66550f5b9c5184` |
| jetbrains-toolbox             | 3.6.4,3.6.4.86641                                  | `97ce78344ab11d8dd7390edaa97602be6a68c0c873e8d4b86b19cebd5692330a` |
| kimi                          | 3.1.8                                              | `4e245261670242d388dc4ab32dc8bddcf7738f07ebd4fadd3a67249e5616999f` |
| little-snitch                 | 6.4.1                                              | `e899d3a20862e2bf8a6b9b0952078c3e900564f2cd69e6df863fbd088f877d76` |
| notion                        | 7.30.0                                             | `531f3183f101a924499f949c6a214f53b3405fcab1f554034f9dcb971f307483` |
| opencode-desktop              | 1.18.17                                            | `ecccbcc63aa0fcb331e47468808d8dfd0c5242831e68d4c69d12c9b51dc1aadd` |
| orbstack                      | 2.2.3,20963                                        | `1aac4067bd6f70086a5a43291f2bd361e9f420d2585d4e3fe8b299529664eec8` |
| plex-media-server             | 1.43.3.10861,07dfddaeb                             | `c9c03348869baa0d196e6e8af4f8a24d43a229ab1218172199adfed7c1cf55e2` |
| sketch                        | 2026.2.1,231087                                    | `ef8e5bf55a4c3d784fbc102c44168df8468d7ab027222af97bf93118499dbb53` |
| speechify-voice-ai            | 3.12.0                                             | `80b3468dc698e88e7274dd02a847c8b1b1cde10a492112fc17941b137e142be6` |
| sublime-text                  | 4200                                               | `2b3dd706035e413596a0fec86a7f2e0d0de23f46f6a5f442839a48232b14c816` |
| superwhisper                  | 2.17.2                                             | `897fd8367cf98df7a954047b82df909093f35d9849250ca3bb462041062bb792` |
| surge                         | 6.8.1,12030,69f4be88db9663476f31a6b264109f0b       | `8f0c3fa2392b528ac4d8964b4361947c3d7ce212f44e9405b235062738b5010d` |
| tableplus                     | 26.9.6,762                                         | `8e683526c6d6f4b9ac8449c1033ded25cffd8ba7bb12d51510ead0256e43fa7e` |
| tor-browser                   | 15.0.19                                            | `4340d6b3e1804a6084f553510c01ec39658140c18cb371cecbde992ef5ab7795` |
| transmission                  | 4.1.3                                              | `5e0dfc6088107647c5c2f2a42035b01a75cecbe941d85c59e481a66896a28f2d` |
| visualvm                      | 2.2.1                                              | `ada9d84a1ed42230a44e5548d971681491f3b0616d37a389887603b7342251f7` |
| vlc                           | 3.0.23                                             | `4df4df5f21490863a16591bb2d6968174bfa4657c037be9dcac929ffdcd0d8f4` |
| yaak                          | 2026.5.0                                           | `5fef44435759ba9540978fdf2e7ee4af14f7de1345557373065663bae58db9a8` |
| zed@preview                   | 1.16.0                                             | `a78be581d8747bf90f83e76148d74ccb1bb16d6e2496b3cd40d6ab8c6e812767` |
| zoom                          | 7.1.5.84650                                        | `2563de8e659fb51c4720aecd4def75b86de8105213a7e21f27b9682acab5cbd5` |

## Required plan 018 evidence

Real differential representatives, both ownership directions, combined mise
SHA, exact job URLs, and completion-marker contents still must be added. Unit
tests prove parsing/transactions only; they cannot promote any pending row to
PASS.
