# RFC 0008: Object and Embedded Image Stream Candidates

## Status

Diagnostic only.

The observations in this note are not decoded Ichitaro object semantics. They are preserved as `decoded=false` evidence for the future model, layer, and renderer work.

## Motivation

Full-layout PDF export requires more than extracted text. rjtd must recover images, vector/shape objects, tables, and their layout records as model/control/layer objects before the exporter renders them.

This follows the rhwp-compatible policy: exporters must consume the document model and page/layer tree, not inject bytes directly from raw CFB streams.

## Diagnostic Command

`rjtd object-stream-candidates <file>` scans every readable CFB stream and reports streams that look relevant to visual or object recovery.

The first implementation classifies candidate streams with these evidence types:

| Evidence | Meaning |
| --- | --- |
| `object-path` | stream path contains embedding/object/OLE/binary-object naming such as `EmbedItems`, `Embedding`, `JSFart`, `CompObj`, `Ole`, `Object`, or `Bin` |
| `image-path` | stream path contains image-oriented naming such as `Image`, `Picture`, `Graphic`, `PNG`, `JPEG`, `BMP`, `WMF`, or `EMF` |
| `shape-path` | stream path contains shape/layout naming such as `Figure`, `Shape`, `Draw`, `Frame`, `LayoutBox`, or `SVG` |
| `table-path` | stream path contains table/cell naming, excluding position/style table names |
| `so-marker` | payload contains the preserved `SO\0\0` object/control marker family |
| `image-signature` | payload contains a recognizable binary image signature such as PNG, JPEG, GIF, TIFF, BMP-at-start, or placeable WMF |
| `svg-signature` | payload contains textual `<svg` evidence |

Output rows preserve the stream path, stream size, reason list, first image signature offsets, first SVG offsets, first SO marker offsets, a short payload prefix, and `decoded=false`.

## Current Sweep

The command was swept across the current 61 local `.jtd`, `.jtt`, and `.jttc` samples.

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

- `ichitaro-20030706232401-success-001-success_data-kazoku_ryoko.jtd` exposes `/EmbedItems`, `/Figure`, `/FigureData`, `/Frame`, and `/LayoutBox` candidates. It also preserves `SO\0\0` hits in `/Figure` and `/PaperMark`.
- `ichitaro-20030316043238-success-001-success_data-iwata_file.jtd` exposes JPEG signatures inside `/EmbedItems/Embedding */Contents` streams, with offsets such as `jpeg@67` and `jpeg@72`.
- `ichitaro-20030422210439-success-002-success_data-natsu.jtd` exposes 12 image-signature rows, mostly JPEG signatures inside embedded `Contents` or `EmbeddedPress` streams.
- `ichitaro-20030829031540-success-004-success_data-hyo.jtd` exposes no object-stream candidates by path, SO marker, image signature, or SVG signature. Its visible table content likely depends on `/DocumentText` control/record decoding plus layout/style streams, not a named table object stream.

## Interpretation

Image recovery is now testable through stream/path candidates and binary image signatures. Several samples have strong embedded JPEG evidence inside `EmbedItems` streams, often after a short object header.

Shape and layout-object recovery should start with `/Figure`, `/FigureData`, `/Frame`, and `/LayoutBox` families. These are path-level candidates, not decoded geometry.

Table recovery should not rely on named CFB table streams in the current corpus. The zero `table-path` result, especially in the `hyo` sample, suggests that table structure is more likely encoded in `/DocumentText` controls, style streams, or layout mark records.

## Model Preservation

The parser now promotes these stream observations into decoded-false model evidence as top-level `objectStreamCandidates`.

JSON export and app-core `getDocumentInfo` expose each candidate with:

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

Each `imagePayloads` row records `kind`, `mime`, `signatureOffset`, `start`, `end`, `length`, `complete`, `objectEnvelope`, `payloadPrefixHex`, and `decoded:false`. The document model also retains the payload bytes internally so a future renderer can consume image resources through the model instead of reopening raw streams from an exporter.

The `objectEnvelope` field preserves the undecoded bytes around each payload: header start/end/length, header prefix, trailer start/end/length, trailer prefix, and a conservative `declaredPayloadLength` candidate when the final four header bytes exactly match the detected payload length as little-endian `u32`. It also exposes decoded-false `headerFields`: prefix `u16LePrefix`/`u32LePrefix` numeric candidates and a `sourcePathCandidate` when the header carries a path-length byte followed by a NUL-terminated embedded source path. This is evidence, not decoded Ichitaro object geometry.

The candidate-level `ownershipCandidate` is also decoded-false. It is derived only from the stream path and records a `stream-path` basis, family, optional storage path, optional `embeddingIndex`, and stream role. Examples include `EmbedItems` contents/embedded-press streams, `FigureData` `FDMVector`, and root figure/frame/layout streams. It does not prove page placement or final object geometry.

The candidate-level `ownershipReferences` field is decoded-false cross-stream evidence. It is currently attached only to embedded image candidates with a path-derived `Embedding N` owner and records byte-pattern matches for that `N` in `FigureData`, `/Figure`, `/Frame`, `/LayoutBox`, `/PageMark`, and `/PaperMark` streams. Each row records `targetPath`, `encoding`, `totalMatches`, a bounded `offsets` preview, and `decoded:false`. These rows prove that a candidate embedding index is observed elsewhere in object/layout-related streams; they do not yet identify the authoritative record field or page geometry.

`getValidationWarnings` reports these entries as `JtdObjectStreamCandidateDiagnosticOnly`.

A JSON export sweep across the same 61 local samples succeeds with 0 failures and preserves 933 `objectStreamCandidates` across 43 files. The sweep exposes 17 files with image-signature candidates, 4 files with SO-marker candidates, 0 table-path files, and 0 SVG-signature files. This matches the diagnostic CLI distribution while keeping the evidence inside the document model.

A second JSON export sweep over the model-preserved `imagePayloads` field also succeeds across all 61 local samples with 0 failures. It finds 14 files with complete payload spans, 94 complete payloads total, and 373,466 preserved payload bytes: 62 JPEG rows across 8 files, 31 GIF89a rows across 9 files, and 1 GIF87a row in 1 file.

The same sweep preserves 94 object envelopes. Six rows expose a matching little-endian declared payload length, all currently in `Embedding */Contents` JPEG streams; large `FDMVector` and `EmbeddedPress` wrapper streams remain envelope-only until their nested object records are decoded.

Header field candidate sweep results: all 61 local samples export successfully, 66 payload rows expose a `sourcePathCandidate`, and every source-path candidate is currently in a `Contents` stream. The path extensions split into 34 `jpg` and 32 `gif` rows. The first little-endian prefix word is `9` in 59 rows and `4` in 6 rows, matching the dominant `09 00 01 00` and secondary `04 00 01 00` header families observed in embedded image contents.

Ownership candidate sweep results: all 61 local samples export successfully, 474 of 933 object stream candidates expose a path-derived `ownershipCandidate`, and all 94 image payload rows are covered by one. Payload rows split by role as `contents` 67, `embedded-press` 8, and `fdm-vector` 19. Candidate families include `embed-items` 335, `figure-data` 38, `figure` 31, `frame` 42, and `layout-box` 28.

Ownership reference sweep results: all 61 local samples export successfully, 12 files expose cross-stream reference candidates, 56 embedded image candidates have `ownershipReferences`, and the model preserves 646 reference rows with 10,055 total byte matches. Reference rows split by target family as `frame` 212, `figure-data` 140, `page-mark` 117, `paper-mark` 80, `layout-box` 67, and `figure` 30; by encoding as `u16-be` 195, `u16-le` 184, `u32-be` 150, and `u32-le` 117. The covered source candidates are still all `embed-items` rows (`contents` 52 and `embedded-press` 4), and 75 of 94 image payload rows now have cross-stream reference evidence.

`rjtd object-ownership-references <file>` expands those model-owned references into match-context diagnostics. For each reported preview offset it prints source stream, target stream, encoding, offset, total matches for that reference row, mod2/mod4 alignment, local context hex, and le/be 16/32-bit values read at the match offset. This command is diagnostic-only and does not create renderable geometry.

The 61-sample `object-ownership-references` sweep succeeds with 0 failures and reports 3,273 preview-offset rows across the same 12 files. Target-family rows split as `figure-data` 1,021, `frame` 941, `layout-box` 538, `page-mark` 502, `paper-mark` 158, and `figure` 113. Reported offsets are not uniformly aligned: mod2 `0` 1,401 vs `1` 1,872, and mod4 `0` 728, `1` 855, `2` 673, `3` 1,017. At the match offset, `u16-be` and `u16-le` rows expose the embedding index directly as the corresponding 16-bit value; `u32-le` rows expose it in the low 16 bits. This narrows later record-field analysis but still does not identify the authoritative geometry field.

`rjtd object-ownership-reference-fields <file>` projects those same preview offsets onto candidate record strides. For each target path, encoding, stride, and field offset it summarizes match count, source count, embedding index set, row-index preview, and cross-row match count. The command intentionally tests every reported offset against a fixed stride set (`4,8,12,16,20,24,28,32,36,40,44,48,52,56,60,64,68,72,80,84`), so the output is a hypothesis surface rather than a decoded record table.

The 61-sample `object-ownership-reference-fields` sweep succeeds with 0 failures and reports 33,492 projected field groups across the same 12 files. The strongest cross-row-free candidates with stride >= 12 are currently `frame/u16-le/12/5` with 106 weighted matches across 12 files, `frame/u16-be/12/7` with 95 across 9 files, and `frame/u16-be/20/15` with 74 across 9 files. This suggests the next analysis should focus on frame records, but it still does not prove which stride or field is semantically authoritative.

`rjtd object-frame-reference-records <file>` expands the strongest frame projections into candidate row bytes. It currently reports only the three strongest decoded-false projection families (`u16-le/12/5`, `u16-be/12/7`, and `u16-be/20/15`) and prints source stream, embedding index, target stream, encoding, stride, field offset, match offset, row index, row start, row hex, BE/LE 16-bit field views, and BE/LE 32-bit field views.

The 61-sample `object-frame-reference-records` sweep succeeds with 0 failures and expands 275 rows across 12 files: `u16-le/12/5` 106 rows, `u16-be/12/7` 95 rows, and `u16-be/20/15` 74 rows. The rows expose repeated byte families such as `00010000000N000000020001`-style 12-byte rows and `00000000010200380000000N` rows. These families are promising evidence for frame-record analysis, but they are not yet decoded placement geometry or paint operations.

`rjtd object-frame-record-families <file>` groups those expanded rows into named decoded-false diagnostic families. The names are observation buckets, not specification terms: for example, `frame-index-flag-row12` captures 12-byte rows whose big-endian word view has small trailing flag-like fields, while `frame-index-tail-coordinate-row12` and `frame-index-tail-window20` capture the repeated tail-window shapes seen in `/Frame` reference projections.

The 61-sample `object-frame-record-families` sweep succeeds with 0 failures and groups the same 275 records across 12 files. Family counts are `frame-index-tail-coordinate-row12` 69, `frame-index-tail-window20` 69, `frame-index-mixed-row12` 61, `frame-index-flag-row12` 45, `frame-index-tail-zero-row12` 24, `frame-index-mixed-window20` 5, and `frame-index-tail-mixed-row12` 2. The next promotion step should compare these row families against page/layout marks and image payload ownership before treating any field as authoritative geometry.

`rjtd object-frame-row-links <file>` checks whether expanded 20-byte frame windows contain a matching 12-byte frame row as their suffix. This is intended to distinguish independent record candidates from context windows around a smaller record.

The 61-sample `object-frame-row-links` sweep succeeds with 0 failures. It finds 74 row20 windows in 9 files; 69 link to a same-source 12-byte suffix row and 5 remain unlinked. Every linked row is `frame-index-tail-window20 -> frame-index-tail-coordinate-row12`. This strongly suggests that the `u16-be/20/15` `tail-window20` projection is usually a context window around the `u16-be/12/7` row rather than an independent authoritative record. The unlinked rows are all `frame-index-mixed-window20`, so they should stay separate until more object/header evidence explains them.

Parser/export JSON now preserves this evidence as `objectStreamCandidates[].frameReferenceRows`. Each row records `targetPath`, `encoding`, `stride`, `fieldOffset`, match `offset`, `rowIndex`, `rowStart`, family, raw `rowHex`, optional `suffixLink`, and `decoded:false`. This keeps later image-placement work model-first: exporters should consume these model-owned rows rather than scanning raw `/Frame` streams directly.

The 61-sample JSON export sweep succeeds with 0 failures and preserves `frameReferenceRows` in the same 12 positive files. It exports 275 rows and 69 suffix links; family counts match the CLI family sweep exactly.

`rjtd object-image-frame-candidates <file>` summarizes this evidence from the image payload source point of view. For each source with complete image payload spans, it reports the path-derived embedding index, payload kinds, total frame rows, row-family counts, row12 coordinate-looking candidates, row20 suffix-link counts, LE row12 counts, a diagnostic preferred bucket, and coordinate-looking row12 pairs. The command is still decoded-false: the preferred bucket is an investigation priority, not renderable geometry.

The 61-sample `object-image-frame-candidates` sweep succeeds with 0 failures. It finds 14 files with image payload sources, 60 image sources, 56 frame-linked sources, 4 sources without `/Frame` rows, and the same 275 frame rows. Diagnostic preferred buckets split as `row12-tail-coordinate` 27, `row12-tail-zero` 8, `u16-le-row12` 20, and `none` 5. Most `none` rows are `FDMVector` sources without a path-derived `Embedding N`, and the `u16-le-row12` bucket is mostly index/flag-like rather than coordinate-looking. Therefore `row12-tail-coordinate` is a strong placement-analysis candidate, but it is not sufficient coverage for PDF image rendering.

`rjtd object-fdm-index <file>` inspects `/FigureData/*/FDMIndex` streams against their sibling `/FigureData/*/FDMVector` streams. The current observed row shape is a 20-byte header followed by 22-byte rows carrying a big-endian vector offset, a 16-bit kind field, and four big-endian signed bbox-like fields. The command links each row to the vector segment ending at the next greater vector offset and reports segment image signatures as decoded-false evidence.

The 61-sample `object-fdm-index` sweep succeeds with 0 failures. It finds 31 files with indexes, 39 index streams, 417 parsed rows, 6 rows with image signatures, 13 image hits, and 2 missing sibling vectors. This proves `FDMIndex`/`FDMVector` is a separate object-placement evidence path from `Embedding N` `/Frame` rows.

`rjtd object-fdm-index-shape <file>` separates those raw row projections into shape families. It distinguishes exact 22-byte tables, declared-count prefix tables followed by auxiliary payload bytes, mixed declared rows, unknown-header streams, and missing sibling vectors.

The 61-sample `object-fdm-index-shape` sweep succeeds with 0 failures. It finds 39 indexes, 35 `fdm-index-v1` headers, 4 unknown headers, and 34 plausible declared counts. The raw whole-stream 22-byte projection has 417 rows and 252 invalid offsets, but the declared-count prefix projection has 147 rows, 43 invalid offsets, and the same 13 image hits. Shape counts are `row22-count-prefix` 17, `row22-exact` 14, `row22-mixed-declared` 3, `unknown-header` 3, and `missing-vector` 2. This means many previously invalid rows are likely auxiliary payload bytes after the FDMIndex table, not real FDMIndex rows.

`rjtd object-fdm-index-rows <file>` prints a row-level diagnostic view for those tables. It reports each row's scope (`declared`, `post-declared`, or `raw`), role (`vector-segment`, `coordinate-like-invalid`, or `invalid-vector-offset`), BE16/i16 field views, raw row bytes, and segment image signatures. The role is decoded-false: `coordinate-like-invalid` means the row resembles signed-coordinate data when viewed as BE16 fields, not that its semantic record type is decoded.

The 61-sample `object-fdm-index-rows` sweep also succeeds with 0 failures. It finds 31 files with indexes, 39 indexes, 417 rows, 147 declared rows, 253 post-declared rows, 17 raw rows, 165 valid vector rows, 252 invalid rows, 13 image hits, and 2 missing vectors. Role counts are `vector-segment` 165, `coordinate-like-invalid` 231, and `invalid-vector-offset` 21. All 43 declared invalid rows are `coordinate-like-invalid`, concentrated in three files, and none of those invalid declared rows are image-bearing vector segments.

Parser/export/app-core JSON now preserves the declared-count prefix rows as `objectStreamCandidates[].fdmIndexEntries` on the corresponding `FDMVector` candidate. Each row records `indexPath`, `vectorPath`, row/index/vector offsets, vector segment length, `kind`/`kindHex`, bbox-like fields, `validVectorOffset`, vector prefix, absolute image signatures, segment-relative image signatures, and `decoded:false`.

The 61-sample JSON export sweep succeeds with 0 failures and preserves 147 `fdmIndexEntries` in 30 candidates across 24 files. Three files have image-linked rows: 6 rows contain 13 image hits. The sweep also reports 104 valid vector offsets and 43 invalid or out-of-range vector offsets, so this evidence identifies the currently observed image-bearing FDMVector segments while reducing false auxiliary rows. It must still not be promoted to renderable page geometry or paint resources.

`rjtd object-fdm-image-candidates <file>` summarizes the image-bearing subset of model-owned `fdmIndexEntries`. For each row with segment image signatures, it reports the FDMVector source, FDMIndex source, row/vector offsets, `kind`, raw and normalized bbox-like fields, bbox order, bbox plausibility, segment image hits, complete image payload count, and `renderable:false` with reason `page-placement-unproven`.

The 61-sample `object-fdm-image-candidates` sweep succeeds with 0 failures. It finds 3 files with candidates, 3 FDM sources, 6 image-bearing rows, 13 image hits, 11 complete payloads, 5 plausible bbox rows, and 0 renderable rows. `shanai_lan` and `tounyou` contain the 5 complete/plausible rows. The finance sample contains 2 JPEG signatures but 0 complete payloads and an implausible bbox, so it remains signature-only diagnostic evidence.

App-core `getPageOverlayImages` exposes the same FDM rows as `unplacedDiagnostics` while keeping `behind`, `front`, and `imageCount` empty/zero. Each diagnostic has `placementProven:false`, `renderable:false`, `decoded:false`, and reason `page-placement-unproven`. This keeps the rhwp-shaped overlay API callable without claiming decoded page placement or paint resources.

Parser/export/app-core JSON also preserves `/Frame` fixed 60-byte records as decoded-false `objectFrameRecords`. The observed record layout has a 16-byte header, a big-endian declared count at offset 14, and 60-byte rows. The rows expose an object id, record kind/type, and geometry-looking fields, but these fields remain diagnostic until units, page association, and paint order are proven.

`rjtd object-fdm-frame-links <file>` correlates image-bearing FDMIndex rows with those `/Frame` records by `fdm row index == frame object id`. The 61-sample sweep succeeds with 0 failures and finds 3 positive files, 6 FDM image rows, 6 frame-linked rows, 0 missing-frame rows, 11 complete payloads, and 0 renderable rows. This proves that the currently observed FDM image rows have a frame-record trail, but it still does not prove which image payload should be painted where on the page.

## Next Work

- Decode the semantic object header fields preceding image payload signatures and connect them to `/Figure`, `/Frame`, `/LayoutBox`, and layout mark evidence.
- Decode `/Frame` geometry units, page association, paint order, payload-to-image selection, and the remaining coordinate-like FDMIndex diagnostic rows before rendering FDMVector images.
- Prove which `Embedding N` reference encoding and record-local offset is semantically authoritative before promoting ownership references into page geometry.
- Connect preserved image payload bytes to model-level image resources only after object ownership and page geometry are proven.
- Build real page/layer paint operations from decoded object and layout records before adding non-text PDF rendering.
- Investigate table semantics through `/DocumentText` control ranges and layout/style streams rather than stream-name matching.
