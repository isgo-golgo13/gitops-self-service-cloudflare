# Day 0 target: a small k3s (or any) cluster you already run. Point at its kubeconfig context.
provider "kubernetes" {
  config_path    = var.cluster.kubeconfig
  config_context = var.cluster.context
}
provider "helm" {
  kubernetes = {
    config_path    = var.cluster.kubeconfig
    config_context = var.cluster.context
  }
}
