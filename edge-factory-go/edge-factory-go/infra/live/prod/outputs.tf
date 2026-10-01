output "apps" {
  description = "Per-app status: hostnames, Worker, data bindings, guards."
  value       = { for name, app in module.app : name => app.status }
}

output "wrangler_bindings" {
  description = "Per-app binding manifests for the app team's wrangler config (make bindings writes them to artifacts/)."
  value       = { for name, app in module.app : name => app.wrangler_bindings }
}
