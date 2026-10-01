output "hostname" {
  value = local.hostname
}

output "rate_limit_rpm" {
  description = "Consumed by the zone baseline: one rate-limit rule per hostname."
  value       = local.tier.rate_limit_rpm
}

# What the app team's wrangler config needs: binding names and the IDs behind them.
# The platform owns the shell and the data; the team owns the code.
output "wrangler_bindings" {
  value = {
    name   = module.worker.worker.name
    routes = [{ pattern = local.hostname, custom_domain = true }]
    d1_databases = local.s.database == "d1" ? [{
      binding = "DB", database_name = module.d1[0].database.name, database_id = module.d1[0].database.id
    }] : []
    r2_buckets    = local.s.objects ? [{ binding = "OBJECTS", bucket_name = module.objects[0].bucket.name }] : []
    hyperdrive    = local.s.database == "postgres" ? [{ binding = "PG", id = module.hyperdrive[0].hyperdrive.id }] : []
    containers    = local.s.container ? [{ class_name = "AppContainer", image = "./Containerfile", max_instances = { small = 2, medium = 5, large = 20 }[local.s.tier] }] : []
    observability = { enabled = true, head_sampling_rate = local.tier.observability_sampling }
  }
}

output "status" {
  value = {
    worker        = module.worker.worker
    access        = local.exposure.access ? module.access[0].access : null
    load_balancer = local.exposure.load_balancer ? module.load_balancer[0].load_balancer : null
    d1            = local.s.database == "d1" ? module.d1[0].database : null
    hyperdrive    = local.s.database == "postgres" ? module.hyperdrive[0].hyperdrive : null
    objects       = local.s.objects ? module.objects[0].bucket : null
  }
}
