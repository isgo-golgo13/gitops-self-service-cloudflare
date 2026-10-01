output "hyperdrive" {
  value = { id = cloudflare_hyperdrive_config.this.id, name = cloudflare_hyperdrive_config.this.name }
}
