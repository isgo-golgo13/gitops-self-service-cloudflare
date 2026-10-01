# Installs Crossplane on the VKS control-plane cluster and registers the two bootstrap objects
# that let Harbor take over: the registry pull secret and the ImageConfig. Everything else
# (packages, XRD, Composition, environment) is `make register` from platform/.
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

resource "kubernetes_namespace_v1" "crossplane" {
  metadata {
    name = "crossplane-system"
  }
}

resource "kubernetes_secret_v1" "registry" {
  metadata {
    name      = "registry-credentials"
    namespace = kubernetes_namespace_v1.crossplane.metadata[0].name
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

# Public half of the platform signing key: what ImageConfig verifies packages against.
resource "kubernetes_secret_v1" "cosign_public_key" {
  metadata {
    name      = "cosign-public-key"
    namespace = kubernetes_namespace_v1.crossplane.metadata[0].name
  }
  data = {
    "cosign.pub" = var.cosign_public_key
  }
}

resource "helm_release" "crossplane" {
  name       = "crossplane"
  namespace  = kubernetes_namespace_v1.crossplane.metadata[0].name
  repository = var.chart.repository
  chart      = "crossplane"
  version    = var.chart.version
  wait       = true
  timeout    = 600

  values = [yamlencode({
    imagePullSecrets = [kubernetes_secret_v1.registry.metadata[0].name]
    # Signature verification of packages is alpha in Crossplane 2.x; enabled deliberately.
    args = ["--enable-signature-verification"]
    resourcesCrossplane = {
      limits   = { cpu = "1", memory = "1Gi" }
      requests = { cpu = "200m", memory = "256Mi" }
    }
  })]
}
