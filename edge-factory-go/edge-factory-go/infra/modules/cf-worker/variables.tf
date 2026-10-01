variable "account_id" { type = string }
variable "zone_id" { type = string }
variable "name" {
  description = "Worker name; app teams deploy code to it with wrangler."
  type        = string
}
variable "hostname" {
  description = "FQDN bound to the Worker (creates the proxied DNS record)."
  type        = string
}
variable "workers_dev_subdomain" {
  type    = bool
  default = false
}
variable "observability_sampling_rate" {
  type    = number
  default = 1
}
variable "logpush" {
  type    = bool
  default = false
}
variable "tags" {
  type    = list(string)
  default = []
}
