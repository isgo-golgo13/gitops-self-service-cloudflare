terraform {
  required_version = ">= 1.10.0, < 2.0.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = ">= 5.26.0, < 6.0.0"
    }
  }
}

# D1 is Cloudflare's managed, replicated SQLite. There are no "instances" to run;
# read replication is a switch. Postgres is a different thing (see cf-hyperdrive).
resource "cloudflare_d1_database" "this" {
  account_id            = var.account_id
  name                  = var.name
  primary_location_hint = var.location
  read_replication = {
    mode = var.read_replication ? "auto" : "disabled"
  }
}
