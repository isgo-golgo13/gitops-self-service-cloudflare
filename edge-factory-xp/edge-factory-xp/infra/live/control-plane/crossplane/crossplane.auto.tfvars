cluster = { kubeconfig = "~/.kube/config", context = "k3s-edge" }
chart   = { repository = "https://charts.crossplane.io/stable", version = "2.4.2" }
# cosign_public_key, registry -> TF_VAR_* from OpenBao at bootstrap, never here.
