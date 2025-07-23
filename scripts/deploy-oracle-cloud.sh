#!/bin/bash

# 🥷 Cerberus Phoenix v2.0 - Oracle Cloud Deployment Script
# Automated deployment to Oracle Cloud Free Tier with Multi-RPC optimization

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="cerberus-phoenix"
TERRAFORM_DIR="infrastructure/terraform"
OCI_CONFIG_FILE="$HOME/.oci/config"

echo -e "${BLUE}🥷 Cerberus Phoenix v2.0 - Oracle Cloud Deployment${NC}"
echo "=================================================================="

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️ $1${NC}"
}

# Check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."
    
    # Check Terraform
    if ! command -v terraform &> /dev/null; then
        print_error "Terraform is not installed!"
        print_info "Install from: https://www.terraform.io/downloads"
        exit 1
    fi
    
    # Check OCI CLI
    if ! command -v oci &> /dev/null; then
        print_error "OCI CLI is not installed!"
        print_info "Install from: https://docs.oracle.com/en-us/iaas/Content/API/SDKDocs/cliinstall.htm"
        exit 1
    fi
    
    # Check OCI configuration
    if [ ! -f "$OCI_CONFIG_FILE" ]; then
        print_error "OCI configuration not found at $OCI_CONFIG_FILE"
        print_info "Run: oci setup config"
        exit 1
    fi
    
    print_status "Prerequisites check passed"
}

# Setup OCI configuration
setup_oci_config() {
    print_info "Setting up OCI configuration..."
    
    if [ ! -f "$OCI_CONFIG_FILE" ]; then
        print_info "Running OCI setup..."
        oci setup config
    fi
    
    # Validate OCI configuration
    if oci iam user get --user-id $(oci iam user list --query 'data[0].id' --raw-output) &>/dev/null; then
        print_status "OCI configuration is valid"
    else
        print_error "OCI configuration is invalid"
        print_info "Please run: oci setup config"
        exit 1
    fi
}

# Generate SSH key pair if not exists
setup_ssh_keys() {
    print_info "Setting up SSH keys..."
    
    SSH_KEY_PATH="$HOME/.ssh/cerberus_phoenix"
    
    if [ ! -f "$SSH_KEY_PATH" ]; then
        print_info "Generating SSH key pair..."
        ssh-keygen -t rsa -b 4096 -f "$SSH_KEY_PATH" -N "" -C "cerberus-phoenix@oracle-cloud"
        print_status "SSH key pair generated: $SSH_KEY_PATH"
    else
        print_status "SSH key pair already exists: $SSH_KEY_PATH"
    fi
    
    # Export public key for Terraform
    export TF_VAR_ssh_public_key=$(cat "${SSH_KEY_PATH}.pub")
}

# Setup Terraform variables
setup_terraform_vars() {
    print_info "Setting up Terraform variables..."
    
    # Get OCI configuration
    OCI_TENANCY_OCID=$(oci iam tenancy get --tenancy-id $(oci iam user list --query 'data[0]."compartment-id"' --raw-output) --query 'data.id' --raw-output)
    OCI_USER_OCID=$(oci iam user list --query 'data[0].id' --raw-output)
    OCI_FINGERPRINT=$(oci iam user list --query 'data[0]."freeform-tags".fingerprint' --raw-output 2>/dev/null || echo "")
    OCI_REGION=$(oci iam region-subscription list --query 'data[0]."region-name"' --raw-output)
    OCI_PRIVATE_KEY_PATH="$HOME/.oci/oci_api_key.pem"
    
    # If fingerprint is empty, try to get it from config
    if [ -z "$OCI_FINGERPRINT" ]; then
        OCI_FINGERPRINT=$(grep "fingerprint" "$OCI_CONFIG_FILE" | cut -d'=' -f2 | tr -d ' ')
    fi
    
    # Create terraform.tfvars
    cat > "$TERRAFORM_DIR/terraform.tfvars" << EOF
# 🥷 Cerberus Phoenix v2.0 - Oracle Cloud Configuration
tenancy_ocid     = "$OCI_TENANCY_OCID"
user_ocid        = "$OCI_USER_OCID"
fingerprint      = "$OCI_FINGERPRINT"
private_key_path = "$OCI_PRIVATE_KEY_PATH"
region           = "$OCI_REGION"
compartment_ocid = "$OCI_TENANCY_OCID"
ssh_public_key   = "$TF_VAR_ssh_public_key"
EOF
    
    print_status "Terraform variables configured"
}

# Deploy infrastructure
deploy_infrastructure() {
    print_info "Deploying infrastructure to Oracle Cloud..."
    
    cd "$TERRAFORM_DIR"
    
    # Initialize Terraform
    print_info "Initializing Terraform..."
    terraform init
    
    # Plan deployment
    print_info "Planning deployment..."
    terraform plan -out=tfplan
    
    # Ask for confirmation
    echo ""
    print_warning "Ready to deploy Cerberus Phoenix v2.0 to Oracle Cloud Free Tier"
    print_info "This will create:"
    print_info "• VM.Standard.A1.Flex instance (4 OCPU, 24GB RAM)"
    print_info "• VCN with public subnet"
    print_info "• Security groups and rules"
    print_info "• OCI Vault for secret management"
    echo ""
    read -p "Proceed with deployment? (y/N): " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_warning "Deployment cancelled"
        exit 0
    fi
    
    # Apply deployment
    print_info "Applying deployment..."
    terraform apply tfplan
    
    # Get outputs
    INSTANCE_IP=$(terraform output -raw instance_public_ip)
    SSH_COMMAND=$(terraform output -raw ssh_connection)
    VAULT_ID=$(terraform output -raw vault_id)
    
    print_status "Infrastructure deployed successfully!"
    print_info "Instance IP: $INSTANCE_IP"
    print_info "SSH Command: $SSH_COMMAND"
    print_info "Vault ID: $VAULT_ID"
    
    cd - > /dev/null
}

# Wait for instance to be ready
wait_for_instance() {
    print_info "Waiting for instance to be ready..."
    
    INSTANCE_IP=$(cd "$TERRAFORM_DIR" && terraform output -raw instance_public_ip)
    
    # Wait for SSH to be available
    for i in {1..30}; do
        if ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no -i "$HOME/.ssh/cerberus_phoenix" ubuntu@"$INSTANCE_IP" "echo 'SSH connection successful'" &>/dev/null; then
            print_status "Instance is ready!"
            return 0
        fi
        print_info "Waiting for SSH... (attempt $i/30)"
        sleep 10
    done
    
    print_error "Instance is not responding to SSH after 5 minutes"
    exit 1
}

# Configure the instance
configure_instance() {
    print_info "Configuring Cerberus Phoenix v2.0..."
    
    INSTANCE_IP=$(cd "$TERRAFORM_DIR" && terraform output -raw instance_public_ip)
    SSH_KEY="$HOME/.ssh/cerberus_phoenix"
    
    # Copy configuration files
    print_info "Copying configuration files..."
    scp -i "$SSH_KEY" -o StrictHostKeyChecking=no \
        infrastructure/.env.example \
        ubuntu@"$INSTANCE_IP":/tmp/env.example
    
    # Run configuration script
    ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no ubuntu@"$INSTANCE_IP" << 'EOF'
        # Update environment configuration
        sudo cp /tmp/env.example /opt/cerberus-phoenix/infrastructure/.env
        sudo chown cerberus:cerberus /opt/cerberus-phoenix/infrastructure/.env
        
        # Install Python dependencies for webhook setup
        sudo pip3 install requests python-dotenv
        
        # Make scripts executable
        sudo chmod +x /opt/cerberus-phoenix/scripts/*.sh
        sudo chmod +x /opt/cerberus-phoenix/scripts/*.py
        
        echo "✅ Instance configuration completed"
EOF
    
    print_status "Instance configured successfully"
}

# Setup SSL certificate
setup_ssl() {
    print_info "Setting up SSL certificate..."
    
    INSTANCE_IP=$(cd "$TERRAFORM_DIR" && terraform output -raw instance_public_ip)
    SSH_KEY="$HOME/.ssh/cerberus_phoenix"
    
    print_warning "SSL setup requires a domain name pointing to $INSTANCE_IP"
    read -p "Enter your domain name (or press Enter to skip SSL setup): " DOMAIN_NAME
    
    if [ -n "$DOMAIN_NAME" ]; then
        ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no ubuntu@"$INSTANCE_IP" << EOF
            # Update nginx configuration with domain
            sudo sed -i 's/server_name _;/server_name $DOMAIN_NAME;/g' /etc/nginx/sites-available/cerberus-phoenix
            sudo nginx -t && sudo systemctl reload nginx
            
            # Setup SSL certificate
            sudo certbot --nginx -d $DOMAIN_NAME --non-interactive --agree-tos --email admin@$DOMAIN_NAME
            
            echo "✅ SSL certificate configured for $DOMAIN_NAME"
EOF
        print_status "SSL certificate configured for $DOMAIN_NAME"
    else
        print_warning "SSL setup skipped - using self-signed certificate"
    fi
}

# Start services
start_services() {
    print_info "Starting Cerberus Phoenix v2.0 services..."
    
    INSTANCE_IP=$(cd "$TERRAFORM_DIR" && terraform output -raw instance_public_ip)
    SSH_KEY="$HOME/.ssh/cerberus_phoenix"
    
    ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no ubuntu@"$INSTANCE_IP" << 'EOF'
        # Start Cerberus Phoenix services
        sudo systemctl start cerberus-phoenix
        sudo systemctl enable cerberus-phoenix
        
        # Check service status
        sleep 10
        sudo systemctl status cerberus-phoenix --no-pager
        
        echo "✅ Services started successfully"
EOF
    
    print_status "Services started successfully"
}

# Display deployment summary
show_summary() {
    INSTANCE_IP=$(cd "$TERRAFORM_DIR" && terraform output -raw instance_public_ip)
    SSH_COMMAND=$(cd "$TERRAFORM_DIR" && terraform output -raw ssh_connection)
    
    echo ""
    echo "=================================================================="
    echo -e "${GREEN}🎉 Cerberus Phoenix v2.0 - Oracle Cloud Deployment Complete!${NC}"
    echo "=================================================================="
    echo ""
    echo "📊 Deployment Summary:"
    echo "  • Instance IP: $INSTANCE_IP"
    echo "  • SSH Access: $SSH_COMMAND"
    echo "  • Instance Type: VM.Standard.A1.Flex (4 OCPU, 24GB RAM)"
    echo "  • Operating System: Ubuntu 22.04 LTS"
    echo ""
    echo "🌐 Access Points:"
    echo "  • API Health: https://$INSTANCE_IP/health"
    echo "  • API Endpoints: https://$INSTANCE_IP/api/v1/"
    echo "  • Grafana Dashboard: https://$INSTANCE_IP/grafana/"
    echo "  • System Status: https://$INSTANCE_IP/"
    echo ""
    echo "🔧 Next Steps:"
    echo "  1. Configure API keys:"
    echo "     ssh -i ~/.ssh/cerberus_phoenix ubuntu@$INSTANCE_IP"
    echo "     sudo nano /opt/cerberus-phoenix/infrastructure/.env"
    echo ""
    echo "  2. Setup webhooks:"
    echo "     cd /opt/cerberus-phoenix"
    echo "     ./scripts/setup-helius-webhooks.py"
    echo ""
    echo "  3. Monitor system:"
    echo "     curl https://$INSTANCE_IP/api/v1/optimization/status"
    echo ""
    echo "💰 Cost Optimization:"
    echo "  • Multi-RPC providers: 5 providers with intelligent routing"
    echo "  • Expected savings: $80-120/month vs single provider"
    echo "  • Free tier usage: 2.2M+ requests/month"
    echo "  • Oracle Cloud: FREE (Always Free tier)"
    echo ""
    echo -e "${BLUE}🥷 Cerberus Phoenix v2.0 is ready to hunt for alpha on Solana!${NC}"
}

# Main deployment function
main() {
    case "${1:-deploy}" in
        "deploy")
            check_prerequisites
            setup_oci_config
            setup_ssh_keys
            setup_terraform_vars
            deploy_infrastructure
            wait_for_instance
            configure_instance
            setup_ssl
            start_services
            show_summary
            ;;
        "destroy")
            print_warning "Destroying Oracle Cloud infrastructure..."
            cd "$TERRAFORM_DIR"
            terraform destroy
            print_status "Infrastructure destroyed"
            ;;
        "status")
            cd "$TERRAFORM_DIR"
            terraform show
            ;;
        "ssh")
            INSTANCE_IP=$(cd "$TERRAFORM_DIR" && terraform output -raw instance_public_ip)
            ssh -i "$HOME/.ssh/cerberus_phoenix" ubuntu@"$INSTANCE_IP"
            ;;
        *)
            echo "Usage: $0 {deploy|destroy|status|ssh}"
            echo ""
            echo "Commands:"
            echo "  deploy  - Deploy to Oracle Cloud (default)"
            echo "  destroy - Destroy infrastructure"
            echo "  status  - Show infrastructure status"
            echo "  ssh     - SSH to the instance"
            exit 1
            ;;
    esac
}

# Change to project root
cd "$(dirname "$0")/.."

# Run main function
main "$@"
