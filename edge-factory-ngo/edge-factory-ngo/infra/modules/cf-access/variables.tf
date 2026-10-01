variable "account_id" { type = string }
variable "name" { type = string }
variable "hostname" { type = string }
variable "allowed_email_domains" { type = list(string) }
variable "session_duration" {
  type    = string
  default = "24h"
}
