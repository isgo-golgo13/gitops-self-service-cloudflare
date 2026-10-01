terraform {
  required_version = ">= 1.10.0, < 2.0.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = ">= 5.26.0, < 6.0.0"
    }
  }
}

# Only for exposure = hybrid: a Cloudflare load balancer in front of EXTERNAL origins
# (a Tunnel to on-prem, another cloud). Workers and Containers never need this.
resource "cloudflare_load_balancer_monitor" "this" {
  account_id     = var.account_id
  description    = "${var.name} health"
  type           = "https"
  method         = "GET"
  path           = var.health_path
  expected_codes = "2xx"
  interval       = 60
  timeout        = 5
  retries        = 2
}

resource "cloudflare_load_balancer_pool" "this" {
  account_id      = var.account_id
  name            = var.name
  monitor         = cloudflare_load_balancer_monitor.this.id
  minimum_origins = 1
  origins = [for i, o in var.origins : {
    name    = "origin-${i}"
    address = o
    enabled = true
  }]
}

resource "cloudflare_load_balancer" "this" {
  zone_id          = var.zone_id
  name             = var.hostname
  default_pools    = [cloudflare_load_balancer_pool.this.id]
  fallback_pool    = cloudflare_load_balancer_pool.this.id
  proxied          = true
  steering_policy  = "dynamic_latency"
  session_affinity = "none"
}
