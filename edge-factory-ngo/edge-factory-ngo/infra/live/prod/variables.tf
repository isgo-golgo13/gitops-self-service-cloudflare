variable "account_id" { type = string }

variable "region" {
  description = "Default location hint for data: enam | wnam | weur | eeur | apac | oc."
  type        = string
  default     = "weur"
}

variable "allowed_email_domains" {
  description = "Identity domains admitted to private apps (Zero Trust Access)."
  type        = list(string)
  default     = []
}

# Only for apps with database = postgres: the external Postgres each one fronts.
# Supplied via TF_VAR_postgres_origins from OpenBao, never from a file.
variable "postgres_origins" {
  type = map(object({
    host     = string
    port     = optional(number, 5432)
    database = string
    user     = string
    password = string
  }))
  default   = {}
  sensitive = true
}
