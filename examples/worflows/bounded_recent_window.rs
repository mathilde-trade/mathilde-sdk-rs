// {
//             "question": "How did BTC bar truth and computed outputs evolve over the last 24 hours?",
//             "why": "This workflow reconstructs a bounded recent window instead of only reading the newest row.",
//             "steps": [
//                 {
//                     "use": "Call Aggregator range for BTC on the target timeframe over the last 24 hours.",
//                     "route": "POST https://aggregator.api.mathilde.dev/v1/bars/range",
//                     "retrieve": "The bounded bar-truth window."
//                 },
//                 {
//                     "use": "Call Primitives range over the same pair, timeframe, and time window.",
//                     "route": "POST https://primitives.api.mathilde.dev/v1/outputs/range",
//                     "retrieve": "The bounded computed-outputs window aligned to the same period."
//                 },
//                 {
//                     "use": "Align the bars and outputs rows by timestamp.",
//                     "route": "local alignment step",
//                     "retrieve": "One recent sequence where market truth and computed measurement are bound together."
//                 },
//                 {
//                     "use": "Read the ordered sequence from oldest to newest.",
//                     "route": "same aligned window",
//                     "retrieve": "How the current measured state formed across the last 24 hours."
//                 }
//             ],
//             "stop_when": "The last-24h window is fully reconstructed as an aligned bars-plus-outputs sequence.",
//             "non_goal": "Do not use this workflow when the real question is unknown-timestamp discovery."
//         },

fn main() {}
