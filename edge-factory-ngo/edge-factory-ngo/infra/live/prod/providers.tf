# CLOUDFLARE_API_TOKEN comes from the environment (OpenBao -> job env / runner pod).
# The token is scoped: Workers, R2, D1, Hyperdrive, Access, Zone WAF, LB, DNS for the zones listed.
provider "cloudflare" {}
