// "question": "What is the current measured BTC downside state, and when did a materially similar measured downside state occur before?",
//             "why": "This workflow stays on the measurement side. It retrieves the current outputs row, finds historical outputs rows with similar downside structure, and replays local context around those matched moments.",
//             "steps": [
//                 {
//                     "use": "Call Primitives latest.",
//                     "route": "POST https://primitives.api.mathilde.dev/v1/outputs/latest",
//                     "retrieve": "The newest stable computed outputs row for BTC on the target timeframe."
//                 },
//                 {
//                     "use": "Inspect the current downside-related outputs fields.",
//                     "route": "same response",
//                     "retrieve": "The current measured downside structure that will define the historical predicate."
//                 },
//                 {
//                     "use": "Call Primitives search with that predicate.",
//                     "route": "POST https://primitives.api.mathilde.dev/v1/outputs/search",
//                     "retrieve": "Historical timestamps where a materially similar measured downside state was true."
//                 },
//                 {
//                     "use": "Call Primitives time-machine on the matched timestamps.",
//                     "route": "POST https://primitives.api.mathilde.dev/v1/outputs/time-machine",
//                     "retrieve": "Local computed-output context before and after each matched historical moment."
//                 },
//                 {
//                     "use": "If bar-truth context is also needed, call Aggregator time-machine on the same matched windows.",
//                     "route": "POST https://aggregator.api.mathilde.dev/v1/bars/time-machine",
//                     "retrieve": "The bounded bar context around the same matched moments."
//                 }
//             ],
//             "stop_when": "The current measured downside state, the matched historical moments, and the replay context around those moments are all retrieved.",
//             "non_goal": "Do not turn historical similarity into a prediction claim."

fn main() {}
