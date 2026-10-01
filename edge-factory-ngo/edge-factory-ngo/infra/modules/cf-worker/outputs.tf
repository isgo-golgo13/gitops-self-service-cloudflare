output "worker" {
  value = {
    id       = cloudflare_worker.this.id
    name     = cloudflare_worker.this.name
    hostname = cloudflare_workers_custom_domain.this.hostname
  }
}
