variable "account_id" { type = string }
variable "zone_id" { type = string }
variable "name" { type = string }
variable "hostname" { type = string }
variable "origins" {
  description = "Origin hostnames or IPs (no scheme)."
  type        = list(string)
}
variable "health_path" {
  type    = string
  default = "/healthz"
}
