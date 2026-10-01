output "rulesets" {
  value = { waf = cloudflare_ruleset.waf_managed.id, rate_limit = cloudflare_ruleset.rate_limit.id }
}
