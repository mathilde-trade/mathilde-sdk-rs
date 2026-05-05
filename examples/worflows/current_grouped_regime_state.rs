// {
//             "question": "What is the current grouped BTC regime state, and where did a materially similar grouped regime state occur before?",
//             "why": "This workflow stays on decomposed market-state measurement rather than on raw bar truth or generic primitive outputs.",
//             "steps": [
//                 {
//                     "use": "If the relevant regime families or groups are not known yet, open Regime taxonomy first.",
//                     "route": "GET https://regime.api.mathilde.dev/v1/docs/taxonomy",
//                     "retrieve": "The family-and-group space needed before building the grouped regime predicate."
//                 },
//                 {
//                     "use": "If deeper algorithm meaning is still needed, open Regime registry.",
//                     "route": "GET https://regime.api.mathilde.dev/v1/docs/registry",
//                     "retrieve": "The deeper regime algorithm and shipped-output discovery surface."
//                 },
//                 {
//                     "use": "Call Regime latest for BTC on the defended 1h lane.",
//                     "route": "POST https://regime.api.mathilde.dev/v1/outputs/latest",
//                     "retrieve": "The newest stable grouped regime outputs row for BTC on the supported timeframe."
//                 },
//                 {
//                     "use": "Call Regime search with the measured grouped-state predicate.",
//                     "route": "POST https://regime.api.mathilde.dev/v1/outputs/search",
//                     "retrieve": "Historical timestamps where a materially similar grouped regime state was true."
//                 },
//                 {
//                     "use": "Call Regime time-machine on the matched timestamps.",
//                     "route": "POST https://regime.api.mathilde.dev/v1/outputs/time-machine",
//                     "retrieve": "Local grouped regime context before and after each matched historical moment."
//                 }
//             ],
//             "stop_when": "The current grouped regime state, the matched historical moments, and the replay context around those moments are all retrieved.",
//             "non_goal": "Do not turn grouped historical similarity into a prediction claim."
//         }

fn main() {}
