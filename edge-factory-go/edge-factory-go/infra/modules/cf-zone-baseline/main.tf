terraform {
  required_version = ">= 1.10.0, < 2.0.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = ">= 5.26.0, < 6.0.0"
    }
  }
}

# Rulesets are one-per-phase-per-zone. The root aggregates every app's needs into these
# two objects; apps never own a ruleset.
resource "cloudflare_ruleset" "waf_managed" {
  zone_id     = var.zone_id
  name        = "managed-waf"
  description = "Cloudflare Managed + OWASP rulesets for every hostname in the zone"
  kind        = "zone"
  phase       = "http_request_firewall_managed"
  rules = [
    {
      description       = "Cloudflare Managed Ruleset"
      expression        = "true"
      action            = "execute"
      action_parameters = { id = "efb7b8c949ac4650a09736fc376e9aee" }
      enabled           = true
    },
    {
      description       = "OWASP Core Ruleset"
      expression        = "true"
      action            = "execute"
      action_parameters = { id = "4814384a9e5d4991b9815dcfc25d2f1f" }
      enabled           = true
    }
  ]
}

resource "cloudflare_ruleset" "rate_limit" {
  zone_id     = var.zone_id
  name        = "rate-limits"
  description = "Per-hostname rate limits, one rule per app from its tier"
  kind        = "zone"
  phase       = "http_ratelimit"
  rules = [for h, rpm in var.rate_limits : {
    description = "rate limit ${h}"
    expression  = "(http.host eq \"${h}\")"
    action      = "block"
    ratelimit = {
      characteristics     = ["ip.src", "cf.colo.id"]
      period              = 60
      requests_per_period = rpm
      mitigation_timeout  = 60
    }
    enabled = true
  }]
}

# Zone-wide security floor.
resource "cloudflare_zone_setting" "tls" {
  zone_id    = var.zone_id
  setting_id = "min_tls_version"
  value      = "1.2"
}

resource "cloudflare_zone_setting" "https" {
  zone_id    = var.zone_id
  setting_id = "always_use_https"
  value      = "on"
}
