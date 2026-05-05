// // "understanding_workflow": [
//         {
//             "step": 1,
//             "surface": "Aggregator summary",
//             "route": "GET https://aggregator.api.mathilde.dev/v1/docs/summary",
//             "question": "What problem does MATHILDE solve at the bar-truth layer?",
//             "why_now": "Start here because it is the shortest correct orientation to stable boundaries, canonical minute truth, and why raw market streams are not yet a safe dataset.",
//             "do_not_start_with": "Do not start with OpenAPI, taxonomy, or registry before this orientation exists."
//         },
//         {
//             "step": 2,
//             "surface": "Aggregator system",
//             "route": "GET https://aggregator.api.mathilde.dev/v1/docs/system",
//             "question": "How does Aggregator turn imperfect external streams into bounded, safe-to-serve bars?",
//             "why_now": "This is the conceptual foundation for every downstream surface.",
//             "do_not_start_with": "Do not move to computed outputs before the upstream bar-truth contract is clear."
//         },
//         {
//             "step": 3,
//             "surface": "Aggregator endpoints",
//             "route": "GET https://aggregator.api.mathilde.dev/v1/docs/endpoints",
//             "question": "Now that bars are understood, which read family retrieves them correctly?",
//             "why_now": "Endpoint family choice becomes meaningful only after the measured object is understood.",
//             "do_not_start_with": "Do not use route names alone to infer family meaning."
//         },
//         {
//             "step": 4,
//             "surface": "Primitives summary",
//             "route": "GET https://primitives.api.mathilde.dev/v1/docs/summary",
//             "question": "What is a primitives outputs row in MATHILDE terms?",
//             "why_now": "Primitives is downstream of Aggregator, so it should be read only after bar truth is clear.",
//             "do_not_start_with": "Do not begin selector discovery before the outputs row itself is understood."
//         },
//         {
//             "step": 5,
//             "surface": "Primitives system",
//             "route": "GET https://primitives.api.mathilde.dev/v1/docs/system",
//             "question": "What counts as a primitive measurement, and why are outputs grouped this way?",
//             "why_now": "This document explains the conceptual model behind the grouped outputs surface.",
//             "do_not_start_with": "Do not jump into taxonomy or registry before this model is clear."
//         },
//         {
//             "step": 6,
//             "surface": "Primitives taxonomy",
//             "route": "GET https://primitives.api.mathilde.dev/v1/docs/taxonomy",
//             "question": "Which primitive families and groups exist?",
//             "why_now": "Taxonomy is the selector-space map. It should narrow the search space before deeper algorithm research.",
//             "do_not_start_with": "Do not treat taxonomy as an onboarding narrative. It is a discovery payload."
//         },
//         {
//             "step": 7,
//             "surface": "Primitives registry",
//             "route": "GET https://primitives.api.mathilde.dev/v1/docs/registry",
//             "question": "Which exact primitive algorithms and shipped outputs exist inside the selected families?",
//             "why_now": "Registry is large and only becomes legible after taxonomy has narrowed the space.",
//             "do_not_start_with": "Do not read the full registry cold if the relevant families are still unknown."
//         },
//         {
//             "step": 8,
//             "surface": "Primitives endpoints",
//             "route": "GET https://primitives.api.mathilde.dev/v1/docs/endpoints",
//             "question": "Given a known outputs object and selector space, which read family should be called?",
//             "why_now": "This is the correct time to convert conceptual understanding into retrieval routing.",
//             "do_not_start_with": "Do not begin with endpoints before the outputs object and selector space are understood."
//         },
//         {
//             "step": 9,
//             "surface": "Regime summary",
//             "route": "GET https://regime.api.mathilde.dev/v1/docs/summary",
//             "question": "What is Regime measuring at a high level?",
//             "why_now": "Regime is easier to place after lower-level bars and primitives surfaces are already clear.",
//             "do_not_start_with": "Do not start here if bar truth and primitive outputs are still conceptually unclear."
//         },
//         {
//             "step": 10,
//             "surface": "Regime system",
//             "route": "GET https://regime.api.mathilde.dev/v1/docs/system",
//             "question": "How is the fixed family-and-question matrix organized, and why is Regime 1h only?",
//             "why_now": "This is the conceptual contract for Regime as a measured state system.",
//             "do_not_start_with": "Do not infer the matrix from route names alone."
//         },
//         {
//             "step": 11,
//             "surface": "Regime taxonomy",
//             "route": "GET https://regime.api.mathilde.dev/v1/docs/taxonomy",
//             "question": "Which dimensions and question slots exist inside the Regime matrix?",
//             "why_now": "Taxonomy makes the matrix machine-readable after the system-level explanation is clear.",
//             "do_not_start_with": "Do not use taxonomy as the first narrative explanation of Regime."
//         },
//         {
//             "step": 12,
//             "surface": "Regime registry",
//             "route": "GET https://regime.api.mathilde.dev/v1/docs/registry",
//             "question": "Which exact Regime kernels, questions, and shipped fields exist?",
//             "why_now": "Registry is the deeper kernel-level discovery surface after taxonomy has narrowed the search space.",
//             "do_not_start_with": "Do not read the full registry cold before the matrix and taxonomy are known."
//         },
//         {
//             "step": 13,
//             "surface": "Regime endpoints",
//             "route": "GET https://regime.api.mathilde.dev/v1/docs/endpoints",
//             "question": "Now that the Regime matrix is understood, which read family retrieves it correctly?",
//             "why_now": "This is the final routing step for the Regime surface itself.",
//             "do_not_start_with": "Do not start with endpoints before understanding what Regime is measuring."
//         },
//         {
//             "step": 14,
//             "surface": "OpenAPI",
//             "route": "GET https://aggregator.api.mathilde.dev/openapi.json, GET https://primitives.api.mathilde.dev/openapi.json, GET https://regime.api.mathilde.dev/openapi.json",
//             "question": "What is the exact transport and schema contract once the object and family are already known?",
//             "why_now": "OpenAPI is exact and useful, but it is a transport contract, not an onboarding explanation.",
//             "do_not_start_with": "Do not start here if the measured object or endpoint family is still unclear."
//         }
//     ],

fn main() {}
