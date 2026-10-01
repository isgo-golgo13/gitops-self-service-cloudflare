variable "account_id" { type = string }
variable "name" { type = string }
variable "location" {
  description = "Location hint: apac | eeur | enam | weur | wnam | oc."
  type        = string
  default     = "enam"
}
variable "storage_class" {
  type    = string
  default = "Standard"
}
variable "jurisdiction" {
  description = "default | eu | fedramp"
  type        = string
  default     = "default"
}
variable "infrequent_access_after_days" {
  type    = number
  default = null
}
