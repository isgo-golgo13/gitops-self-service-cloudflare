variable "zone_id" { type = string }
variable "rate_limits" {
  description = "hostname -> requests per minute per client IP"
  type        = map(number)
}
