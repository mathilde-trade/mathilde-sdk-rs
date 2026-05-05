// {
//             "question": "When did BTC show the same measured local-stress condition before, and what was the immediate context around those hits?",
//             "why": "This workflow begins from a concrete measured condition rather than from a fixed historical window.",
//             "steps": [
//                 {
//                     "use": "If the relevant outputs families are not known yet, open Primitives taxonomy first.",
//                     "route": "GET https://primitives.api.mathilde.dev/v1/docs/taxonomy",
//                     "retrieve": "The family and group space needed before building the predicate."
//                 },
//                 {
//                     "use": "If deeper algorithm meaning is still needed, open Primitives registry.",
//                     "route": "GET https://primitives.api.mathilde.dev/v1/docs/registry",
//                     "retrieve": "The deeper algorithm and shipped-output discovery surface."
//                 },
//                 {
//                     "use": "Call Primitives search with the measured local-stress predicate.",
//                     "route": "POST https://primitives.api.mathilde.dev/v1/outputs/search",
//                     "retrieve": "The hit timestamps where that measured condition became true."
//                 },
//                 {
//                     "use": "Call Primitives time-machine on those hits.",
//                     "route": "POST https://primitives.api.mathilde.dev/v1/outputs/time-machine",
//                     "retrieve": "The bounded computed-output context around each hit rather than hit timestamps alone."
//                 },
//                 {
//                     "use": "If the bar path itself must also be inspected, call Aggregator time-machine on the same windows.",
//                     "route": "POST https://aggregator.api.mathilde.dev/v1/bars/time-machine",
//                     "retrieve": "The matching bar-truth context around the same local-stress hits."
//                 }
//             ],
//             "stop_when": "The hit timestamps and their bounded context windows are both retrieved.",
//             "non_goal": "Do not stop at search alone when local context is still required."
//         },

fn main() {}
