# Essential-mac cask mechanism matrix

Classification pinned 2026-08-13. Operational differential result: **PENDING
PLAN 018**. Nothing in this file is a real-brew parity PASS yet.

Source identity for every row:

- API: `https://formulae.brew.sh/api/cask/<token>.json`
- tap: `homebrew/cask`
- tap commit: `139b32436d745fd04f1d531bad85b8864a7c7270`
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

| Cask                          | Version                                            | SHA-256                                                            |
| ----------------------------- | -------------------------------------------------- | ------------------------------------------------------------------ |
| 1password                     | 8.12.32                                            | `68e7f55911ee92b8174435e463527b4a7e18ccb35674dd70bd0e666df0bf64be` |
| 1password-cli                 | 2.38.1                                             | `4ba5d569da6361aeae451098ebe8385c3d4d9019f4f013a3d859b06e9195d3bf` |
| bartender                     | 6.6.2                                              | `b16fe954b5102d9e18f93c504f0f98425f09303b5d9fc42cb93d5b7edae035a4` |
| chatgpt                       | 26.803.81509                                       | `da8315ec65a0b0c320ae8a99f63e6c3c8357e2abccc0d1d6855cc6ec20043a49` |
| claude                        | 1.28929.0,d1a6bcd4ef8627d603a8290548a984220b6701cf | `2dc3de940a0cb8ab1947c5d84af25f3f10f95acf5e059e12a3b0f7142269e026` |
| claude-code                   | 2.1.221                                            | `2dacc2906d5f95ad989ccd5372ed46119acfb50a9f11cfb193401bf2495d90ae` |
| cleanshot                     | 4.8.10                                             | `aac94efb4e3e9404d468a258143783a530b161d1aed4ea78d3f634ff138927ee` |
| cloudflare-warp               | 2026.6.880.0                                       | `aba6fb9c6e75bcc531767e801a6e298db5ef446a815575b1c67559da4a78e700` |
| codex                         | 0.147.0                                            | `f4fa12b64b51efe56065f32706cbc99b859d041b738ae99f28b3ff02d011d9ff` |
| codexbar                      | 0.49.3                                             | `b35286a7a55895e457309707c78d9a23bd27cf2cf285cf4346af2ae97a0e0ffe` |
| font-jetbrains-mono           | 2.304                                              | `8857c1ff6dcda6b22e412f778e1eba30f6edc2135f48d9cce61320d8bb0e6c09` |
| font-jetbrains-mono-nerd-font | 3.5.0                                              | `703bb8720be95bbee93fb1f4aa1d8f8741281f51ced36a2f291114c3e0f677f6` |
| ghostty                       | 1.3.1                                              | `c9c0b6d384209d88e4e73c94014d5d9dfa4b64c0c6b862eeb5b6e20e3fafe7a9` |
| google-chrome                 | 151.0.7922.138                                     | `5092624bae84946de8ace68435a362d14789d9e840973df37489eade685eaf5d` |
| grammarly-desktop             | 1.183.1.0                                          | `17f2ae48c810336ea1503e4b6f54c3f7859acf0163958fb4830df4ca703f367c` |
| grok-build                    | 1.0.3                                              | `7ed4bda3e1c7997805c9ed3bc7f459b5ec9c95fa33c82b41c883dd2b7dc39ca7` |
| handbrake-app                 | 1.11.2                                             | `90ba5f9138029e4b41903b8845a45a7c33f5de4149df2e9e90a476224c0d8163` |
| jetbrains-toolbox             | 3.6.4,3.6.4.86641                                  | `c09c2d65b50a0c634c0ad5491623a7346f7611b17471f41af138a28ff9562ea8` |
| kimi                          | 3.1.8                                              | `dcacc6e2a8c65f7c3d1776b3c6b64dd67f8cc8c64bc102a2dbcb54ed6659a1d6` |
| little-snitch                 | 6.4.1                                              | `058eccb22170b39d01f6ec117d8e0bf18cfe1bcce9b5d8d11b69bddd52fddf84` |
| notion                        | 7.30.0                                             | `d7248ed997d3abf27a9c9f1f77f228fed1cf393760ad5a9f25e9057b6cd64605` |
| opencode-desktop              | 1.18.16                                            | `02f6909165ddfbe86131a1c5650efaec9edb314398e016bb8215f394e3024f0c` |
| orbstack                      | 2.2.3,20963                                        | `49d71aa9d2da1cd0300475f6cf4568afa6c799287035f91e8c8e99ff0632c749` |
| plex-media-server             | 1.43.3.10861,07dfddaeb                             | `55006d2b666136dc15f77be69d33c64d842e476cdd9b336bdb37f32e41a694b6` |
| sketch                        | 2026.2.1,231087                                    | `de8cf0e093a429ecb2b57ba4d59e669664d06ffda69a9238b5b081cef2a827f9` |
| speechify-voice-ai            | 3.12.0                                             | `ec7f9b08e91fa787538a20ed29d4fbe97708c1e977e917ed1a6eaaa2316a15ba` |
| sublime-text                  | 4200                                               | `b09ce2758807d989c567c6690d8e48e8aa32c973518340be0b7794c88d073e30` |
| superwhisper                  | 2.17.2                                             | `e65de3a3e41843c9918578f868039d59d80c8e28c2bca7ced9b7970c9eeb26d3` |
| surge                         | 6.8.1,12030,69f4be88db9663476f31a6b264109f0b       | `75b48149d0516b1d70147dcbac281bf3c58ccdc2a2e9f4189b57a2dacaad75e4` |
| tableplus                     | 26.9.6,762                                         | `ef002864acb9661958eeff39bb8d044e60a582f1a2c55dfe208238c604a6de68` |
| tor-browser                   | 15.0.19                                            | `91538adddf58144870f9325119af05864ed754e48fc390a5fbf7461c504d639b` |
| transmission                  | 4.1.3                                              | `cca0f53d6d607cd2b28689424aacf72bb398f401b68f12e41a1508229638ab05` |
| visualvm                      | 2.2.1                                              | `4769589b3655f88ca189e9bb4590142565ecc6e07f8c5fb44fdab3d345334e1c` |
| vlc                           | 3.0.23                                             | `9fd2a031ca031d8df6de3355174de318f4e12b3e0851e0c07e3e41a2744bfec1` |
| yaak                          | 2026.5.0                                           | `281f5e65f65cc86d6dfbda431c345f9427ed37d1e60f1c8b2a7bc06364427cee` |
| zed@preview                   | 1.16.0                                             | `3dd12679822320209a35beec806531865218f1645d684cdfcfccab04d3ecc8c3` |
| zoom                          | 7.1.5.84650                                        | `b49efb230a9cc90257b681d83d9d082e2df9d916287e6e95aa238b5038199e73` |

## Required plan 018 evidence

Real differential representatives, both ownership directions, combined mise
SHA, exact job URLs, and completion-marker contents still must be added. Unit
tests prove parsing/transactions only; they cannot promote any pending row to
PASS.
