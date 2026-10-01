variable "account_id" { type = string }
variable "name" { type = string }
variable "origin" {
  type = object({
    host     = string
    port     = optional(number, 5432)
    database = string
    user     = string
    password = string
  })
  sensitive = true
}
variable "caching" {
  type    = bool
  default = true
}
