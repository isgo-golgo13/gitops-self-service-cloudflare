# Installs Flux on a VKS cluster and points it at this repository's OCI artifact.
# Everything after this module (tofu-controller, External Secrets, the image factory,
# the Terraform CRs) is reconciled by Flux from gitops/ - never by this module.
terraform {
  required_version = ">= 1.10.0, < 2.0.0"

  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = ">= 3.2.0, < 4.0.0"
    }
    helm = {
      source  = "hashicorp/helm"
      version = ">= 3.3.0, < 4.0.0"
    }
  }
}

resource "kubernetes_namespace_v1" "flux" {
  metadata {
    name = "flux-system"
    labels = {
      "pod-security.kubernetes.io/enforce" = "privileged"
    }
  }
}

# Pull credentials for the OCI artifact and the Helm charts in Harbor.
resource "kubernetes_secret_v1" "registry" {
  metadata {
    name      = "registry-credentials"
    namespace = kubernetes_namespace_v1.flux.metadata[0].name
  }
  type = "kubernetes.io/dockerconfigjson"
  data = {
    ".dockerconfigjson" = jsonencode({
      auths = {
        (var.registry.host) = {
          username = var.registry.username
          password = var.registry.password
          auth     = base64encode("${var.registry.username}:${var.registry.password}")
        }
      }
    })
  }
}

# Flux controllers (source, kustomize, helm, notification) from the community chart,
# mirrored into Harbor for the enclave.
resource "helm_release" "flux" {
  name       = "flux"
  namespace  = kubernetes_namespace_v1.flux.metadata[0].name
  repository = var.flux_chart.repository
  chart      = "flux2"
  version    = var.flux_chart.version
  wait       = true
  timeout    = 600

  values = [yamlencode({
    imagePullSecrets          = [{ name = kubernetes_secret_v1.registry.metadata[0].name }]
    installCRDs               = true
    imageAutomationController = { create = false }
    imageReflectionController = { create = false }
    sourceController          = { create = true }
    kustomizeController       = { create = true }
    helmController            = { create = true }
    notificationController    = { create = true }
  })]
}

# The bootstrap source and sync: an OCIRepository for this repository's artifact and a
# Kustomization that applies gitops/flux-system. Rendered by Helm at apply time so the
# Flux CRDs do not have to exist at plan time.
resource "helm_release" "sync" {
  name      = "vm-factory-sync"
  namespace = kubernetes_namespace_v1.flux.metadata[0].name
  chart     = "${path.module}/chart"
  wait      = true

  values = [yamlencode({
    artifact = {
      url       = var.artifact.url
      tag       = var.artifact.tag
      interval  = var.artifact.interval
      secretRef = kubernetes_secret_v1.registry.metadata[0].name
    }
    sync = {
      path     = "./gitops/flux-system"
      interval = "10m"
    }
  })]

  depends_on = [helm_release.flux]
}
