output "database" {
  value = { id = cloudflare_d1_database.this.id, name = cloudflare_d1_database.this.name }
}
