# Azure Provider Configuration
terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
  }
}

provider "azurerm" {
  features {}

  subscription_id = var.azure_subscription_id
}

# Resource Group
resource "azurerm_resource_group" "randomvalidator" {
  name     = "randomvalidator-rg"
  location = var.azure_location
}

# PostgreSQL Flexible Server (Free Tier: B1ms with 32GB storage)
resource "azurerm_postgresql_flexible_server" "randomvalidator" {
  name                   = "randomvalidator-db-${random_string.db_suffix.result}"
  resource_group_name    = azurerm_resource_group.randomvalidator.name
  location              = azurerm_resource_group.randomvalidator.location
  version               = "16"
  administrator_login    = var.db_admin_username
  administrator_password = var.db_admin_password

  # Free tier: B1ms burstable (750 hours/month free)
  sku_name   = "B_Standard_B1ms"
  storage_mb = 32768  # 32GB - included in free tier

  backup_retention_days = 7

  # Allow Azure services and public internet access (we'll restrict via firewall rules)
  public_network_access_enabled = true
}

# Random suffix for unique database name
resource "random_string" "db_suffix" {
  length  = 6
  special = false
  upper   = false
}

# Database
resource "azurerm_postgresql_flexible_server_database" "randomvalidator" {
  name      = "randomvalidator"
  server_id = azurerm_postgresql_flexible_server.randomvalidator.id
  collation = "en_US.utf8"
  charset   = "utf8"
}

# Firewall rule to allow GCP instance
resource "azurerm_postgresql_flexible_server_firewall_rule" "gcp_instance" {
  name             = "allow-gcp-instance"
  server_id        = azurerm_postgresql_flexible_server.randomvalidator.id
  start_ip_address = google_compute_address.static_ip.address
  end_ip_address   = google_compute_address.static_ip.address
}

# Firewall rule to allow your local machine for management (optional - remove in production)
resource "azurerm_postgresql_flexible_server_firewall_rule" "allow_local" {
  name             = "allow-local-dev"
  server_id        = azurerm_postgresql_flexible_server.randomvalidator.id
  start_ip_address = "0.0.0.0"
  end_ip_address   = "255.255.255.255"

  # Comment: Remove this rule in production, only for initial setup/debugging
}

# Outputs
output "database_host" {
  value       = azurerm_postgresql_flexible_server.randomvalidator.fqdn
  description = "PostgreSQL server hostname"
  sensitive   = false
}

output "database_name" {
  value       = azurerm_postgresql_flexible_server_database.randomvalidator.name
  description = "PostgreSQL database name"
}

output "database_connection_string" {
  value       = "postgresql://${var.db_admin_username}:${var.db_admin_password}@${azurerm_postgresql_flexible_server.randomvalidator.fqdn}:5432/${azurerm_postgresql_flexible_server_database.randomvalidator.name}?sslmode=require"
  description = "PostgreSQL connection string"
  sensitive   = true
}
