variable "account_id" { type = string }
variable "name" { type = string }
variable "location" {
  type    = string
  default = "enam"
}
variable "read_replication" {
  type    = bool
  default = true
}
