output "namespace" {
  value = kubernetes_namespace_v1.crossplane.metadata[0].name
}
