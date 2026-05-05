# SDK Live Public Surface Verification Report

- execution_timestamp_utc: `2026-05-05T09:00:54Z`
- final_status: `live_public_surface_checks_passed`

## Configuration Summary

- http_base_url: `https://aggregator.api.mathilde.dev`
- grpc_base_url: `https://aggregator.grpc.mathilde.dev`
- ws_base_url: `wss://aggregator.api.mathilde.dev`
- bearer_token_present: `true`
- intro_bearer_token_present: `true`

## Surface Results

| Family | Surface | Status | Note |
| --- | --- | --- | --- |
| `intro` | `intro.intro` | `passed` | subsystem=intro keys=14 |
| `docs` | `docs_system` | `passed` | subsystem=aggregator sections=24 |
| `docs` | `docs_themes` | `passed` | subsystem=aggregator themes=13 |
| `docs` | `docs_endpoints` | `passed` | subsystem=feed_public_endpoint_usage sections=8 |
| `docs` | `openapi` | `passed` | paths=17 |
| `primitives_docs` | `primitives.docs_system` | `passed` | subsystem=primitives_processor keys=5 |
| `primitives_docs` | `primitives.docs_summary` | `passed` | subsystem=summary keys=5 |
| `primitives_docs` | `primitives.docs_taxonomy` | `passed` | keys=6 |
| `primitives_docs` | `primitives.docs_registry` | `passed` | keys=5 |
| `primitives_docs` | `primitives.docs_endpoints` | `passed` | keys=5 |
| `primitives_docs` | `primitives.openapi` | `passed` | openapi=3.1.0 paths=18 |
| `regime_docs` | `regime.docs_system` | `passed` | subsystem=regime keys=5 |
| `regime_docs` | `regime.docs_summary` | `passed` | subsystem=summary keys=5 |
| `regime_docs` | `regime.docs_taxonomy` | `passed` | keys=6 |
| `regime_docs` | `regime.docs_registry` | `passed` | keys=5 |
| `regime_docs` | `regime.docs_endpoints` | `passed` | keys=5 |
| `regime_docs` | `regime.openapi` | `passed` | keys=5 |
| `discovery` | `pairs_status` | `passed` | pair=ADAUSDT rows=1 |
| `discovery` | `pairs_list` | `passed` | pairs=5 |
| `discovery` | `files_downloads` | `passed` | rows=2681 first_pair=ADAUSDT expires_at_utc=2026-05-05T09:05:54Z |
| `bars_http` | `latest` | `passed` | pair=ADAUSDT close_end_ms=1777971600000 |
| `bars_http` | `range` | `passed` | rows=5 close_end_ms=1777971600000 |
| `bars_http` | `search` | `passed` | hits=20 predicate=ADAUSDT.close > 0 |
| `bars_http` | `time_machine` | `passed` | rows=5 |
| `bars_grpc` | `latest_grpc` | `passed` | pair=ADAUSDT close_end_ms=1777971600000 |
| `bars_grpc` | `range_grpc` | `passed` | rows=5 |
| `bars_grpc` | `search_grpc` | `passed` | hits=20 |
| `bars_grpc` | `time_machine_grpc` | `passed` | rows=5 |
| `ws` | `connect_bars_ws` | `passed` | window_s=120 meta_count=5 payload_frames=5 payload_rows=4 last_close_ms=1777971720000 |
| `ws` | `connect_messages_ws` | `passed` | window_s=120 subscribed=true message_count=2 heartbeat_count=1 last_close_ms=1777971840000 |
| `ws_optional` | `connect_bars_ws_recovering` | `passed` | window_s=30 reconnect_count=2 pre_reconnect_frames=2 post_reconnect_frames=5 last_close_ms=1777971900000 |
| `ws_optional` | `connect_messages_ws_recovering` | `passed` | window_s=120 reconnect_count=2 subscribed_before_reconnect=true subscribed_after_reconnect=true pre_reconnect_messages=1 post_reconnect_messages=1 last_close_ms=1777972020000 |
| `unknown` | `docs_summary` | `passed` | subsystem=summary sections=13 |

## Proved Observations

- constructed `Aggregator` from checked-in public defaults without introducing new environment variables
- constructed `Primitives` from checked-in public defaults without introducing new environment variables
- constructed `Regime` from checked-in public defaults without introducing new environment variables
- constructed `Intro` from checked-in public defaults without introducing new environment variables
- Markdown report directory and per-surface scaffold were initialized
- `intro.intro` passed: subsystem=intro keys=14
- `docs_system` passed: subsystem=aggregator sections=24
- `docs_summary` passed: subsystem=summary sections=13
- `docs_themes` passed: subsystem=aggregator themes=13
- `primitives.docs_system` passed: subsystem=primitives_processor keys=5
- `primitives.docs_summary` passed: subsystem=summary keys=5
- `primitives.docs_taxonomy` passed: keys=6
- `primitives.docs_registry` passed: keys=5
- `primitives.docs_endpoints` passed: keys=5
- `primitives.openapi` passed: keys=5
- `regime.docs_system` passed: subsystem=regime keys=5
- `regime.docs_summary` passed: subsystem=summary keys=5
- `regime.docs_taxonomy` passed: keys=6
- `regime.docs_registry` passed: keys=5
- `regime.docs_endpoints` passed: keys=5
- `regime.openapi` passed: keys=5
- `primitives.openapi` passed: openapi=3.1.0 paths=18
- `docs_endpoints` passed: subsystem=feed_public_endpoint_usage sections=8
- `openapi` passed: paths=17
- `pairs_list` passed: pairs=5
- `pairs_status` passed: pair=ADAUSDT rows=1
- `files_downloads` passed: rows=2681 first_pair=ADAUSDT expires_at_utc=2026-05-05T09:05:54Z
- `latest` passed: pair=ADAUSDT close_end_ms=1777971600000
- `range` passed: rows=5 close_end_ms=1777971600000
- `search` passed: hits=20 predicate=ADAUSDT.close > 0
- `time_machine` passed: rows=5
- `latest_grpc` passed: pair=ADAUSDT close_end_ms=1777971600000
- `range_grpc` passed: rows=5
- `search_grpc` passed: hits=20
- `time_machine_grpc` passed: rows=5
- `connect_bars_ws` passed: window_s=120 meta_count=5 payload_frames=5 payload_rows=4 last_close_ms=1777971720000
- `connect_messages_ws` passed: window_s=120 subscribed=true message_count=2 heartbeat_count=1 last_close_ms=1777971840000
- `connect_bars_ws_recovering` passed: window_s=30 reconnect_count=2 pre_reconnect_frames=2 post_reconnect_frames=5 last_close_ms=1777971900000
- `connect_messages_ws_recovering` passed: window_s=120 reconnect_count=2 subscribed_before_reconnect=true subscribed_after_reconnect=true pre_reconnect_messages=1 post_reconnect_messages=1 last_close_ms=1777972020000
- WS validation phase lasted 390 seconds across raw and recovering checks

## Failures

- none

## Skipped Checks

- none

## Final Status

`live_public_surface_checks_passed`
