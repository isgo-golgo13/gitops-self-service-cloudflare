output "namespace" {
  value = kubernetes_namespace_v1.flux.metadata[0].name
}

output "source" {
  description = "Name of the OCIRepository Flux reconciles from."
  value       = "vm-factory"
}
