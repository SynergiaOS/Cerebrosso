# 🐺 Cerberus Phoenix Evolved - Oracle Cloud Infrastructure
# Production-ready deployment with Vault, monitoring, and security

terraform {
  required_version = ">= 1.0"
  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 5.0"
    }
  }
}

provider "oci" {
  tenancy_ocid     = var.tenancy_ocid
  user_ocid        = var.user_ocid
  fingerprint      = var.fingerprint
  private_key_path = var.private_key_path
  region           = var.region
}

# 🌐 VCN and Networking
resource "oci_core_vcn" "cerberus_vcn" {
  compartment_id = var.compartment_id
  cidr_blocks    = ["10.0.0.0/16"]
  display_name   = "cerberus-vcn"
  dns_label      = "cerberus"

  freeform_tags = {
    Project = "Cerberus Phoenix Evolved"
    Environment = var.environment
  }
}

resource "oci_core_internet_gateway" "cerberus_igw" {
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.cerberus_vcn.id
  display_name   = "cerberus-igw"
  enabled        = true
}

resource "oci_core_route_table" "cerberus_rt" {
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.cerberus_vcn.id
  display_name   = "cerberus-route-table"

  route_rules {
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
    network_entity_id = oci_core_internet_gateway.cerberus_igw.id
  }
}

resource "oci_core_subnet" "cerberus_subnet" {
  compartment_id      = var.compartment_id
  vcn_id              = oci_core_vcn.cerberus_vcn.id
  cidr_block          = "10.0.1.0/24"
  display_name        = "cerberus-subnet"
  dns_label           = "cerberus"
  route_table_id      = oci_core_route_table.cerberus_rt.id
  security_list_ids   = [oci_core_security_list.cerberus_seclist.id]
  prohibit_public_ip_on_vnic = false
}

# 🔐 Security List
resource "oci_core_security_list" "cerberus_seclist" {
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.cerberus_vcn.id
  display_name   = "cerberus-security-list"

  # SSH access
  ingress_security_rules {
    protocol = "6"
    source   = "0.0.0.0/0"
    tcp_options {
      min = 22
      max = 22
    }
  }

  # HTTP/HTTPS
  ingress_security_rules {
    protocol = "6"
    source   = "0.0.0.0/0"
    tcp_options {
      min = 80
      max = 80
    }
  }

  ingress_security_rules {
    protocol = "6"
    source   = "0.0.0.0/0"
    tcp_options {
      min = 443
      max = 443
    }
  }

  # Application ports
  ingress_security_rules {
    protocol = "6"
    source   = "10.0.0.0/16"
    tcp_options {
      min = 3000
      max = 9090
    }
  }

  # All outbound traffic
  egress_security_rules {
    protocol    = "all"
    destination = "0.0.0.0/0"
  }
}

# 🖥️ HFT Ninja Instance
resource "oci_core_instance" "hft_ninja" {
  availability_domain = var.availability_domain
  compartment_id      = var.compartment_id
  display_name        = "cerberus-hft-ninja"
  shape               = "VM.Standard.E4.Flex"

  shape_config {
    ocpus         = 4
    memory_in_gbs = 32
  }

  create_vnic_details {
    subnet_id        = oci_core_subnet.cerberus_subnet.id
    display_name     = "hft-ninja-vnic"
    assign_public_ip = true
    hostname_label   = "hft-ninja"
  }

  source_details {
    source_type = "image"
    source_id   = var.ubuntu_image_id
  }

  metadata = {
    ssh_authorized_keys = file(var.ssh_public_key_path)
    user_data = base64encode(templatefile("${path.module}/cloud-init-hft.yaml", {
      docker_compose_content = base64encode(file("${path.module}/../docker-compose.yml"))
    }))
  }

  freeform_tags = {
    Project = "Cerberus Phoenix Evolved"
    Service = "HFT-Ninja"
    Environment = var.environment
  }
}

# 🔐 Vault Instance
resource "oci_core_instance" "vault" {
  availability_domain = var.availability_domain
  compartment_id      = var.compartment_id
  display_name        = "cerberus-vault"
  shape               = "VM.Standard.E3.Flex"

  shape_config {
    ocpus         = 2
    memory_in_gbs = 16
  }

  create_vnic_details {
    subnet_id        = oci_core_subnet.cerberus_subnet.id
    display_name     = "vault-vnic"
    assign_public_ip = true
    hostname_label   = "vault"
  }

  source_details {
    source_type = "image"
    source_id   = var.ubuntu_image_id
  }

  metadata = {
    ssh_authorized_keys = file(var.ssh_public_key_path)
    user_data = base64encode(file("${path.module}/vault-init.sh"))
  }

  freeform_tags = {
    Project = "Cerberus Phoenix Evolved"
    Service = "Vault"
    Environment = var.environment
  }
}

# 📊 Monitoring Instance
resource "oci_core_instance" "monitoring" {
  availability_domain = var.availability_domain
  compartment_id      = var.compartment_id
  display_name        = "cerberus-monitoring"
  shape               = "VM.Standard.E3.Flex"

  shape_config {
    ocpus         = 2
    memory_in_gbs = 16
  }

  create_vnic_details {
    subnet_id        = oci_core_subnet.cerberus_subnet.id
    display_name     = "monitoring-vnic"
    assign_public_ip = true
    hostname_label   = "monitoring"
  }

  source_details {
    source_type = "image"
    source_id   = var.ubuntu_image_id
  }

  metadata = {
    ssh_authorized_keys = file(var.ssh_public_key_path)
    user_data = base64encode(file("${path.module}/monitoring-init.sh"))
  }

  freeform_tags = {
    Project = "Cerberus Phoenix Evolved"
    Service = "Monitoring"
    Environment = var.environment
  }
}

# 💾 Block Volumes for persistent storage
resource "oci_core_volume" "vault_storage" {
  availability_domain = var.availability_domain
  compartment_id      = var.compartment_id
  display_name        = "vault-storage"
  size_in_gbs         = 100

  freeform_tags = {
    Project = "Cerberus Phoenix Evolved"
    Service = "Vault"
  }
}

resource "oci_core_volume_attachment" "vault_storage_attachment" {
  attachment_type = "iscsi"
  instance_id     = oci_core_instance.vault.id
  volume_id       = oci_core_volume.vault_storage.id
}
