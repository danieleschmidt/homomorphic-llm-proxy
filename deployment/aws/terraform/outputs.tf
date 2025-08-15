# VPC Outputs
output "vpc_id" {
  description = "VPC ID"
  value       = aws_vpc.main.id
}

output "vpc_cidr_block" {
  description = "VPC CIDR block"
  value       = aws_vpc.main.cidr_block
}

output "public_subnet_ids" {
  description = "List of public subnet IDs"
  value       = aws_subnet.public[*].id
}

output "private_subnet_ids" {
  description = "List of private subnet IDs"
  value       = aws_subnet.private[*].id
}

# EKS Cluster Outputs
output "cluster_id" {
  description = "EKS cluster ID"
  value       = aws_eks_cluster.main.id
}

output "cluster_arn" {
  description = "EKS cluster ARN"
  value       = aws_eks_cluster.main.arn
}

output "cluster_endpoint" {
  description = "Endpoint for EKS control plane"
  value       = aws_eks_cluster.main.endpoint
}

output "cluster_security_group_id" {
  description = "Security group ID attached to the EKS cluster"
  value       = aws_eks_cluster.main.vpc_config[0].cluster_security_group_id
}

output "cluster_certificate_authority_data" {
  description = "Base64 encoded certificate data required to communicate with the cluster"
  value       = aws_eks_cluster.main.certificate_authority[0].data
}

output "cluster_version" {
  description = "The Kubernetes version for the EKS cluster"
  value       = aws_eks_cluster.main.version
}

# EKS Node Group Outputs
output "node_group_id" {
  description = "EKS node group ID"
  value       = aws_eks_node_group.main.id
}

output "node_group_arn" {
  description = "Amazon Resource Name (ARN) of the EKS Node Group"
  value       = aws_eks_node_group.main.arn
}

output "node_group_status" {
  description = "Status of the EKS Node Group"
  value       = aws_eks_node_group.main.status
}

# RDS Outputs
output "rds_endpoint" {
  description = "RDS instance endpoint"
  value       = aws_db_instance.main.endpoint
  sensitive   = true
}

output "rds_port" {
  description = "RDS instance port"
  value       = aws_db_instance.main.port
}

output "rds_identifier" {
  description = "RDS instance identifier"
  value       = aws_db_instance.main.identifier
}

output "rds_arn" {
  description = "RDS instance ARN"
  value       = aws_db_instance.main.arn
}

# ElastiCache Outputs
output "redis_cluster_id" {
  description = "ElastiCache Redis cluster ID"
  value       = aws_elasticache_replication_group.main.id
}

output "redis_configuration_endpoint" {
  description = "ElastiCache Redis configuration endpoint"
  value       = aws_elasticache_replication_group.main.configuration_endpoint_address
  sensitive   = true
}

output "redis_primary_endpoint" {
  description = "ElastiCache Redis primary endpoint"
  value       = aws_elasticache_replication_group.main.primary_endpoint_address
  sensitive   = true
}

output "redis_port" {
  description = "ElastiCache Redis port"
  value       = aws_elasticache_replication_group.main.port
}

# Load Balancer Outputs
output "load_balancer_id" {
  description = "Application Load Balancer ID"
  value       = aws_lb.main.id
}

output "load_balancer_arn" {
  description = "Application Load Balancer ARN"
  value       = aws_lb.main.arn
}

output "load_balancer_dns_name" {
  description = "DNS name of the load balancer"
  value       = aws_lb.main.dns_name
}

output "load_balancer_hosted_zone_id" {
  description = "Hosted zone ID of the load balancer"
  value       = aws_lb.main.zone_id
}

output "target_group_arn" {
  description = "Target group ARN"
  value       = aws_lb_target_group.main.arn
}

# Security Outputs
output "cluster_security_group_ids" {
  description = "Security group IDs attached to the EKS cluster"
  value       = [aws_security_group.eks_cluster.id]
}

output "node_security_group_ids" {
  description = "Security group IDs attached to the EKS nodes"
  value       = [aws_security_group.eks_nodes.id]
}

output "rds_security_group_id" {
  description = "Security group ID for RDS"
  value       = aws_security_group.rds.id
}

output "redis_security_group_id" {
  description = "Security group ID for ElastiCache Redis"
  value       = aws_security_group.elasticache.id
}

output "alb_security_group_id" {
  description = "Security group ID for Application Load Balancer"
  value       = aws_security_group.alb.id
}

# S3 Outputs
output "artifacts_bucket_id" {
  description = "Artifacts S3 bucket ID"
  value       = aws_s3_bucket.artifacts.id
}

output "artifacts_bucket_arn" {
  description = "Artifacts S3 bucket ARN"
  value       = aws_s3_bucket.artifacts.arn
}

output "artifacts_bucket_domain_name" {
  description = "Artifacts S3 bucket domain name"
  value       = aws_s3_bucket.artifacts.bucket_domain_name
}

# ECR Outputs
output "ecr_repository_url" {
  description = "ECR repository URL"
  value       = aws_ecr_repository.main.repository_url
}

output "ecr_repository_arn" {
  description = "ECR repository ARN"
  value       = aws_ecr_repository.main.arn
}

# Certificate Outputs
output "certificate_arn" {
  description = "ACM certificate ARN"
  value       = aws_acm_certificate.main.arn
}

output "certificate_domain_name" {
  description = "Domain name for which the certificate is issued"
  value       = aws_acm_certificate.main.domain_name
}

# WAF Outputs
output "waf_web_acl_id" {
  description = "WAF Web ACL ID"
  value       = aws_wafv2_web_acl.main.id
}

output "waf_web_acl_arn" {
  description = "WAF Web ACL ARN"
  value       = aws_wafv2_web_acl.main.arn
}

# KMS Outputs
output "kms_key_id" {
  description = "KMS key ID for EKS encryption"
  value       = aws_kms_key.eks.id
}

output "kms_key_arn" {
  description = "KMS key ARN for EKS encryption"
  value       = aws_kms_key.eks.arn
}

# IAM Outputs
output "cluster_iam_role_name" {
  description = "IAM role name associated with EKS cluster"
  value       = aws_iam_role.eks_cluster.name
}

output "cluster_iam_role_arn" {
  description = "IAM role ARN associated with EKS cluster"
  value       = aws_iam_role.eks_cluster.arn
}

output "node_group_iam_role_name" {
  description = "IAM role name associated with EKS node group"
  value       = aws_iam_role.eks_node_group.name
}

output "node_group_iam_role_arn" {
  description = "IAM role ARN associated with EKS node group"
  value       = aws_iam_role.eks_node_group.arn
}

# Monitoring Outputs
output "cloudwatch_log_group_name" {
  description = "CloudWatch log group name for EKS cluster"
  value       = aws_cloudwatch_log_group.eks_cluster.name
}

output "cloudwatch_log_group_arn" {
  description = "CloudWatch log group ARN for EKS cluster"
  value       = aws_cloudwatch_log_group.eks_cluster.arn
}

# Application Configuration Outputs
output "application_url" {
  description = "Application URL (HTTPS)"
  value       = "https://${var.domain_name}"
}

output "application_health_check_url" {
  description = "Application health check URL"
  value       = "https://${var.domain_name}/health"
}

output "application_metrics_url" {
  description = "Application metrics URL (internal)"
  value       = "http://${aws_lb.main.dns_name}:9090/metrics"
}

# Connection Information for Application
output "database_connection_info" {
  description = "Database connection information"
  value = {
    host     = aws_db_instance.main.endpoint
    port     = aws_db_instance.main.port
    dbname   = aws_db_instance.main.db_name
    username = aws_db_instance.main.username
  }
  sensitive = true
}

output "redis_connection_info" {
  description = "Redis connection information"
  value = {
    endpoint = aws_elasticache_replication_group.main.primary_endpoint_address
    port     = aws_elasticache_replication_group.main.port
  }
  sensitive = true
}

# Kubectl Configuration
output "kubectl_config" {
  description = "kubectl config to connect to the cluster"
  value = {
    cluster_name     = aws_eks_cluster.main.name
    endpoint         = aws_eks_cluster.main.endpoint
    ca_data          = aws_eks_cluster.main.certificate_authority[0].data
    region           = var.aws_region
  }
  sensitive = true
}

# Environment Summary
output "deployment_summary" {
  description = "Summary of deployed infrastructure"
  value = {
    environment          = var.environment
    region              = var.aws_region
    cluster_name        = aws_eks_cluster.main.name
    cluster_version     = aws_eks_cluster.main.version
    load_balancer_dns   = aws_lb.main.dns_name
    domain_name         = var.domain_name
    database_engine     = aws_db_instance.main.engine
    redis_engine        = "redis"
    monitoring_enabled  = var.enable_monitoring
    waf_enabled        = var.enable_waf
  }
}

# Cost Estimation Helpers
output "estimated_monthly_costs" {
  description = "Estimated monthly costs breakdown (approximation)"
  value = {
    eks_cluster        = "~$73/month (control plane)"
    node_group_compute = "Variable based on instance types and count"
    rds_database      = "Variable based on ${var.rds_instance_class}"
    elasticache       = "Variable based on ${var.redis_node_type}"
    load_balancer     = "~$22/month"
    data_transfer     = "Variable based on usage"
    storage           = "Variable based on usage"
    note              = "These are rough estimates. Actual costs depend on usage patterns."
  }
}

# Security and Compliance
output "security_features_enabled" {
  description = "Security features that are enabled"
  value = {
    encryption_at_rest    = var.enable_encryption_at_rest
    encryption_in_transit = var.enable_encryption_in_transit
    waf_protection       = var.enable_waf
    vpc_security_groups  = true
    iam_roles           = true
    secrets_encryption  = true
    network_isolation   = true
  }
}