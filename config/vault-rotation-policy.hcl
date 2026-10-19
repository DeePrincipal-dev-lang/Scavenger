# Secret rotation policies for Scavenger

# Rotate all API secrets every 30 days
path "secret/data/scavenger/api" {
  capabilities = ["read", "create", "update"]

  min_writable_secret_id_entries = 1

  metadata {
    max_versions = 5
    delete_version_after = "720h"  # 30 days
    custom_metadata = {
      rotation_interval = "720h"
      auto_rotate = "true"
    }
  }
}

# Rotate Stellar signing key every 90 days
path "secret/data/scavenger/stellar" {
  capabilities = ["read", "create", "update"]

  metadata {
    max_versions = 3
    delete_version_after = "2160h"  # 90 days
    custom_metadata = {
      rotation_interval = "2160h"
      auto_rotate = "true"
    }
  }
}

# Database credentials — rotated every 7 days
path "secret/data/scavenger/database" {
  capabilities = ["read", "create", "update"]

  metadata {
    max_versions = 5
    delete_version_after = "168h"  # 7 days
    custom_metadata = {
      rotation_interval = "168h"
      auto_rotate = "true"
    }
  }
}

# Read-only access for applications
path "secret/data/scavenger/*" {
  capabilities = ["read", "list"]
}

path "secret/metadata/scavenger/*" {
  capabilities = ["list", "read"]
}

# Allow token self-renewal
path "auth/token/renew-self" {
  capabilities = ["update"]
}

# Allow checking own token
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
