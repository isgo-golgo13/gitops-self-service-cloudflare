# Root: the collection of app stacks (for_each), the zones they live in, and one
# zone baseline per zone. Requests are one JSON file per app under requests/, read
# and merged here (OpenTofu does not deep-merge variables across *.auto.tfvars files).

locals {
  request_files = fileset("${path.module}/requests", "*.json")
  apps          = merge([for f in local.request_files : jsondecode(file("${path.module}/requests/${f}")).apps]...)
}

data "cloudflare_zone" "this" {
  for_each = toset(distinct([for a in values(local.apps) : a.zone]))
  filter = {
    name = each.value
  }
}

module "app" {
  source   = "../../modules/app-stack"
  for_each = local.apps

  spec = merge({
    database = "none", objects = false, container = false, origins = []
  }, each.value, { name = each.key })
  account_id            = var.account_id
  zone_id               = data.cloudflare_zone.this[each.value.zone].id
  region                = var.region
  allowed_email_domains = var.allowed_email_domains
  postgres_origin       = try(each.value.database, "none") == "postgres" ? var.postgres_origins[each.key] : null
}

# One baseline per zone: managed WAF + a rate-limit rule per app hostname.
module "zone_baseline" {
  source   = "../../modules/cf-zone-baseline"
  for_each = data.cloudflare_zone.this

  zone_id     = each.value.id
  rate_limits = { for name, app in module.app : app.hostname => app.rate_limit_rpm if local.apps[name].zone == each.key }
}
