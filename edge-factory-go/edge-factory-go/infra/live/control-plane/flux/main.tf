# Day 0: Flux + the bootstrap source on the control-plane cluster. Run once.
module "flux" {
  source     = "../../../modules/flux-tofu-controller"
  registry   = var.registry
  artifact   = var.artifact
  flux_chart = var.flux_chart
}
