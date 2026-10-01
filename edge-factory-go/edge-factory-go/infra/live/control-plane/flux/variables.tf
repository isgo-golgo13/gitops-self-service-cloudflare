variable "cluster" {
  type = object({ kubeconfig = string, context = string })
}
variable "registry" {
  description = "GHCR (or any OCI registry) pull credential: TF_VAR_registry from OpenBao."
  type        = object({ host = string, username = string, password = string })
  sensitive   = true
}
variable "artifact" {
  type = object({ url = string, tag = string, interval = string })
}
variable "flux_chart" {
  type = object({ repository = string, version = string })
}
