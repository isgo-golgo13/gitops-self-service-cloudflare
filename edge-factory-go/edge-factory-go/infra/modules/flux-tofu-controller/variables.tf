variable "registry" {
  description = "Harbor (or any OCI registry) holding the repository artifact and mirrored charts."
  type = object({
    host     = string
    username = string
    password = string
  })
  sensitive = true
}

variable "artifact" {
  description = "OCI artifact of this repository pushed with `flux push artifact` (make push)."
  type = object({
    url      = string # oci://ghcr.io/isgo-golgo13/edge-factory-go/tree
    tag      = string # main
    interval = string # 1m
  })
}

variable "flux_chart" {
  description = "flux2 community chart location and version (mirror it into Harbor for the enclave)."
  type = object({
    repository = string
    version    = string
  })
  default = {
    repository = "oci://ghcr.io/fluxcd-community/charts"
    version    = "2.19.1"
  }
}
