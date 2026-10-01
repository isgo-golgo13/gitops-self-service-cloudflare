cluster    = { kubeconfig = "~/.kube/config", context = "k3s-edge" }
artifact   = { url = "oci://ghcr.io/isgo-golgo13/edge-factory-go/tree", tag = "main", interval = "1m" }
flux_chart = { repository = "oci://ghcr.io/fluxcd-community/charts", version = "2.19.1" }
# registry = { host = "ghcr.io", username = "...", password = "..." } -> TF_VAR_registry from OpenBao
