variable "project_id" {
  description = "GCP Project ID"
  type        = string
}

variable "region" {
  description = "GCP region for resources (must be us-west1, us-central1, or us-east1 for free tier)"
  type        = string
  default     = "us-central1"

  validation {
    condition     = contains(["us-west1", "us-central1", "us-east1"], var.region)
    error_message = "Region must be us-west1, us-central1, or us-east1 for Always Free tier."
  }
}

variable "zone" {
  description = "GCP zone for compute instance"
  type        = string
  default     = "us-central1-a"
}

variable "repository_url" {
  description = "GitHub repository URL for the application"
  type        = string
}

# Azure Variables
variable "azure_subscription_id" {
  description = "Azure Subscription ID"
  type        = string
}

variable "azure_location" {
  description = "Azure region for resources (should be close to GCP region)"
  type        = string
  default     = "centralus"  # Close to GCP us-central1
}

variable "db_admin_username" {
  description = "PostgreSQL administrator username"
  type        = string
  default     = "dbadmin"
}

variable "db_admin_password" {
  description = "PostgreSQL administrator password (min 8 chars)"
  type        = string
  sensitive   = true
}
