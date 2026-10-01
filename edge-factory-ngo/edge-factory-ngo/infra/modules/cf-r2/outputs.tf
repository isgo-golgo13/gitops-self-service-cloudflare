output "bucket" {
  value = { name = cloudflare_r2_bucket.this.name, location = cloudflare_r2_bucket.this.location }
}
