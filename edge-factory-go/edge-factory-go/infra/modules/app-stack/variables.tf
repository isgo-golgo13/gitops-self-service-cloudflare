# The friendly spec. This is the whole user-facing API; the XRD in the XP variant
# declares exactly these fields. Validation here mirrors the XRD's CEL rules.
variable "spec" {
  type = object({
    name      = string
    zone      = string
    hostname  = string
    tier      = string
    exposure  = string
    database  = optional(string, "none")
    objects   = optional(bool, false)
    container = optional(bool, false)
    origins   = optional(list(string), [])
  })

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{1,40}$", var.spec.name)) && can(regex("^[a-z][a-z0-9-]{0,40}$", var.spec.hostname))
    error_message = "name and hostname: lowercase letters, digits, hyphens."
  }
  validation {
    condition     = contains(["small", "medium", "large"], var.spec.tier)
    error_message = "tier must be small, medium or large."
  }
  validation {
    condition     = contains(["public", "private", "hybrid"], var.spec.exposure)
    error_message = "exposure must be public, private or hybrid."
  }
  validation {
    condition     = contains(["none", "d1", "postgres"], var.spec.database)
    error_message = "database must be none, d1 or postgres."
  }
  validation {
    condition     = var.spec.exposure != "hybrid" || length(var.spec.origins) > 0
    error_message = "exposure = hybrid needs at least one external origin."
  }
}

# ---- platform-supplied (never in the friendly spec) ----------------------------
variable "account_id" { type = string }
variable "zone_id" { type = string }
variable "region" {
  description = "Location hint for D1 and R2: apac | eeur | enam | weur | wnam | oc."
  type        = string
}
variable "allowed_email_domains" {
  description = "Identities allowed through Access for private apps."
  type        = list(string)
  default     = []
}
variable "postgres_origin" {
  description = "Only when database = postgres: the external Postgres Hyperdrive fronts."
  type = object({
    host     = string
    port     = optional(number, 5432)
    database = string
    user     = string
    password = string
  })
  default   = null
  sensitive = true
}
