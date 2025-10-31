terraform {
  required_version = ">= 1.0"
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.0"
    }
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
}

# Enable required APIs
resource "google_project_service" "compute" {
  service            = "compute.googleapis.com"
  disable_on_destroy = false
}

resource "google_project_service" "cloudresourcemanager" {
  service            = "cloudresourcemanager.googleapis.com"
  disable_on_destroy = false
}

# Reserve a static IP address
resource "google_compute_address" "static_ip" {
  name   = "randomvalidator-ip"
  region = var.region

  depends_on = [google_project_service.compute]
}

# Firewall rule to allow HTTP traffic
resource "google_compute_firewall" "allow_http" {
  name    = "allow-http-randomvalidator"
  network = "default"

  allow {
    protocol = "tcp"
    ports    = ["80", "3000"]
  }

  source_ranges = ["0.0.0.0/0"]
  target_tags   = ["randomvalidator"]

  depends_on = [google_project_service.compute]
}

# Firewall rule to allow SSH
resource "google_compute_firewall" "allow_ssh" {
  name    = "allow-ssh-randomvalidator"
  network = "default"

  allow {
    protocol = "tcp"
    ports    = ["22"]
  }

  source_ranges = ["0.0.0.0/0"]
  target_tags   = ["randomvalidator"]

  depends_on = [google_project_service.compute]
}

# Create startup script
locals {
  # Database connection string (will be available after Azure DB is created)
  database_url = "postgresql://${var.db_admin_username}:${var.db_admin_password}@${azurerm_postgresql_flexible_server.randomvalidator.fqdn}:5432/${azurerm_postgresql_flexible_server_database.randomvalidator.name}?sslmode=require"

  startup_script = <<-EOF
    #!/bin/bash
    set -e

    # Update system
    apt-get update
    apt-get install -y build-essential curl git postgresql-client

    # Install Rust
    if ! command -v rustc &> /dev/null; then
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
      source $HOME/.cargo/env
    fi

    # Create application directory
    mkdir -p /opt/randomvalidator
    cd /opt/randomvalidator

    # Clone or update repository
    if [ -d ".git" ]; then
      git pull
    else
      git clone ${var.repository_url} .
    fi

    # Build NIST test suite
    cd nist/sts-2.1.2/sts-2.1.2
    make clean || true
    make
    cd /opt/randomvalidator

    # Build Rust application
    source $HOME/.cargo/env
    cargo build --release --bin server

    # Create systemd service
    cat > /etc/systemd/system/randomvalidator.service <<-SERVICE
    [Unit]
    Description=Random Number Validator
    After=network.target

    [Service]
    Type=simple
    User=root
    WorkingDirectory=/opt/randomvalidator
    Environment="RUST_LOG=info"
    Environment="DATABASE_URL=${local.database_url}"
    ExecStart=/opt/randomvalidator/target/release/server
    Restart=always
    RestartSec=10

    [Install]
    WantedBy=multi-user.target
    SERVICE

    # Enable and start service
    systemctl daemon-reload
    systemctl enable randomvalidator
    systemctl restart randomvalidator

    echo "Deployment complete!"
  EOF
}

# Compute Engine instance (e2-micro - free tier)
resource "google_compute_instance" "randomvalidator" {
  name         = "randomvalidator-instance"
  machine_type = "e2-micro"
  zone         = var.zone

  tags = ["randomvalidator"]

  boot_disk {
    initialize_params {
      image = "ubuntu-os-cloud/ubuntu-2204-lts"
      size  = 30 # GB
      type  = "pd-standard"
    }
  }

  network_interface {
    network = "default"

    access_config {
      nat_ip = google_compute_address.static_ip.address
    }
  }

  metadata_startup_script = local.startup_script

  # Allow the instance to be stopped for maintenance
  allow_stopping_for_update = true

  depends_on = [
    google_project_service.compute,
    google_compute_firewall.allow_http,
    google_compute_firewall.allow_ssh,
    azurerm_postgresql_flexible_server.randomvalidator,
    azurerm_postgresql_flexible_server_database.randomvalidator
  ]
}

# Output the instance IP
output "instance_ip" {
  value       = google_compute_address.static_ip.address
  description = "Public IP address of the instance"
}

output "instance_name" {
  value       = google_compute_instance.randomvalidator.name
  description = "Name of the compute instance"
}

output "application_url" {
  value       = "http://${google_compute_address.static_ip.address}:3000"
  description = "URL to access the application"
}
