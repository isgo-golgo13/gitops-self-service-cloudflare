output "load_balancer" {
  value = { id = cloudflare_load_balancer.this.id, pool = cloudflare_load_balancer_pool.this.id }
}
