# 🐺 Cerberus Phoenix Evolved - Terraform Variables

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
  default     = "us-ashburn-1"
}

variable "compartment_id" {
  description = "OCID of the compartment"
  type        = string
}

variable "availability_domain" {
  description = "Availability domain for resources"
  type        = string
}

variable "ubuntu_image_id" {
  description = "OCID of the Ubuntu image"
  type        = string
  # Ubuntu 22.04 LTS in us-ashburn-1
  default = "ocid1.image.oc1.iad.aaaaaaaaba6mxohpjzlzbt2kds3hv2qvqzqvt2lqcjp7qx3lfqbkzjdkq7nq"
}

variable "ssh_public_key_path" {
  description = "Path to the SSH public key file"
  type        = string
  default     = "~/.ssh/id_rsa.pub"
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "dev"
}

variable "project_name" {
  description = "Project name for tagging"
  type        = string
  default     = "cerberus-phoenix-evolved"
}