terraform {
  required_version = ">= 1.10.0, < 2.0.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = ">= 5.26.0, < 6.0.0"
    }
  }
}

resource "cloudflare_r2_bucket" "this" {
  account_id    = var.account_id
  name          = var.name
  location      = var.location
  storage_class = var.storage_class
  jurisdiction  = var.jurisdiction
}

# Expire multipart leftovers and optionally transition cold objects.
resource "cloudflare_r2_bucket_lifecycle" "this" {
  account_id   = var.account_id
  bucket_name  = cloudflare_r2_bucket.this.name
  jurisdiction = var.jurisdiction
  rules = concat([
    {
      id         = "abort-incomplete-multipart"
      enabled    = true
      conditions = { prefix = "" }
      abort_multipart_uploads_transition = {
        condition = { type = "Age", max_age = 7 * 24 * 3600 }
      }
    }
    ], var.infrequent_access_after_days == null ? [] : [{
      id         = "to-infrequent-access"
      enabled    = true
      conditions = { prefix = "" }
      storage_class_transitions = [{
        storage_class = "InfrequentAccess"
        condition     = { type = "Age", max_age = var.infrequent_access_after_days * 24 * 3600 }
      }]
  }])
}
