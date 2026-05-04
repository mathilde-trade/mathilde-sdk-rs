# Intro ednpoint

we should move if not done teh intro enpoints api.mathilde.dev in a dedicate system intro not in aggertaor

# Full enspint test

we have avialble some good examples for full tests ednspints here
/home/tia/\_DEV/MATHILDE/aggregator/services/feed/src/bin/feed_endpoint_tests.rs
/home/tia/\_DEV/MATHILDE/compute-primitives/services/feed/src/bin/feed_endpoint_tests.rs
/home/tia/\_DEV/MATHILDE/compute-regime/services/feed/src/bin/feed_endpoint_tests.rs

i thikn we should have a coplete enpoints_test where we test all ednpoints of ouor sdk to ensure correctness

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
