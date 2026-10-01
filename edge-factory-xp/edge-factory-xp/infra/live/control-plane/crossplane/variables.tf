variable "cluster" {
  type = object({ kubeconfig = string, context = string })
}
variable "registry" {
  description = "GHCR pull credential: TF_VAR_registry from OpenBao."
  type        = object({ host = string, username = string, password = string })
  sensitive   = true
}
variable "cosign_public_key" { type = string }
variable "chart" {
  type = object({ repository = string, version = string })
}
