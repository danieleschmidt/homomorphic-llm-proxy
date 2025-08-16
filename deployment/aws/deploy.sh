#!/bin/bash

# Terragon Labs FHE Proxy - AWS Deployment Script
# This script automates the deployment of the FHE LLM Proxy to AWS using Terraform and Kubernetes

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TERRAFORM_DIR="${SCRIPT_DIR}/terraform"
K8S_DIR="${PROJECT_ROOT}/k8s"

# Default values
ENVIRONMENT=${ENVIRONMENT:-"dev"}
AWS_REGION=${AWS_REGION:-"us-west-2"}
PROJECT_NAME=${PROJECT_NAME:-"fhe-proxy"}
DOMAIN_NAME=${DOMAIN_NAME:-""}
SKIP_CONFIRMATION=${SKIP_CONFIRMATION:-"false"}

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check required tools
    local required_tools=("terraform" "kubectl" "aws" "docker" "jq")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log_error "$tool is required but not installed"
            exit 1
        fi
    done
    
    # Check AWS credentials
    if ! aws sts get-caller-identity &> /dev/null; then
        log_error "AWS credentials not configured. Please run 'aws configure' first"
        exit 1
    fi
    
    # Check Terraform version
    local tf_version=$(terraform version -json | jq -r '.terraform_version')
    log_info "Using Terraform version: $tf_version"
    
    # Check kubectl version
    local kubectl_version=$(kubectl version --client -o json | jq -r '.clientVersion.gitVersion')
    log_info "Using kubectl version: $kubectl_version"
    
    log_success "All prerequisites checked"
}

validate_environment() {
    log_info "Validating environment configuration..."
    
    # Validate environment
    if [[ ! "$ENVIRONMENT" =~ ^(dev|staging|production)$ ]]; then
        log_error "Invalid environment: $ENVIRONMENT. Must be dev, staging, or production"
        exit 1
    fi
    
    # Check for required environment variables
    local required_env_vars=()
    
    if [[ "$ENVIRONMENT" == "production" ]]; then
        required_env_vars+=("DOMAIN_NAME" "DB_PASSWORD" "REDIS_AUTH_TOKEN" "MASTER_API_KEY" "JWT_SECRET")
    fi
    
    for var in "${required_env_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            log_error "Required environment variable $var is not set"
            exit 1
        fi
    done
    
    log_success "Environment validation passed"
}

build_and_push_image() {
    log_info "Building and pushing Docker image..."
    
    # Get AWS account ID and region
    local account_id=$(aws sts get-caller-identity --query Account --output text)
    local ecr_repo="${account_id}.dkr.ecr.${AWS_REGION}.amazonaws.com/${PROJECT_NAME}"
    
    # Login to ECR
    aws ecr get-login-password --region "$AWS_REGION" | docker login --username AWS --password-stdin "$ecr_repo"
    
    # Build image
    log_info "Building Docker image..."
    cd "$PROJECT_ROOT"
    docker build -t "${PROJECT_NAME}:latest" -f Dockerfile .
    
    # Tag for ECR
    docker tag "${PROJECT_NAME}:latest" "${ecr_repo}:latest"
    docker tag "${PROJECT_NAME}:latest" "${ecr_repo}:$(git rev-parse --short HEAD)"
    
    # Push to ECR
    log_info "Pushing image to ECR..."
    docker push "${ecr_repo}:latest"
    docker push "${ecr_repo}:$(git rev-parse --short HEAD)"
    
    log_success "Docker image built and pushed successfully"
    echo "Image URI: ${ecr_repo}:latest"
}

deploy_infrastructure() {
    log_info "Deploying infrastructure with Terraform..."
    
    cd "$TERRAFORM_DIR"
    
    # Initialize Terraform
    log_info "Initializing Terraform..."
    terraform init -upgrade
    
    # Validate Terraform configuration
    log_info "Validating Terraform configuration..."
    terraform validate
    
    # Plan deployment
    log_info "Planning Terraform deployment..."
    terraform plan \
        -var="environment=${ENVIRONMENT}" \
        -var="aws_region=${AWS_REGION}" \
        -var="project_name=${PROJECT_NAME}" \
        -var="domain_name=${DOMAIN_NAME}" \
        -var="db_password=${DB_PASSWORD:-$(openssl rand -base64 32)}" \
        -var="redis_auth_token=${REDIS_AUTH_TOKEN:-$(openssl rand -base64 32)}" \
        -var="master_api_key=${MASTER_API_KEY:-$(openssl rand -base64 32)}" \
        -var="jwt_secret=${JWT_SECRET:-$(openssl rand -base64 32)}" \
        -out=tfplan
    
    # Confirm deployment
    if [[ "$SKIP_CONFIRMATION" != "true" ]]; then
        echo
        log_warning "The above resources will be created/modified. Do you want to continue? (y/N)"
        read -r response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            log_info "Deployment cancelled"
            exit 0
        fi
    fi
    
    # Apply deployment
    log_info "Applying Terraform deployment..."
    terraform apply tfplan
    
    # Save outputs
    terraform output -json > terraform-outputs.json
    
    log_success "Infrastructure deployed successfully"
}

configure_kubectl() {
    log_info "Configuring kubectl..."
    
    cd "$TERRAFORM_DIR"
    
    # Get cluster information from Terraform outputs
    local cluster_name=$(terraform output -raw cluster_id)
    local cluster_region=$(terraform output -raw cluster_region || echo "$AWS_REGION")
    
    # Update kubeconfig
    aws eks update-kubeconfig --region "$cluster_region" --name "$cluster_name"
    
    # Verify connection
    kubectl cluster-info
    kubectl get nodes
    
    log_success "kubectl configured successfully"
}

deploy_kubernetes_manifests() {
    log_info "Deploying Kubernetes manifests..."
    
    cd "$TERRAFORM_DIR"
    
    # Get infrastructure outputs
    local cluster_name=$(terraform output -raw cluster_id)
    local ecr_repository_url=$(terraform output -raw ecr_repository_url)
    local rds_endpoint=$(terraform output -raw rds_endpoint)
    local redis_endpoint=$(terraform output -raw redis_primary_endpoint)
    
    # Create namespace
    kubectl create namespace "$PROJECT_NAME" --dry-run=client -o yaml | kubectl apply -f -
    
    # Create secrets
    log_info "Creating Kubernetes secrets..."
    kubectl create secret generic "${PROJECT_NAME}-secrets" \
        --from-literal=jwt_secret="${JWT_SECRET:-$(openssl rand -base64 32)}" \
        --from-literal=master_api_key="${MASTER_API_KEY:-$(openssl rand -base64 32)}" \
        --from-literal=openai_api_key="${OPENAI_API_KEY:-}" \
        --from-literal=database_url="postgresql://${DB_USERNAME:-fheproxy_user}:${DB_PASSWORD}@${rds_endpoint}:5432/${DB_NAME:-fheproxy}" \
        --from-literal=redis_url="redis://:${REDIS_AUTH_TOKEN}@${redis_endpoint}:6379" \
        --namespace="${PROJECT_NAME}" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Apply ConfigMaps
    log_info "Applying ConfigMaps..."
    kubectl create configmap "${PROJECT_NAME}-config" \
        --from-file="${PROJECT_ROOT}/config.toml" \
        --namespace="${PROJECT_NAME}" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Update deployment manifest with correct image
    local deployment_manifest="${K8S_DIR}/deployment.yaml"
    sed -i.bak "s|image: fhe-proxy:latest|image: ${ecr_repository_url}:latest|g" "$deployment_manifest"
    sed -i.bak "s|namespace: fhe-proxy|namespace: ${PROJECT_NAME}|g" "$deployment_manifest"
    
    # Apply Kubernetes manifests
    log_info "Applying Kubernetes manifests..."
    kubectl apply -f "${K8S_DIR}/namespace.yaml" || true
    kubectl apply -f "${K8S_DIR}/configmap.yaml"
    kubectl apply -f "${K8S_DIR}/secret.yaml"
    kubectl apply -f "${K8S_DIR}/deployment.yaml"
    kubectl apply -f "${K8S_DIR}/service.yaml"
    kubectl apply -f "${K8S_DIR}/ingress.yaml"
    kubectl apply -f "${K8S_DIR}/hpa.yaml"
    
    # Wait for deployment to be ready
    log_info "Waiting for deployment to be ready..."
    kubectl wait --for=condition=available --timeout=600s deployment/"${PROJECT_NAME}" -n "${PROJECT_NAME}"
    
    log_success "Kubernetes manifests deployed successfully"
}

setup_monitoring() {
    log_info "Setting up monitoring..."
    
    # Apply monitoring manifests
    if [[ -f "${PROJECT_ROOT}/k8s-manifests/monitoring.yaml" ]]; then
        kubectl apply -f "${PROJECT_ROOT}/k8s-manifests/monitoring.yaml"
    fi
    
    # Install Prometheus operator if not exists
    if ! kubectl get crd prometheuses.monitoring.coreos.com &> /dev/null; then
        log_info "Installing Prometheus operator..."
        kubectl create -f https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/bundle.yaml
    fi
    
    log_success "Monitoring setup completed"
}

run_health_checks() {
    log_info "Running health checks..."
    
    # Wait for pods to be ready
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name="${PROJECT_NAME}" -n "${PROJECT_NAME}" --timeout=300s
    
    # Get service endpoint
    local service_ip=$(kubectl get service "${PROJECT_NAME}" -n "${PROJECT_NAME}" -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
    if [[ -z "$service_ip" ]]; then
        service_ip=$(kubectl get service "${PROJECT_NAME}" -n "${PROJECT_NAME}" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')
    fi
    
    if [[ -n "$service_ip" ]]; then
        log_info "Testing health endpoint..."
        for i in {1..30}; do
            if curl -f "http://${service_ip}/health" &> /dev/null; then
                log_success "Health check passed"
                break
            fi
            log_info "Waiting for service to be ready... (attempt $i/30)"
            sleep 10
        done
    else
        log_warning "Could not get service IP, skipping external health check"
    fi
    
    # Check pod logs
    log_info "Checking application logs..."
    kubectl logs -l app.kubernetes.io/name="${PROJECT_NAME}" -n "${PROJECT_NAME}" --tail=20
    
    log_success "Health checks completed"
}

cleanup_on_failure() {
    log_error "Deployment failed. Cleaning up..."
    
    # Clean up Kubernetes resources
    kubectl delete namespace "${PROJECT_NAME}" --ignore-not-found=true
    
    # Optionally clean up Terraform resources
    if [[ "${CLEANUP_ON_FAILURE:-false}" == "true" ]]; then
        cd "$TERRAFORM_DIR"
        terraform destroy -auto-approve
    fi
}

print_deployment_summary() {
    log_success "Deployment completed successfully!"
    echo
    echo "=== Deployment Summary ==="
    echo "Environment: $ENVIRONMENT"
    echo "Region: $AWS_REGION"
    echo "Project: $PROJECT_NAME"
    
    cd "$TERRAFORM_DIR"
    if [[ -f terraform-outputs.json ]]; then
        local app_url=$(jq -r '.application_url.value // "N/A"' terraform-outputs.json)
        local cluster_name=$(jq -r '.cluster_id.value // "N/A"' terraform-outputs.json)
        local load_balancer_dns=$(jq -r '.load_balancer_dns_name.value // "N/A"' terraform-outputs.json)
        
        echo "Application URL: $app_url"
        echo "Cluster Name: $cluster_name"
        echo "Load Balancer: $load_balancer_dns"
    fi
    
    echo
    echo "=== Next Steps ==="
    echo "1. Configure DNS to point to the load balancer"
    echo "2. Set up SSL certificate validation"
    echo "3. Configure monitoring dashboards"
    echo "4. Set up backup schedules"
    echo
    echo "=== Useful Commands ==="
    echo "View pods: kubectl get pods -n ${PROJECT_NAME}"
    echo "View logs: kubectl logs -f deployment/${PROJECT_NAME} -n ${PROJECT_NAME}"
    echo "Port forward: kubectl port-forward service/${PROJECT_NAME} 8080:8080 -n ${PROJECT_NAME}"
    echo "Scale deployment: kubectl scale deployment ${PROJECT_NAME} --replicas=5 -n ${PROJECT_NAME}"
}

# Main execution
main() {
    log_info "Starting FHE Proxy deployment to AWS..."
    log_info "Environment: $ENVIRONMENT"
    log_info "Region: $AWS_REGION"
    log_info "Project: $PROJECT_NAME"
    
    # Trap for cleanup on failure
    trap cleanup_on_failure ERR
    
    check_prerequisites
    validate_environment
    build_and_push_image
    deploy_infrastructure
    configure_kubectl
    deploy_kubernetes_manifests
    setup_monitoring
    run_health_checks
    print_deployment_summary
    
    log_success "Deployment completed successfully!"
}

# Handle command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        -r|--region)
            AWS_REGION="$2"
            shift 2
            ;;
        -d|--domain)
            DOMAIN_NAME="$2"
            shift 2
            ;;
        -y|--yes)
            SKIP_CONFIRMATION="true"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  -e, --environment ENV    Deployment environment (dev|staging|production)"
            echo "  -r, --region REGION      AWS region (default: us-west-2)"
            echo "  -d, --domain DOMAIN      Domain name for the application"
            echo "  -y, --yes               Skip confirmation prompts"
            echo "  -h, --help              Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Run main function
main "$@"