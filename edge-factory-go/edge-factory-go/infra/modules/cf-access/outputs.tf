output "access" {
  value = { application_id = cloudflare_zero_trust_access_application.this.id, policy_id = cloudflare_zero_trust_access_policy.allow.id }
}
