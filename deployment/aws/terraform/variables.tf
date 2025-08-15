# AWS Region
variable "aws_region" {
  description = "AWS region for deployment"
  type        = string
  default     = "us-west-2"
}

# Project configuration
variable "project_name" {
  description = "Name of the project"
  type        = string
  default     = "fhe-proxy"
}

variable "environment" {
  description = "Environment name (dev, staging, production)"
  type        = string
  validation {
    condition     = contains(["dev", "staging", "production"], var.environment)
    error_message = "Environment must be dev, staging, or production."
  }
}

variable "domain_name" {
  description = "Domain name for the application"
  type        = string
}

# VPC Configuration
variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "public_subnet_cidrs" {
  description = "CIDR blocks for public subnets"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
}

variable "private_subnet_cidrs" {
  description = "CIDR blocks for private subnets"
  type        = list(string)
  default     = ["10.0.11.0/24", "10.0.12.0/24", "10.0.13.0/24"]
}

# EKS Configuration
variable "kubernetes_version" {
  description = "Kubernetes version for EKS cluster"
  type        = string
  default     = "1.28"
}

variable "cluster_endpoint_public_access_cidrs" {
  description = "List of CIDR blocks that can access the EKS public API endpoint"
  type        = list(string)
  default     = ["0.0.0.0/0"]
}

# EKS Node Group Configuration
variable "node_instance_types" {
  description = "Instance types for EKS worker nodes"
  type        = list(string)
  default     = ["p3.2xlarge", "p3.8xlarge"]  # GPU instances for FHE
}

variable "node_desired_size" {
  description = "Desired number of worker nodes"
  type        = number
  default     = 3
}

variable "node_min_size" {
  description = "Minimum number of worker nodes"
  type        = number
  default     = 1
}

variable "node_max_size" {
  description = "Maximum number of worker nodes"
  type        = number
  default     = 10
}

variable "node_disk_size" {
  description = "Disk size for worker nodes in GB"
  type        = number
  default     = 100
}

# RDS Configuration
variable "rds_instance_class" {
  description = "RDS instance class"
  type        = string
  default     = "db.r6g.large"
}

variable "rds_allocated_storage" {
  description = "Initial allocated storage for RDS in GB"
  type        = number
  default     = 100
}

variable "rds_max_allocated_storage" {
  description = "Maximum allocated storage for RDS in GB"
  type        = number
  default     = 1000
}

variable "db_name" {
  description = "Database name"
  type        = string
  default     = "fheproxy"
}

variable "db_username" {
  description = "Database username"
  type        = string
  default     = "fheproxy_user"
}

variable "db_password" {
  description = "Database password"
  type        = string
  sensitive   = true
}

# ElastiCache Configuration
variable "redis_node_type" {
  description = "ElastiCache Redis node type"
  type        = string
  default     = "cache.r6g.large"
}

variable "redis_num_cache_nodes" {
  description = "Number of cache nodes for Redis"
  type        = number
  default     = 2
}

variable "redis_auth_token" {
  description = "Auth token for Redis cluster"
  type        = string
  sensitive   = true
}

# Application Configuration
variable "container_image" {
  description = "Container image for FHE proxy"
  type        = string
  default     = "terragonlabs/homomorphic-llm-proxy:latest"
}

variable "app_replicas" {
  description = "Number of application replicas"
  type        = number
  default     = 3
}

variable "app_cpu_request" {
  description = "CPU request for app containers"
  type        = string
  default     = "1000m"
}

variable "app_cpu_limit" {
  description = "CPU limit for app containers"
  type        = string
  default     = "4000m"
}

variable "app_memory_request" {
  description = "Memory request for app containers"
  type        = string
  default     = "2Gi"
}

variable "app_memory_limit" {
  description = "Memory limit for app containers"
  type        = string
  default     = "8Gi"
}

# Monitoring Configuration
variable "enable_monitoring" {
  description = "Enable monitoring stack (Prometheus, Grafana)"
  type        = bool
  default     = true
}

variable "monitoring_retention_days" {
  description = "Monitoring data retention in days"
  type        = number
  default     = 30
}

# Security Configuration
variable "enable_waf" {
  description = "Enable AWS WAF"
  type        = bool
  default     = true
}

variable "enable_encryption_at_rest" {
  description = "Enable encryption at rest for all services"
  type        = bool
  default     = true
}

variable "enable_encryption_in_transit" {
  description = "Enable encryption in transit for all services"
  type        = bool
  default     = true
}

# API Keys (should be set via environment variables or tfvars)
variable "openai_api_key" {
  description = "OpenAI API key for LLM integration"
  type        = string
  sensitive   = true
  default     = ""
}

variable "anthropic_api_key" {
  description = "Anthropic API key for Claude integration"
  type        = string
  sensitive   = true
  default     = ""
}

variable "master_api_key" {
  description = "Master API key for FHE proxy administration"
  type        = string
  sensitive   = true
}

variable "jwt_secret" {
  description = "JWT secret for token signing"
  type        = string
  sensitive   = true
}

# Backup Configuration
variable "backup_retention_days" {
  description = "Number of days to retain backups"
  type        = number
  default     = 30
}

variable "enable_point_in_time_recovery" {
  description = "Enable point-in-time recovery for RDS"
  type        = bool
  default     = true
}

# Auto Scaling Configuration
variable "enable_cluster_autoscaler" {
  description = "Enable Kubernetes cluster autoscaler"
  type        = bool
  default     = true
}

variable "enable_hpa" {
  description = "Enable Horizontal Pod Autoscaler"
  type        = bool
  default     = true
}

variable "hpa_target_cpu_utilization" {
  description = "Target CPU utilization for HPA"
  type        = number
  default     = 70
}

variable "hpa_target_memory_utilization" {
  description = "Target memory utilization for HPA"
  type        = number
  default     = 80
}

# GPU Configuration
variable "gpu_node_group_enabled" {
  description = "Enable dedicated GPU node group for FHE computations"
  type        = bool
  default     = true
}

variable "gpu_instance_types" {
  description = "GPU instance types for FHE computations"
  type        = list(string)
  default     = ["p3.2xlarge", "p3.8xlarge", "p4d.xlarge"]
}

variable "gpu_node_desired_size" {
  description = "Desired size for GPU node group"
  type        = number
  default     = 2
}

variable "gpu_node_min_size" {
  description = "Minimum size for GPU node group"
  type        = number
  default     = 0
}

variable "gpu_node_max_size" {
  description = "Maximum size for GPU node group"
  type        = number
  default     = 5
}

# Cost Optimization
variable "enable_spot_instances" {
  description = "Enable spot instances for cost optimization"
  type        = bool
  default     = false
}

variable "spot_instance_pools" {
  description = "Number of spot instance pools"
  type        = number
  default     = 2
}

# Compliance and Governance
variable "enable_compliance_monitoring" {
  description = "Enable compliance monitoring and auditing"
  type        = bool
  default     = true
}

variable "data_residency_region" {
  description = "Data residency requirement (region constraint)"
  type        = string
  default     = ""
}

variable "enable_data_encryption" {
  description = "Enable data encryption for compliance"
  type        = bool
  default     = true
}

# Performance Configuration
variable "fhe_performance_tier" {
  description = "FHE performance tier (basic, optimized, enterprise)"
  type        = string
  default     = "optimized"
  validation {
    condition     = contains(["basic", "optimized", "enterprise"], var.fhe_performance_tier)
    error_message = "Performance tier must be basic, optimized, or enterprise."
  }
}

variable "enable_performance_insights" {
  description = "Enable performance insights for databases"
  type        = bool
  default     = true
}

# Disaster Recovery
variable "enable_multi_region_backup" {
  description = "Enable multi-region backup for disaster recovery"
  type        = bool
  default     = false
}

variable "dr_region" {
  description = "Disaster recovery region"
  type        = string
  default     = "us-east-1"
}

# Tags
variable "additional_tags" {
  description = "Additional tags to apply to all resources"
  type        = map(string)
  default     = {}
}