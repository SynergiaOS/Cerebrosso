# 🐺 Projekt Cerberus Phoenix v2.0 - Oracle Cloud Infrastructure
# Terraform configuration for Oracle Cloud Free Tier deployment

terraform {
  required_version = ">= 1.0"
  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 5.0"
    }
  }
}

# 🔐 Provider Configuration
provider "oci" {
  tenancy_ocid     = var.tenancy_ocid
  user_ocid        = var.user_ocid
  fingerprint      = var.fingerprint
  private_key_path = var.private_key_path
  region           = var.region
}

# 📊 Variables
variable "tenancy_ocid" {
  description = "OCID of the tenancy"
  type        = string
}

variable "user_ocid" {
  description = "OCID of the user"
  type        = string
}

variable "fingerprint" {
  description = "Fingerprint of the public key"
  type        = string
}

variable "private_key_path" {
  description = "Path to the private key file"
  type        = string
}

variable "region" {
  description = "Oracle Cloud region"
  type        = string
  default     = "eu-frankfurt-1"
}

variable "compartment_ocid" {
  description = "OCID of the compartment"
  type        = string
}

variable "ssh_public_key" {
  description = "SSH public key for instance access"
  type        = string
}

# 🌐 Data Sources
data "oci_identity_availability_domains" "ads" {
  compartment_id = var.tenancy_ocid
}

data "oci_core_images" "ubuntu_images" {
  compartment_id           = var.compartment_ocid
  operating_system         = "Canonical Ubuntu"
  operating_system_version = "22.04"
  shape                    = "VM.Standard.A1.Flex"
  sort_by                  = "TIMECREATED"
  sort_order               = "DESC"
}

# 🔒 Security Group
resource "oci_core_network_security_group" "cerberus_nsg" {
  compartment_id = var.compartment_ocid
  vcn_id         = oci_core_vcn.cerberus_vcn.id
  display_name   = "cerberus-nsg"

  freeform_tags = {
    "Project" = "Cerberus-Phoenix"
    "Environment" = "production"
  }
}

# 🔒 Security Rules
resource "oci_core_network_security_group_security_rule" "cerberus_nsg_rule_ssh" {
  network_security_group_id = oci_core_network_security_group.cerberus_nsg.id
  direction                 = "INGRESS"
  protocol                  = "6"
  source                    = "0.0.0.0/0"
  source_type               = "CIDR_BLOCK"
  
  tcp_options {
    destination_port_range {
      min = 22
      max = 22
    }
  }
}

resource "oci_core_network_security_group_security_rule" "cerberus_nsg_rule_http" {
  network_security_group_id = oci_core_network_security_group.cerberus_nsg.id
  direction                 = "INGRESS"
  protocol                  = "6"
  source                    = "0.0.0.0/0"
  source_type               = "CIDR_BLOCK"
  
  tcp_options {
    destination_port_range {
      min = 80
      max = 80
    }
  }
}

resource "oci_core_network_security_group_security_rule" "cerberus_nsg_rule_https" {
  network_security_group_id = oci_core_network_security_group.cerberus_nsg.id
  direction                 = "INGRESS"
  protocol                  = "6"
  source                    = "0.0.0.0/0"
  source_type               = "CIDR_BLOCK"
  
  tcp_options {
    destination_port_range {
      min = 443
      max = 443
    }
  }
}

# 🌐 VCN (Virtual Cloud Network)
resource "oci_core_vcn" "cerberus_vcn" {
  compartment_id = var.compartment_ocid
  cidr_blocks    = ["10.0.0.0/16"]
  display_name   = "cerberus-vcn"
  dns_label      = "cerberus"

  freeform_tags = {
    "Project" = "Cerberus-Phoenix"
    "Environment" = "production"
  }
}

# 🌐 Internet Gateway
resource "oci_core_internet_gateway" "cerberus_igw" {
  compartment_id = var.compartment_ocid
  vcn_id         = oci_core_vcn.cerberus_vcn.id
  display_name   = "cerberus-igw"
  enabled        = true

  freeform_tags = {
    "Project" = "Cerberus-Phoenix"
    "Environment" = "production"
  }
}

# 🛣️ Route Table
resource "oci_core_route_table" "cerberus_rt" {
  compartment_id = var.compartment_ocid
  vcn_id         = oci_core_vcn.cerberus_vcn.id
  display_name   = "cerberus-rt"

  route_rules {
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
    network_entity_id = oci_core_internet_gateway.cerberus_igw.id
  }

  freeform_tags = {
    "Project" = "Cerberus-Phoenix"
    "Environment" = "production"
  }
}

# 🌐 Subnet
resource "oci_core_subnet" "cerberus_subnet" {
  compartment_id      = var.compartment_ocid
  vcn_id              = oci_core_vcn.cerberus_vcn.id
  cidr_block          = "10.0.1.0/24"
  display_name        = "cerberus-subnet"
  dns_label           = "cerberus"
  route_table_id      = oci_core_route_table.cerberus_rt.id
  security_list_ids   = [oci_core_vcn.cerberus_vcn.default_security_list_id]
  availability_domain = data.oci_identity_availability_domains.ads.availability_domains[0].name

  freeform_tags = {
    "Project" = "Cerberus-Phoenix"
    "Environment" = "production"
  }
}

# 🖥️ Compute Instance (Oracle Cloud Free Tier - 4 OCPU, 24GB RAM)
resource "oci_core_instance" "cerberus_instance" {
  compartment_id      = var.compartment_ocid
  availability_domain = data.oci_identity_availability_domains.ads.availability_domains[0].name
  display_name        = "cerberus-phoenix-vm"
  shape               = "VM.Standard.A1.Flex"

  shape_config {
    ocpus         = 4
    memory_in_gbs = 24
  }

  create_vnic_details {
    subnet_id                 = oci_core_subnet.cerberus_subnet.id
    display_name              = "cerberus-vnic"
    assign_public_ip          = true
    assign_private_dns_record = true
    hostname_label            = "cerberus"
    nsg_ids                   = [oci_core_network_security_group.cerberus_nsg.id]
  }

  source_details {
    source_type = "image"
    source_id   = data.oci_core_images.ubuntu_images.images[0].id
  }

  metadata = {
    ssh_authorized_keys = var.ssh_public_key
    user_data          = base64encode(file("${path.module}/cloud-init.yml"))
  }

  freeform_tags = {
    "Project" = "Cerberus-Phoenix"
    "Environment" = "production"
    "Component" = "main-vm"
  }
}

# 🔐 Vault Secret for Solana Keypair
resource "oci_vault_secret" "cerberus_solana_keypair" {
  compartment_id = var.compartment_ocid
  secret_name    = "cerberus-solana-keypair"
  vault_id       = oci_kms_vault.cerberus_vault.id
  key_id         = oci_kms_key.cerberus_key.id

  secret_content {
    content_type = "BASE64"
    content      = base64encode(jsonencode({
      private_key = "PLACEHOLDER_PRIVATE_KEY"
      public_key  = "PLACEHOLDER_PUBLIC_KEY"
    }))
  }

  freeform_tags = {
    "Project" = "Cerberus-Phoenix"
    "Environment" = "production"
    "Component" = "solana-keypair"
  }
}

# 🔐 KMS Vault for secret management
resource "oci_kms_vault" "cerberus_vault" {
  compartment_id   = var.compartment_ocid
  display_name     = "cerberus-vault"
  vault_type       = "DEFAULT"

  freeform_tags = {
    "Project" = "Cerberus-Phoenix"
    "Environment" = "production"
  }
}

# 🔑 KMS Key for encryption
resource "oci_kms_key" "cerberus_key" {
  compartment_id      = var.compartment_ocid
  display_name        = "cerberus-key"
  management_endpoint = oci_kms_vault.cerberus_vault.management_endpoint

  key_shape {
    algorithm = "AES"
    length    = 32
  }

  freeform_tags = {
    "Project" = "Cerberus-Phoenix"
    "Environment" = "production"
  }
}

# 📊 Outputs
output "instance_public_ip" {
  description = "Public IP address of the Cerberus instance"
  value       = oci_core_instance.cerberus_instance.public_ip
}

output "instance_private_ip" {
  description = "Private IP address of the Cerberus instance"
  value       = oci_core_instance.cerberus_instance.private_ip
}

output "ssh_connection" {
  description = "SSH connection command"
  value       = "ssh ubuntu@${oci_core_instance.cerberus_instance.public_ip}"
}

output "vault_id" {
  description = "OCI Vault ID for secret management"
  value       = oci_kms_vault.cerberus_vault.id
}

output "vault_management_endpoint" {
  description = "OCI Vault management endpoint"
  value       = oci_kms_vault.cerberus_vault.management_endpoint
}
