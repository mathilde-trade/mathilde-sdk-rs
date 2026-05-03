1. add correct domains in config as defaults
   /home/tia/\_DEV/MATHILDE/deploy/dev/manifest.dev.json
   and make teh system autmaic for that
2. check files donwolads our system now expose teh files as donloeadbles link
   it will be nice teh possibility to make teh sdk able to also donaload those links (every link need teh bearer)
   so can be files_download_items([items] as array of strings, "path if omited will be something liek /tmp/mathilde/" )

you can check for example here
http://aggregator.api.mathilde.dev/v1/files/downloads
{
"end_label_utc": "2026-02-21",
"order": "desc",
"pairs": [
"BTCUSDT",
"ETHUSDT"
],
"period": "day",
"start_label_utc": "2026-02-20",
"tfs": [
"1m",
"5m"
]
}

and see teh files list every file can be donwlaod passing teh bearer so

and system will doenlaods teh ssleceted files passing tehcorrect bearer for every file 3) check every aggerator fields passed agisnt correct api and tests it 4) for range, search, time machine we add a \_traverse option that use teh corretc cursor and traverse this must be a shared helprper that all outer sytsem can use since every system have that so maybe will worth add inside the api calers
/home/tia/\_DEV/MATHILDE/mathilde-sdk-rs/src/systems/aggregator/bars_grpc.rs
/home/tia/\_DEV/MATHILDE/mathilde-sdk-rs/src/systems/aggregator/bars_http.rs
a (,, traverse (optional default false))
