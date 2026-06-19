# RFC 0008: Object and Embedded Image Stream Candidates

## Status

Diagnostic only.

この note の observations は decoded Ichitaro object semantics ではない。future model、layer、renderer work のための `decoded=false` evidence として保存する。

## Motivation

full-layout PDF export には extracted text だけでは足りない。rjtd は images、vector/shape objects、tables、および layout records を model/control/layer objects として recover してから exporter で render する必要がある。

これは rhwp-compatible policy に従う。exporter は raw CFB streams から直接 bytes を注入せず、document model と page/layer tree を consume する。

## Diagnostic Command

`rjtd object-stream-candidates <file>` は readable CFB streams を scan し、visual/object recovery に関係しそうな streams を report する。

最初の implementation は以下の evidence types で candidate streams を classify する。

| Evidence | Meaning |
| --- | --- |
| `object-path` | stream path が `EmbedItems`、`Embedding`、`JSFart`、`CompObj`、`Ole`、`Object`、`Bin` など embedding/object/OLE/binary-object naming を含む |
| `image-path` | stream path が `Image`、`Picture`、`Graphic`、`PNG`、`JPEG`、`BMP`、`WMF`、`EMF` など image-oriented naming を含む |
| `shape-path` | stream path が `Figure`、`Shape`、`Draw`、`Frame`、`LayoutBox`、`SVG` など shape/layout naming を含む |
| `table-path` | stream path が table/cell naming を含む。ただし position/style table names は除外する |
| `so-marker` | payload が preserved `SO\0\0` object/control marker family を含む |
| `image-signature` | payload が PNG、JPEG、GIF、TIFF、start-position BMP、placeable WMF など recognizable binary image signature を含む |
| `svg-signature` | payload が textual `<svg` evidence を含む |

Output rows は stream path、stream size、reason list、first image signature offsets、first SVG offsets、first SO marker offsets、short payload prefix、`decoded=false` を保存する。

## Current Sweep

current 61 local `.jtd`、`.jtt`、`.jttc` samples 全体で command を sweep した。

| Metric | Count |
| --- | ---: |
| samples checked | 61 |
| readable streams scanned | 2,253 |
| candidate rows | 933 |
| files with any candidate | 43 |
| files with `object-path` evidence | 27 |
| files with `shape-path` evidence | 42 |
| files with `table-path` evidence | 0 |
| files with `so-marker` evidence | 4 |
| files with `image-signature` evidence | 17 |
| files with `svg-signature` evidence | 0 |
| unreadable candidate scan streams | 0 |
| total `object-path` rows | 429 |
| total `shape-path` rows | 503 |
| total `table-path` rows | 0 |
| total `so-marker` rows | 11 |
| total `image-signature` rows | 64 |
| total `svg-signature` rows | 0 |

Representative observations:

- `ichitaro-20030706232401-success-001-success_data-kazoku_ryoko.jtd` は `/EmbedItems`、`/Figure`、`/FigureData`、`/Frame`、`/LayoutBox` candidates を expose する。`/Figure` と `/PaperMark` にも `SO\0\0` hits が保存される。
- `ichitaro-20030316043238-success-001-success_data-iwata_file.jtd` は `/EmbedItems/Embedding */Contents` streams 内に JPEG signatures を expose し、`jpeg@67` や `jpeg@72` のような offsets が見える。
- `ichitaro-20030422210439-success-002-success_data-natsu.jtd` は 12 image-signature rows を expose し、多くは embedded `Contents` または `EmbeddedPress` streams 内の JPEG signatures である。
- `ichitaro-20030829031540-success-004-success_data-hyo.jtd` は path、SO marker、image signature、SVG signature のいずれでも object-stream candidates を expose しない。visible table content は named table object stream ではなく、`/DocumentText` controls/records と layout/style streams の decoding に依存する可能性が高い。

## Interpretation

Image recovery は stream/path candidates と binary image signatures により testable になった。複数 samples は short object header の後に embedded JPEG evidence を `EmbedItems` streams 内で持つ。

Shape and layout-object recovery は `/Figure`、`/FigureData`、`/Frame`、`/LayoutBox` families から始めるべきである。これは path-level candidates であり、decoded geometry ではない。

Table recovery は current corpus では named CFB table streams に依存すべきではない。`hyo` sample を含めて `table-path` が 0 であるため、table structure は `/DocumentText` controls、style streams、または layout mark records に encode されている可能性が高い。

## Model Preservation

parser はこれらの stream observations を top-level `objectStreamCandidates` として decoded-false model evidence に promote するようになった。

JSON export と app-core `getDocumentInfo` は各 candidate を以下の fields で expose する。

- `path`
- `size`
- `reasons`
- `ownershipCandidate`
- `ownershipReferences`
- `fdmIndexEntries`
- `imageSignatures`
- `imagePayloads`
- `svgOffsets`
- `soOffsets`
- `payloadPrefixHex`
- `decoded:false`

各 `imagePayloads` row は `kind`、`mime`、`signatureOffset`、`start`、`end`、`length`、`complete`、`objectEnvelope`、`payloadPrefixHex`、`decoded:false` を記録する。document model は payload bytes も内部に保持するため、future renderer は exporter から raw streams を開き直さず、model 経由で image resources を消費できる。

`objectEnvelope` field は payload 周辺の undecoded bytes を保存する。header start/end/length、header prefix、trailer start/end/length、trailer prefix、そして header 最後の 4 bytes が detected payload length と little-endian `u32` として完全一致する場合のみ conservative `declaredPayloadLength` candidate を記録する。さらに decoded-false `headerFields` として prefix `u16LePrefix`/`u32LePrefix` numeric candidates、および header が path-length byte と NUL-terminated embedded source path を持つ場合の `sourcePathCandidate` を expose する。これは evidence であり decoded Ichitaro object geometry ではない。

candidate-level `ownershipCandidate` も decoded-false である。これは stream path だけから derive され、`stream-path` basis、family、optional storage path、optional `embeddingIndex`、stream role を記録する。例として `EmbedItems` contents/embedded-press streams、`FigureData` `FDMVector`、root figure/frame/layout streams がある。page placement や final object geometry を証明するものではない。

candidate-level `ownershipReferences` field は decoded-false cross-stream evidence である。現在は path-derived `Embedding N` owner を持つ embedded image candidates にだけ attach し、その `N` の byte-pattern matches を `FigureData`、`/Figure`、`/Frame`、`/LayoutBox`、`/PageMark`、`/PaperMark` streams 内で記録する。各 row は `targetPath`、`encoding`、`totalMatches`、bounded `offsets` preview、`decoded:false` を持つ。これは candidate embedding index が object/layout-related streams に観測されることを示す evidence であり、authoritative record field や page geometry をまだ特定しない。

`getValidationWarnings` はこれらを `JtdObjectStreamCandidateDiagnosticOnly` として report する。

同じ 61 local samples の JSON export sweep は 0 failures で、43 files に 933 `objectStreamCandidates` を保存する。image-signature candidates を持つ files は 17、SO-marker candidates を持つ files は 4、table-path files は 0、SVG-signature files は 0 である。これは diagnostic CLI distribution と一致し、evidence を document model 内に保持する。

model-preserved `imagePayloads` field への 2 回目の JSON export sweep も、61 local samples すべてで 0 failures となった。complete payload spans を持つ files は 14、complete payloads は 94、preserved payload bytes は 373,466。内訳は JPEG 62 rows/8 files、GIF89a 31 rows/9 files、GIF87a 1 row/1 file である。

同じ sweep では object envelopes も 94 件保存される。little-endian declared payload length と一致する rows は 6 件で、現時点ではすべて `Embedding */Contents` JPEG streams に限られる。large `FDMVector` と `EmbeddedPress` wrapper streams は nested object records が decode されるまで envelope-only evidence として扱う。

Header field candidate sweep results: 61 local samples はすべて export に成功し、66 payload rows が `sourcePathCandidate` を expose する。source-path candidates は現時点ですべて `Contents` stream 由来である。path extension の内訳は `jpg` 34 rows、`gif` 32 rows。first little-endian prefix word は 59 rows で `9`、6 rows で `4` であり、embedded image contents で観測される dominant `09 00 01 00` と secondary `04 00 01 00` header families に対応する。

Ownership candidate sweep results: 61 local samples はすべて export に成功し、933 object stream candidates のうち 474 が path-derived `ownershipCandidate` を expose する。94 image payload rows はすべて ownership candidate に covered される。payload rows の role 内訳は `contents` 67、`embedded-press` 8、`fdm-vector` 19。candidate families は `embed-items` 335、`figure-data` 38、`figure` 31、`frame` 42、`layout-box` 28 である。

Ownership reference sweep results: 61 local samples はすべて export に成功し、12 files が cross-stream reference candidates を expose する。56 embedded image candidates が `ownershipReferences` を持ち、model は 646 reference rows と 10,055 total byte matches を保存する。reference rows の target family 内訳は `frame` 212、`figure-data` 140、`page-mark` 117、`paper-mark` 80、`layout-box` 67、`figure` 30。encoding 内訳は `u16-be` 195、`u16-le` 184、`u32-be` 150、`u32-le` 117。covered source candidates はまだすべて `embed-items` rows (`contents` 52、`embedded-press` 4) で、image payload rows 94 のうち 75 が cross-stream reference evidence を持つ。

`rjtd object-ownership-references <file>` は、これらの model-owned references を match-context diagnostics として展開する。各 reported preview offset について source stream、target stream、encoding、offset、その reference row の total matches、mod2/mod4 alignment、local context hex、match offset で読んだ le/be 16/32-bit values を出力する。この command は diagnostic-only であり renderable geometry を作らない。

61-sample `object-ownership-references` sweep は 0 failures で、同じ 12 files から 3,273 preview-offset rows を報告する。target-family rows は `figure-data` 1,021、`frame` 941、`layout-box` 538、`page-mark` 502、`paper-mark` 158、`figure` 113。reported offsets は uniform alignment ではなく、mod2 `0` 1,401 vs `1` 1,872、mod4 `0` 728、`1` 855、`2` 673、`3` 1,017 に分かれる。match offset では `u16-be` と `u16-le` rows が embedding index を対応する 16-bit value として直接 expose し、`u32-le` rows は low 16 bits に expose する。これは後続の record-field analysis を絞り込むが、authoritative geometry field をまだ特定しない。

`rjtd object-ownership-reference-fields <file>` は、同じ preview offsets を candidate record strides に投影する。target path、encoding、stride、field offset ごとに match count、source count、embedding index set、row-index preview、cross-row match count を summarize する。この command は各 reported offset を固定 stride set (`4,8,12,16,20,24,28,32,36,40,44,48,52,56,60,64,68,72,80,84`) に対して意図的に試すため、出力は decoded record table ではなく hypothesis surface である。

61-sample `object-ownership-reference-fields` sweep は 0 failures で、同じ 12 files から 33,492 projected field groups を報告する。cross-row-free かつ stride >= 12 の最も強い候補は現在 `frame/u16-le/12/5` が 12 files で weighted matches 106、`frame/u16-be/12/7` が 9 files で 95、`frame/u16-be/20/15` が 9 files で 74 である。これは次の分析が frame records に集中すべきことを示唆するが、どの stride や field が semantically authoritative かはまだ証明しない。

`rjtd object-frame-reference-records <file>` は、最も強い frame projections を candidate row bytes として展開する。現在は decoded-false projection families のうち strongest three (`u16-le/12/5`、`u16-be/12/7`、`u16-be/20/15`) だけを報告し、source stream、embedding index、target stream、encoding、stride、field offset、match offset、row index、row start、row hex、BE/LE 16-bit field views、BE/LE 32-bit field views を出力する。

61-sample `object-frame-reference-records` sweep は 0 failures で、12 files から 275 rows を展開する。内訳は `u16-le/12/5` 106 rows、`u16-be/12/7` 95 rows、`u16-be/20/15` 74 rows。rows は `00010000000N000000020001` style の 12-byte rows や `00000000010200380000000N` rows のような repeated byte families を expose する。これらの families は frame-record analysis にとって有望な evidence だが、まだ decoded placement geometry や paint operations ではない。

`rjtd object-frame-record-families <file>` は、展開済み rows を named decoded-false diagnostic families に group する。これらの名前は specification terms ではなく observation buckets である。たとえば `frame-index-flag-row12` は big-endian word view の trailing fields が小さい flag-like values を持つ 12-byte rows を捕捉し、`frame-index-tail-coordinate-row12` と `frame-index-tail-window20` は `/Frame` reference projections で繰り返し現れる tail-window shapes を捕捉する。

61-sample `object-frame-record-families` sweep は 0 failures で、同じ 12 files の 275 records を group する。Family counts は `frame-index-tail-coordinate-row12` 69、`frame-index-tail-window20` 69、`frame-index-mixed-row12` 61、`frame-index-flag-row12` 45、`frame-index-tail-zero-row12` 24、`frame-index-mixed-window20` 5、`frame-index-tail-mixed-row12` 2。次の promotion step では、どの field も authoritative geometry と扱う前に、これらの row families を page/layout marks と image payload ownership に照合するべきである。

`rjtd object-frame-row-links <file>` は、展開済み 20-byte frame windows が matching 12-byte frame row を suffix として含むかを確認する。これは independent record candidates と smaller record の周辺 context windows を区別するための diagnostic である。

61-sample `object-frame-row-links` sweep は 0 failures である。9 files に 74 row20 windows があり、そのうち 69 は same-source 12-byte suffix row に link し、5 は unlinked のまま残る。Linked row はすべて `frame-index-tail-window20 -> frame-index-tail-coordinate-row12` である。これは `u16-be/20/15` の `tail-window20` projection が通常 independent authoritative record ではなく、`u16-be/12/7` row の context window であることを強く示唆する。unlinked rows はすべて `frame-index-mixed-window20` なので、object/header evidence がさらに得られるまでは別扱いにするべきである。

Parser/export JSON は、この evidence を `objectStreamCandidates[].frameReferenceRows` として保存する。各 row は `targetPath`、`encoding`、`stride`、`fieldOffset`、match `offset`、`rowIndex`、`rowStart`、family、raw `rowHex`、optional `suffixLink`、`decoded:false` を持つ。これにより future image-placement work は model-first のまま維持される。exporters は raw `/Frame` streams を直接 scan するのではなく、model-owned rows を consume するべきである。

61-sample JSON export sweep は 0 failures で、同じ 12 positive files に `frameReferenceRows` を保存する。export された rows は 275、suffix links は 69 であり、family counts は CLI family sweep と完全に一致する。

`rjtd object-image-frame-candidates <file>` は、この evidence を image payload source 側から summarize する。complete image payload spans を持つ各 source について、path-derived embedding index、payload kinds、total frame rows、row-family counts、row12 coordinate-looking candidates、row20 suffix-link counts、LE row12 counts、diagnostic preferred bucket、coordinate-looking row12 pairs を報告する。この command も decoded-false であり、preferred bucket は調査優先度であって renderable geometry ではない。

61-sample `object-image-frame-candidates` sweep は 0 failures である。image payload source 付き files 14、image sources 60、frame-linked sources 56、`/Frame` rows を持たない sources 4、同じ 275 frame rows を見つける。Diagnostic preferred buckets は `row12-tail-coordinate` 27、`row12-tail-zero` 8、`u16-le-row12` 20、`none` 5 に分かれる。`none` rows の多くは path-derived `Embedding N` を持たない `FDMVector` sources であり、`u16-le-row12` bucket は coordinate-looking ではなく index/flag-like rows が中心である。したがって `row12-tail-coordinate` は強い placement-analysis candidate だが、PDF image rendering に十分な coverage ではない。

`rjtd object-fdm-index <file>` は `/FigureData/*/FDMIndex` streams を sibling `/FigureData/*/FDMVector` streams と照合する。現在観測した row shape は 20-byte header の後に 22-byte rows が続き、big-endian vector offset、16-bit kind field、4 つの big-endian signed bbox-like fields を持つ。command は各 row を次に大きい vector offset までの vector segment に link し、segment image signatures を decoded-false evidence として報告する。

61-sample `object-fdm-index` sweep は 0 failures である。indexes 付き files 31、index streams 39、parsed rows 417、image signatures 付き rows 6、image hits 13、missing sibling vectors 2 を見つける。これは `FDMIndex`/`FDMVector` が `Embedding N` `/Frame` rows とは別の object-placement evidence path であることを示す。

`rjtd object-fdm-index-shape <file>` は、これらの raw row projections を shape families に分離する。exact 22-byte tables、auxiliary payload bytes が後続する declared-count prefix tables、mixed declared rows、unknown-header streams、missing sibling vectors を区別する。

61-sample `object-fdm-index-shape` sweep は 0 failures である。indexes 39、`fdm-index-v1` headers 35、unknown headers 4、plausible declared counts 34 を見つける。raw whole-stream 22-byte projection は 417 rows と invalid offsets 252 を持つが、declared-count prefix projection は 147 rows、invalid offsets 43、同じ image hits 13 を持つ。Shape counts は `row22-count-prefix` 17、`row22-exact` 14、`row22-mixed-declared` 3、`unknown-header` 3、`missing-vector` 2。これは、以前 invalid と見えた rows の多くが real FDMIndex rows ではなく FDMIndex table 後の auxiliary payload bytes である可能性が高いことを示す。

`rjtd object-fdm-index-rows <file>` は、これらの tables を row-level diagnostic view として出力する。各 row の scope (`declared`, `post-declared`, `raw`)、role (`vector-segment`, `coordinate-like-invalid`, `invalid-vector-offset`)、BE16/i16 field views、raw row bytes、segment image signatures を報告する。role は decoded-false である。`coordinate-like-invalid` は、BE16 fields として見たときに signed-coordinate data に似るという意味であり、semantic record type が decoded されたことを意味しない。

61-sample `object-fdm-index-rows` sweep も 0 failures である。indexes 付き files 31、indexes 39、rows 417、declared rows 147、post-declared rows 253、raw rows 17、valid vector rows 165、invalid rows 252、image hits 13、missing vectors 2 を見つける。Role counts は `vector-segment` 165、`coordinate-like-invalid` 231、`invalid-vector-offset` 21。declared invalid rows 43 はすべて `coordinate-like-invalid` で、3 files に集中し、それらの invalid declared rows は image-bearing vector segments ではない。

Parser/export/app-core JSON は、declared-count prefix rows を対応する `FDMVector` candidate の `objectStreamCandidates[].fdmIndexEntries` として保存する。各 row は `indexPath`、`vectorPath`、row/index/vector offsets、vector segment length、`kind`/`kindHex`、bbox-like fields、`validVectorOffset`、vector prefix、absolute image signatures、segment-relative image signatures、`decoded:false` を記録する。

61-sample JSON export sweep は 0 failures で、24 files の 30 candidates に 147 `fdmIndexEntries` を保存する。image-linked rows を持つ files は 3 で、6 rows が 13 image hits を含む。また valid vector offsets は 104、invalid/out-of-range vector offsets は 43 であるため、この evidence は現在観測される image-bearing FDMVector segments を識別しつつ false auxiliary rows を減らす。ただし、まだ renderable page geometry や paint resources へ promote してはならない。

`rjtd object-fdm-image-candidates <file>` は、model-owned `fdmIndexEntries` のうち image-bearing subset を summarize する。segment image signatures を持つ各 row について、FDMVector source、FDMIndex source、row/vector offsets、`kind`、raw/normalized bbox-like fields、bbox order、bbox plausibility、segment image hits、complete image payload count、reason `page-placement-unproven` 付きの `renderable:false` を報告する。

61-sample `object-fdm-image-candidates` sweep は 0 failures である。candidates 付き files 3、FDM sources 3、image-bearing rows 6、image hits 13、complete payloads 11、plausible bbox rows 5、renderable rows 0 を見つける。`shanai_lan` と `tounyou` が complete/plausible rows 5 を含む。finance sample は JPEG signatures 2 を持つが complete payloads 0 と implausible bbox を持つため、signature-only diagnostic evidence のまま残す。

App-core `getPageOverlayImages` は同じ FDM rows を `unplacedDiagnostics` として expose し、`behind`、`front`、`imageCount` は empty/zero のまま維持する。各 diagnostic は `placementProven:false`、`renderable:false`、`decoded:false`、reason `page-placement-unproven` を持つ。これにより rhwp-shaped overlay API は callable だが、decoded page placement や paint resources を claim しない。

Parser/export/app-core JSON は `/Frame` fixed 60-byte records も decoded-false `objectFrameRecords` として保存する。観測された record layout は 16-byte header、offset 14 の big-endian declared count、60-byte rows を持つ。Rows は object id、record kind/type、geometry-looking fields を expose するが、units、page association、paint order が証明されるまでは diagnostic のまま扱う。

`rjtd object-fdm-frame-links <file>` は image-bearing FDMIndex rows とこれらの `/Frame` records を `fdm row index == frame object id` で相関させる。61-sample sweep は 0 failures で、positive files 3、FDM image rows 6、frame-linked rows 6、missing-frame rows 0、complete payloads 11、renderable rows 0 を見つける。これは現在観測される FDM image rows が frame-record trail を持つことを示すが、どの image payload を page 上のどこに paint すべきかはまだ証明しない。

## Next Work

- image payload signatures 前の semantic object header fields を decode し、`/Figure`、`/Frame`、`/LayoutBox`、layout mark evidence と接続する。
- `/Frame` geometry units、page association、paint order、payload-to-image selection、remaining coordinate-like FDMIndex diagnostic rows を decode してから FDMVector images を render する。
- ownership references を page geometry に promote する前に、どの `Embedding N` reference encoding と record-local offset が semantically authoritative かを証明する。
- object ownership と page geometry が証明された後にのみ、preserved image payload bytes を model-level image resources に接続する。
- non-text PDF rendering の前に decoded object/layout records から real page/layer paint operations を構築する。
- table semantics は stream-name matching ではなく、`/DocumentText` control ranges と layout/style streams から調査する。
