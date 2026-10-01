terraform {
  required_version = ">= 1.10.0, < 2.0.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = ">= 5.26.0, < 6.0.0"
    }
  }
}

# The parent Worker resource (Cloudflare's resource-oriented Workers API). It carries
# settings, not code: the app team deploys versions (code + container image) with
# `wrangler deploy`, which never conflicts with this resource. A custom domain binds the
# hostname to the Worker and creates the proxied DNS record for it.
resource "cloudflare_worker" "this" {
  account_id = var.account_id
  name       = var.name
  subdomain = {
    enabled          = var.workers_dev_subdomain
    previews_enabled = var.workers_dev_subdomain
  }
  observability = {
    enabled            = true
    head_sampling_rate = var.observability_sampling_rate
    logs = {
      enabled            = true
      head_sampling_rate = var.observability_sampling_rate
      invocation_logs    = true
    }
  }
  logpush = var.logpush
  tags    = var.tags
}

resource "cloudflare_workers_custom_domain" "this" {
  account_id = var.account_id
  zone_id    = var.zone_id
  hostname   = var.hostname
  service    = cloudflare_worker.this.name

  depends_on = [cloudflare_worker.this]
}
