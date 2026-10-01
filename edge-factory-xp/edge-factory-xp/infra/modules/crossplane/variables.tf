variable "registry" {
  description = "Harbor holding the mirrored Crossplane image, providers, functions and the platform packages."
  type = object({
    host     = string
    username = string
    password = string
  })
  sensitive = true
}

variable "cosign_public_key" {
  description = "PEM public key matching the OpenBao Transit signing key used by `make publish` and the IDP."
  type        = string
}

variable "chart" {
  description = "Crossplane Helm chart location (mirrored into Harbor) and version."
  type = object({
    repository = string
    version    = string
  })
  default = {
    repository = "https://charts.crossplane.io/stable"
    version    = "2.4.2"
  }
}
