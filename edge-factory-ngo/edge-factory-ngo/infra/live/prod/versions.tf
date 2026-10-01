terraform {
  required_version = ">= 1.10.0, < 2.0.0"

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "5.26.0"
    }
  }

  # State lives in a DEDICATED R2 bucket (never the application buckets), through the S3
  # backend; settings in backend.tfbackend, keys from OpenBao as AWS_ACCESS_KEY_ID/SECRET.
  backend "s3" {}

  # State and plan encryption is injected as TF_ENCRYPTION (pbkdf2 passphrase from OpenBao)
  # by ci/tofu.mjs, the runner pod, or `make bootstrap`. enforced = true lives in that block.
}
