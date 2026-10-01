terraform {
  required_version = ">= 1.10.0, < 2.0.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = ">= 5.26.0, < 6.0.0"
    }
  }
}

# Zero Trust Access in front of a private hostname: identity before the Worker ever runs.
resource "cloudflare_zero_trust_access_application" "this" {
  account_id       = var.account_id
  name             = var.name
  type             = "self_hosted"
  domain           = var.hostname
  session_duration = var.session_duration
  policies = [{
    id         = cloudflare_zero_trust_access_policy.allow.id
    precedence = 1
  }]
}

resource "cloudflare_zero_trust_access_policy" "allow" {
  account_id       = var.account_id
  name             = "${var.name}-allow"
  decision         = "allow"
  session_duration = var.session_duration
  include          = [for d in var.allowed_email_domains : { email_domain = { domain = d } }]
}
