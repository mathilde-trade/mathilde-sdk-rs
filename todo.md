# Intro ednpoint

we should move if not done teh intro enpoints api.mathilde.dev in a dedicate system intro not in aggertaor

# Full enspint test

we have avialble some good examples for full tests ednspints here
/home/tia/\_DEV/MATHILDE/aggregator/services/feed/src/bin/feed_endpoint_tests.rs
/home/tia/\_DEV/MATHILDE/compute-primitives/services/feed/src/bin/feed_endpoint_tests.rs
/home/tia/\_DEV/MATHILDE/compute-regime/services/feed/src/bin/feed_endpoint_tests.rs

i thikn we should have a coplete enpoints_test where we test all ednpoints of ouor sdk to ensure correctness

# Build time problem

ia@tia:~/\_DEV/MATHILDE/mathilde-sdk-rs$ cargo expand --lib > expanded.r
s
Checking mathilde-sdk-rs v0.0.5 (/media/Development/MATHILDE/mathilde-sdk-rs)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.41s

tia@tia:~/\_DEV/MATHILDE/mathilde-sdk-rs$ wc -l expanded.rs
649408 expanded.rs
tia@tia:~/\_DEV/MATHILDE/mathilde-sdk-rs$ du -h expanded.rs
40M expanded.rs
tia@tia:~/\_DEV/MATHILDE/mathilde-sdk-rs$ grep -n "impl.\*Serialize" expanded.rs | wc -l

grep -n "impl.\*Deserialize" expanded.rs | wc -l

grep -n "ToSchema" expanded.rs | wc -l

grep -n "prost::" expanded.rs | wc -l

grep -n "tonic::" expanded.rs | wc -l
216
426
0
6469
29
tia@tia:~/\_DEV/MATHILDE/mathilde-sdk-rs$ grep -n "impl.*Serialize for" expanded.rs | head -40
grep -n "impl.*Deserialize.\*for" expanded.rs | head -40
1189: impl \_serde::Serialize for TimeInput {
9261: impl \_serde::Serialize for ExcludedSourceCountV1 {
9675: impl \_serde::Serialize for OutputsCursorV1 {
10086: impl \_serde::Serialize for OutputProcessDiagnosticV1 {
12243: impl \_serde::Serialize for OutputBarsMetadataV1 {
14568: impl \_serde::Serialize for OutputMetadataV1 {
49806: impl \_serde::Serialize for OutputRowV1 {
78526: impl \_serde::Serialize for OutputsPresentRowV1 {
78947: impl \_serde::Serialize for OutputsRowsPayloadV1 {
79541: impl \_serde::Serialize for OutputsLatestResponseV1 {
80208: impl \_serde::Serialize for OutputsRangeResponseV1 {
80988: impl \_serde::Serialize for OutputsSearchResponseV1 {
81778: impl \_serde::Serialize for OutputsTimeMachineRowV1 {
82522: impl \_serde::Serialize for OutputsTimeMachineResponseV1 {
83400: impl \_serde::Serialize for OutputsTickEnvelopeV1 {
84020: impl \_serde::Serialize for OutputsViewV1 {
84489: impl \_serde::Serialize for PairNotReadyDetailV1 {
84994: impl \_serde::Serialize for ResyncRequiredDetailV1 {
85712: impl \_serde::Serialize for LatestOutputsRequestV1 {
86940: impl \_serde::Serialize for RangeOutputsRequestV1 {
88345: impl \_serde::Serialize for SearchOutputsRequestV1 {
90004: impl \_serde::Serialize for TimeMachineOutputsRequestV1 {
91058: impl \_serde::Serialize for ProcessorFamily {
92227: impl \_serde::Serialize for ProcessorGroup {
97736: impl \_serde::Serialize for OutputProcessDiagnostic {
101088: impl \_serde::Serialize for ProcessorOutputMin {
130681: impl \_serde::Serialize for OutputBarsMetadata {
132613: impl \_serde::Serialize for OutputMetadata {
136407: impl \_serde::Serialize for ProcessorOutputWithMeta {
170952: impl \_serde::Serialize for ProcessorProjectedOutputMin {
210198: impl \_serde::Serialize for ProcessorProjectedOutputWithMeta {
244451: impl<T: Serialize> Serialize for ProjectedValue<T> {
244628: impl \_serde::Serialize for ExcludedSourceCountV1 {
245042: impl \_serde::Serialize for OutputsCursorV1 {
245453: impl \_serde::Serialize for OutputProcessDiagnosticV1 {
247610: impl \_serde::Serialize for OutputBarsMetadataV1 {
249935: impl \_serde::Serialize for OutputMetadataV1 {
297144: impl \_serde::Serialize for OutputRowV1 {
337061: impl \_serde::Serialize for OutputsPresentRowV1 {
337482: impl \_serde::Serialize for OutputsRowsPayloadV1 {
1219: impl<'de> \_serde::Deserialize<'de> for TimeInput {
9369: impl<'de> \_serde::Deserialize<'de> for **Field {
9713: impl<'de> \_serde::Deserialize<'de> for OutputsCursorV1 {
9786: impl<'de> \_serde::Deserialize<'de> for **Field {
10194: impl<'de> \_serde::Deserialize<'de> for **Field {
12717: impl<'de> \_serde::Deserialize<'de> for **Field {
14646: impl<'de> \_serde::Deserialize<'de> for OutputMetadataV1 {
14779: impl<'de> \_serde::Deserialize<'de> for **Field {
52534: impl<'de> \_serde::Deserialize<'de> for OutputRowV1 {
56749: impl<'de> \_serde::Deserialize<'de> for **Field {
78564: impl<'de> \_serde::Deserialize<'de> for OutputsPresentRowV1 {
78633: impl<'de> \_serde::Deserialize<'de> for **Field {
79055: impl<'de> \_serde::Deserialize<'de> for **Field {
79699: impl<'de> \_serde::Deserialize<'de> for **Field {
80331: impl<'de> \_serde::Deserialize<'de> for **Field {
81181: impl<'de> \_serde::Deserialize<'de> for **Field {
81899: impl<'de> \_serde::Deserialize<'de> for **Field {
82702: impl<'de> \_serde::Deserialize<'de> for **Field {
83552: impl<'de> \_serde::Deserialize<'de> for **Field {
84068: impl<'de> \_serde::Deserialize<'de> for OutputsViewV1 {
84160: impl<'de> \_serde::Deserialize<'de> for **Field {
84623: impl<'de> \_serde::Deserialize<'de> for **Field {
85102: impl<'de> \_serde::Deserialize<'de> for **Field {
85882: impl<'de> \_serde::Deserialize<'de> for **Field {
87155: impl<'de> \_serde::Deserialize<'de> for **Field {
88564: impl<'de> \_serde::Deserialize<'de> for **Field {
90267: impl<'de> \_serde::Deserialize<'de> for **Field {
91234: impl<'de> \_serde::Deserialize<'de> for ProcessorFamily {
91412: impl<'de> \_serde::Deserialize<'de> for **Field {
93955: impl<'de> \_serde::Deserialize<'de> for ProcessorGroup {
95509: impl<'de> \_serde::Deserialize<'de> for **Field {
97774: impl<'de> \_serde::Deserialize<'de> for OutputProcessDiagnostic {
97843: impl<'de> \_serde::Deserialize<'de> for **Field {
103819: impl<'de> \_serde::Deserialize<'de> for ProcessorOutputMin {
108030: impl<'de> \_serde::Deserialize<'de> for **Field {
130861: impl<'de> \_serde::Deserialize<'de> for OutputBarsMetadata {
131154: impl<'de> \_serde::Deserialize<'de> for **Field {
132691: impl<'de> \_serde::Deserialize<'de> for OutputMetadata {
132824: impl<'de> \_serde::Deserialize<'de> for \_\_Field {
139143: impl<'de> \_serde::Deserialize<'de> for ProcessorOutputWithMeta {
tia@tia:~/\_DEV/MATHILDE/mathilde-sdk-rs$

# Readme

README rewrite after `regime` integration:

- intro ednpoint is the enpoint that allow to move accross all mathide is a dexcrptin fo machines and llms in order to naviagte fast inside teh documention of teh entire platform
- after `regime` is added, rewrite README concept-first across systems
- document shared mechanisms first:
  - `latest`
  - `range`
  - `search`
  - `time_machine`
  - files and download flow
  - outputs WS
  - messages WS
  - predicate language
  - docs endpoints as the entrypoint to understand each system
  - registry and taxonomy for output definitions and filtering
  - for regime and primitves add that full famiyl and group descritpins are avialble under rgitry and taxonomy endpoints
- add system-specific examples only after the shared mechanics section is locked
