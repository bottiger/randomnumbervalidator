terraform {
  required_version = ">= 1.0"
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
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
  startup_script = <<-EOF
    #!/bin/bash
    set -e

    # Update system
    apt-get update
    apt-get install -y build-essential curl git postgresql postgresql-contrib gnupg software-properties-common

    # Install Terraform
    wget -O- https://apt.releases.hashicorp.com/gpg | gpg --dearmor -o /usr/share/keyrings/hashicorp-archive-keyring.gpg
    echo "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com $(lsb_release -cs) main" | tee /etc/apt/sources.list.d/hashicorp.list
    apt-get update
    apt-get install -y terraform

    # Setup PostgreSQL
    systemctl start postgresql
    systemctl enable postgresql

    # Create database and user
    sudo -u postgres psql -c "CREATE DATABASE randomvalidator;" || true
    sudo -u postgres psql -c "CREATE USER randomvalidator WITH PASSWORD 'randomvalidator';" || true
    sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE randomvalidator TO randomvalidator;" || true
    sudo -u postgres psql -d randomvalidator -c "GRANT ALL ON SCHEMA public TO randomvalidator;" || true

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
    After=network.target postgresql.service

    [Service]
    Type=simple
    User=root
    WorkingDirectory=/opt/randomvalidator
    Environment="RUST_LOG=info"
    Environment="DATABASE_URL=postgresql://randomvalidator:randomvalidator@localhost:5432/randomvalidator"
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
    google_compute_firewall.allow_ssh
  ]
}

# Outputs
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
