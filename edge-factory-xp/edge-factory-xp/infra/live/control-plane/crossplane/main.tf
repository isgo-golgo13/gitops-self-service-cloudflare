# Day 0: Crossplane on your k3s. Run once.
module "crossplane" {
  source            = "../../../modules/crossplane"
  registry          = var.registry
  cosign_public_key = var.cosign_public_key
  chart             = var.chart
}
