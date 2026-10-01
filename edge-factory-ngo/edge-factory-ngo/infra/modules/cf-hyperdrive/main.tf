terraform {
  required_version = ">= 1.10.0, < 2.0.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = ">= 5.26.0, < 6.0.0"
    }
  }
}

# Hyperdrive: connection pooling + caching in front of an EXTERNAL Postgres (Neon,
# Supabase, your own). The database itself is not a Cloudflare resource.
resource "cloudflare_hyperdrive_config" "this" {
  account_id = var.account_id
  name       = var.name
  origin = {
    scheme   = "postgres"
    host     = var.origin.host
    port     = var.origin.port
    database = var.origin.database
    user     = var.origin.user
    password = var.origin.password
  }
  caching = {
    disabled = !var.caching
  }
}
