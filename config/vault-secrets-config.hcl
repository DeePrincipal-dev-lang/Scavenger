# Vault Configuration for Secrets Management

# Database credentials path
path "secret/data/database/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

# API keys path
path "secret/data/api-keys/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

# Third-party service credentials
path "secret/data/services/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

# Encryption keys
path "secret/data/encryption/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

# SSH keys for deployments
path "secret/data/ssh/*" {
  capabilities = ["read", "list"]
}

# Automatic rotation policies
path "secret/config/rotate/*" {
  capabilities = ["read", "update"]
}

# Secret metadata
path "secret/metadata/*" {
  capabilities = ["read", "list"]
}

# Audit logs
path "sys/audit" {
  capabilities = ["read", "list"]
}

path "auth/token/lookup-self" {
  capabilities = ["read"]
}

path "auth/token/renew-self" {
  capabilities = ["update"]
}
