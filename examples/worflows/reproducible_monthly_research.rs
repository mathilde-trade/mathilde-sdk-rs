// {
//             "question": "How do I build a reproducible monthly BTC 1h research slice that joins stable bars and computed outputs locally?",
//             "why": "This workflow is for offline reproducible analysis rather than interactive live retrieval.",
//             "steps": [
//                 {
//                     "use": "Call Aggregator file-download discovery for the required monthly BTC 1h bars objects.",
//                     "route": "POST https://aggregator.api.mathilde.dev/v1/files/downloads",
//                     "retrieve": "Signed parquet download URLs for the required bars slice."
//                 },
//                 {
//                     "use": "Call Primitives file-download discovery for the matching monthly BTC 1h outputs objects.",
//                     "route": "POST https://primitives.api.mathilde.dev/v1/files/downloads",
//                     "retrieve": "Signed parquet download URLs for the required outputs slice."
//                 },
//                 {
//                     "use": "Download the parquet files.",
//                     "route": "same signed URLs",
//                     "retrieve": "The exact monthly files requested from both surfaces."
//                 },
//                 {
//                     "use": "Join the files locally on canonical row identity.",
//                     "route": "local alignment step",
//                     "retrieve": "A reproducible offline research table with aligned bars and outputs."
//                 },
//                 {
//                     "use": "Preserve the pair, timeframe, month labels, and file identities.",
//                     "route": "local bookkeeping step",
//                     "retrieve": "An auditable research slice that can be rebuilt later."
//                 }
//             ],
//             "stop_when": "The monthly slice is downloaded, joined, and locally reproducible.",
//             "non_goal": "Do not use this workflow when the real question is the newest stable live snapshot."
//         },

fn main() {}
