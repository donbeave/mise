# Essential-mac cask mechanism matrix

Classification pinned 2026-08-13. Operational differential result: **PENDING
PLAN 018**. Nothing in this file is a real-brew parity PASS yet.

Source identity for every row:

- API: `https://formulae.brew.sh/api/cask/<token>.json`
- tap: `homebrew/cask`
- drift-audit tap commit: `38e49d2d9b9113d2384550124f9dca83323c73a8`
- Homebrew behavior reference: `6.0.17`, commit
  `4dacfe77a24dead72de749c0876028b77b99cd04`

Legend: `A` moved app, `B` binary, `G` generated completion, `M` manpage,
`F` font, `C` declared completion, `W` command wrapper, `P` pkg installer.
Uninstall actions are `L` launchctl, `Q` quit, `S` signal, `R` script, `U`
pkgutil, and `D` delete. `Z` means zap metadata exists but mise never runs zap
implicitly and exposes no zap operation. `implemented / oracle pending` is code
support only. `UNSUPPORTED` means fresh availability must fail before mutation.

No pinned corpus row currently publishes a standalone `service` or opaque
`artifact` stanza. Both mechanism types are explicitly UNSUPPORTED and have
synthetic parser tests proving rejection before any installable sibling can
mutate state; they are not silently classified as metadata.

## Pinned mechanism accounting

| Cask                          | Artifacts | Steps             | Uninstall | Auto | Versioned | Privileged/system | Zap | Engine support                                           |
| ----------------------------- | --------- | ----------------- | --------- | ---- | --------- | ----------------- | --- | -------------------------------------------------------- |
| 1password                     | A         | —                 | L,Q       | ✓    | —         | —                 | Z   | implemented / oracle pending                             |
| 1password-cli                 | B,G       | —                 | —         | —    | —         | —                 | Z   | implemented / oracle pending                             |
| bartender                     | A         | —                 | L,Q,D     | ✓    | —         | —                 | Z   | **UNSUPPORTED: protected `/System` delete**              |
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

The SHA-256 is over canonical sorted JSON after removing live `analytics`,
`generated_date`, and the tap-global `tap_git_head`. Every per-cask
`ruby_source_checksum`, version, artifact URL/checksum, and mechanism remains
pinned. The global tap head is recorded above but excluded because an unrelated
tap commit otherwise invalidates every unchanged cask fixture.

| Cask                          | Version                                            | SHA-256                                                            |
| ----------------------------- | -------------------------------------------------- | ------------------------------------------------------------------ |
| 1password                     | 8.12.33                                            | `7f99a444e4e3f75e59f5730850348709ac9c3f923de5efe780b7f352de00eeb7` |
| 1password-cli                 | 2.38.1                                             | `7f8a437caee68aefa8c965ed2f1e14a685eb4330e83370bef4ec7f6dc0fdbcfc` |
| bartender                     | 6.6.2                                              | `c5078747fa2b5ba5043e183891a72a45d01bf3053013f5d88a287bb253b816d9` |
| chatgpt                       | 26.803.81509                                       | `26df9e4ac9ab1225a5efb0a97e8eb321f229fb8fb5a43f2109360c4968ee5c0e` |
| claude                        | 1.28929.0,d1a6bcd4ef8627d603a8290548a984220b6701cf | `10f7f3a27e43c03202c54002bbde4b16a9c1444b2d0f080e694f69ec5936e42f` |
| claude-code                   | 2.1.222                                            | `1b605465a7e9a407303a454e77bd8406db9969ce1da14970382775185aba0174` |
| cleanshot                     | 4.8.10                                             | `ee8175bdc2ffdfcf7cf94a7c2c17e12a83621b28f204b874410cd3a232031429` |
| cloudflare-warp               | 2026.6.880.0                                       | `086798c2bb9d5afe1097baa3c2c7b988ccf7f56d8f3b830597f507e315d405ea` |
| codex                         | 0.147.0                                            | `7c4c425cc4c1f2efc499468a064195c2bff09019fb9959bf3692678c60243135` |
| codexbar                      | 0.49.3                                             | `6ee8f8d1ba0c8ac4ba246d2ff7223bbefe05df1ada7174c8e1aa769131e13131` |
| font-jetbrains-mono           | 2.304                                              | `4aa667b54ee06b86b2473d6cfbedb0900527e918d0be8f89b2026efd69588ca2` |
| font-jetbrains-mono-nerd-font | 3.5.0                                              | `968a3b3af709f03b8ee779a7e4b2158f184d6ec3c3b20d4d709c043ed3e3074c` |
| ghostty                       | 1.3.1                                              | `fd15d22ac65ec6603d446f636014487d07515e296e961600575d39f7e564469b` |
| google-chrome                 | 151.0.7922.138                                     | `2a120276338086c896ce8fed50c78d55589aecb71c8f62198e3bd30f58df2751` |
| grammarly-desktop             | 1.183.1.0                                          | `a169237422edcee61f7ddd7f69284c8216203173749651c3c7052e0bdde7eab7` |
| grok-build                    | 1.0.3                                              | `f27af9215951e93c04a99de48fa1d646ece0a7476649a7617602396bae1cf43b` |
| handbrake-app                 | 1.11.2                                             | `f2a37c33995358be549285ca463b36dd7bf32ca1be8f5e5adb80f68b704a9fa1` |
| jetbrains-toolbox             | 3.6.4,3.6.4.86641                                  | `21f810a08d37fa173257789e3033e2b1cc17812ab1e6f336bb9f84a687e80717` |
| kimi                          | 3.1.8                                              | `014418985609e8e14d373341281c4969e9fc048a1d6372f02d8a383679995434` |
| little-snitch                 | 6.4.1                                              | `4ad35e4ebe93e32f5522bed384960cde0546f1a6b00a0cbbd1542ddcb405bcdf` |
| notion                        | 7.30.0                                             | `6d0630ddfb71e224d0a92648d2aba394833f3e50bd6f53f1f09453f6d6a452d0` |
| opencode-desktop              | 1.18.18                                            | `f96d55de8513558057e4e2b3c8555a16e0be09a9027477fd511d1bac0fef006c` |
| orbstack                      | 2.2.3,20963                                        | `88873c2daa481ab84233017f80107e72f2b53bc1bab14b36fa0bf11b1f91a61e` |
| plex-media-server             | 1.43.3.10861,07dfddaeb                             | `41f07656e5f23fe3645cb49fa15c2c13c52f4cd3acdce78e2443a8e8017e7004` |
| sketch                        | 2026.2.1,231087                                    | `e4e708e5e9324d81a90cc0c4379f3bfc08c1d68131ee8586eb3f0868ddb95598` |
| speechify-voice-ai            | 3.12.0                                             | `b9b40a7911d61140e6d5f37c3a5b961f6ef7f391c38b4e4862d3d8e1bdcb8596` |
| sublime-text                  | 4200                                               | `ef679312ae564b9856d8055a46114e32c9833f7dbaee2ad7ec84b4036e2a83ff` |
| superwhisper                  | 2.17.2                                             | `c0b88b61c3b8ab87596e13cfc01d513c97f036ab9c85de7eaa2ddbf78c2e83d0` |
| surge                         | 6.8.1,12030,69f4be88db9663476f31a6b264109f0b       | `0fb4149cc488687d0cb8a720cbaf77dfa13910aa03fee77bc38154cb3148a2e2` |
| tableplus                     | 26.9.6,762                                         | `af5d8b2c92c5621d20467069c0ae1827a552344b3478fa0950a972b9b397aa2f` |
| tor-browser                   | 15.0.19                                            | `688ab3b9a287158ab7b7fc594bc7d472a6c4d212b4ebd1599b83c2a11a55a203` |
| transmission                  | 4.1.3                                              | `d10996b4b17b3d9379605232cf5fc4aabc4d57353acaf0c494f61cf789ae1591` |
| visualvm                      | 2.2.1                                              | `69b66cb7cc3d8951b0ac8d098016a126e5eb1c87c12ee0923f61a4bb48868d3c` |
| vlc                           | 3.0.23                                             | `d2006154048a8f146edf3f4827019e42deb7f82e03eb954c82d9d9ea0311dcd5` |
| yaak                          | 2026.5.0                                           | `bd15089ef073e10a7b9b35be5794c59d3361b5191ae8ee513b060414b0946efc` |
| zed@preview                   | 1.16.0                                             | `8dac7458ef35a6ef5bc042653b722bcad4eaf9068520d0fe7765e5f8003397f3` |
| zoom                          | 7.1.5.84650                                        | `99bbf3dea0677e8e6418bdf247ea16443a50402af7194d84f4e81fcb3bf56492` |

## Required plan 018 evidence

Real differential representatives, both ownership directions, combined mise
SHA, exact job URLs, and completion-marker contents still must be added. Unit
tests prove parsing/transactions only; they cannot promote any pending row to
PASS.
