terraform {
  required_version = ">= 1.10.0, < 2.0.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = ">= 5.26.0, < 6.0.0"
    }
  }
}

# =============================================================================
# app-stack -- the PROFILE LAYER (OpenTofu's patch-and-transform).
#
# Input: the friendly spec an app team writes (name, hostname, tier, exposure,
# database, objects, container). Nothing provider-shaped.
# Output: the provider arguments every child module needs, plus the platform's
# hard opinions. Every "patch" below is a map lookup, so the opinions are data
# you can read, not code you must trace. Identical semantics to the Crossplane
# Composition in the XP variant: same inputs, same outputs, same defaults.
# =============================================================================

locals {
  s = var.spec

  # ---- tier -> platform opinions (the transforms) ------------------------------
  tier = {
    small = {
      observability_sampling = 0.1
      logpush                = false
      d1_read_replication    = false
      r2_storage_class       = "Standard"
      r2_infrequent_after    = null
      rate_limit_rpm         = 300
      access_session         = "24h"
      hyperdrive_caching     = true
    }
    medium = {
      observability_sampling = 0.5
      logpush                = false
      d1_read_replication    = true
      r2_storage_class       = "Standard"
      r2_infrequent_after    = 90
      rate_limit_rpm         = 1200
      access_session         = "12h"
      hyperdrive_caching     = true
    }
    large = {
      observability_sampling = 1
      logpush                = true
      d1_read_replication    = true
      r2_storage_class       = "Standard"
      r2_infrequent_after    = 180
      rate_limit_rpm         = 6000
      access_session         = "8h"
      hyperdrive_caching     = true
    }
  }[local.s.tier]

  # ---- exposure -> which guards exist --------------------------------------------
  exposure = {
    public  = { access = false, load_balancer = false }
    private = { access = true, load_balancer = false }
    hybrid  = { access = false, load_balancer = true }
  }[local.s.exposure]

  hostname = "${local.s.hostname}.${local.s.zone}"

  # Names: deterministic, flat, Cloudflare-safe. Teams never choose them.
  names = {
    worker     = local.s.name
    r2         = "${local.s.name}-objects"
    d1         = "${local.s.name}-db"
    hyperdrive = "${local.s.name}-pg"
    access     = local.s.name
  }
}

module "worker" {
  source = "../cf-worker"

  account_id                  = var.account_id
  zone_id                     = var.zone_id
  name                        = local.names.worker
  hostname                    = local.hostname
  observability_sampling_rate = local.tier.observability_sampling
  logpush                     = local.tier.logpush
  tags                        = compact(["tier:${local.s.tier}", "exposure:${local.s.exposure}", local.s.container ? "runtime:container" : "runtime:worker"])
}

module "objects" {
  source = "../cf-r2"
  count  = local.s.objects ? 1 : 0

  account_id                   = var.account_id
  name                         = local.names.r2
  location                     = var.region
  storage_class                = local.tier.r2_storage_class
  infrequent_access_after_days = local.tier.r2_infrequent_after
}

module "d1" {
  source = "../cf-d1"
  count  = local.s.database == "d1" ? 1 : 0

  account_id       = var.account_id
  name             = local.names.d1
  location         = var.region
  read_replication = local.tier.d1_read_replication
}

module "hyperdrive" {
  source = "../cf-hyperdrive"
  count  = local.s.database == "postgres" ? 1 : 0

  account_id = var.account_id
  name       = local.names.hyperdrive
  origin     = var.postgres_origin
  caching    = local.tier.hyperdrive_caching
}

module "access" {
  source = "../cf-access"
  count  = local.exposure.access ? 1 : 0

  account_id            = var.account_id
  name                  = local.names.access
  hostname              = local.hostname
  allowed_email_domains = var.allowed_email_domains
  session_duration      = local.tier.access_session
}

module "load_balancer" {
  source = "../cf-load-balancer"
  count  = local.exposure.load_balancer ? 1 : 0

  account_id = var.account_id
  zone_id    = var.zone_id
  name       = local.s.name
  hostname   = local.hostname
  origins    = local.s.origins
}
