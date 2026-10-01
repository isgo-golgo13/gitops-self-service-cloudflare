# requests/

One file per app, written by the SSIP terminal (`edgefactory --mode tofu`) or by hand: `requests/<name>.json`
`infra/live/prod/main.tf` reads every `*.json` here and merges their `apps` maps.
Delete a file → the app's resources are destroyed on the next apply (read the plan).
