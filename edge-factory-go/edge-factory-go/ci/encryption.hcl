# OpenTofu client-side encryption for state and saved plans. Injected as TF_ENCRYPTION;
# __ENCRYPTION_KEY__ is replaced with the secret from OpenBao (64 hex chars, used as the
# pbkdf2 passphrase). "static" is NOT a shipped key provider - do not use it.
key_provider "pbkdf2" "primary" {
  passphrase = "__ENCRYPTION_KEY__"
}
method "aes_gcm" "primary" {
  keys = key_provider.pbkdf2.primary
}
state {
  method   = method.aes_gcm.primary
  enforced = true
}
plan {
  method   = method.aes_gcm.primary
  enforced = true
}
