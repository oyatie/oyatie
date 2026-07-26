# aws-guest OpenTofu module for the imaging µservice.
# Authority: ADR-0131 per-µservice flat layout + zero-handroll OpenTofu-only.
# Use case: AI inference compute fan-out (GPU-bound), paired with on-prem
# PACS over VPN. PACS substrate itself does NOT run on aws-guest in this
# pattern; only AI inference workers + ephemeral compute.
#
# Tenant: paid only. Demo_trial does not deploy aws-guest.
# Cell: tier-1 or tier-2. Sovereign-cell is on-prem in this pattern.

terraform {
  required_version = ">= 1.7"
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.50" }
  }
}

variable "tenant_id" { type = string }
variable "cell_id"   { type = string }
variable "region" {
  type = string
  default = "us-east-1"
}
variable "ai_vendor_egress_subnet_cidrs" {
  type    = list(string)
  default = []
  description = "Allow-listed AI vendor egress CIDRs"
}

resource "aws_vpc" "imaging" {
  cidr_block = "10.42.0.0/16"
  enable_dns_hostnames = true
  tags = {
    "oyatie:microservice" = "imaging"
    "oyatie:tenant"       = var.tenant_id
    "oyatie:cell"         = var.cell_id
    "oyatie:context"      = "aws-guest"
  }
}

resource "aws_subnet" "ai_inference" {
  vpc_id                  = aws_vpc.imaging.id
  cidr_block              = "10.42.10.0/24"
  availability_zone       = "${var.region}a"
  map_public_ip_on_launch = false
  tags = {
    "oyatie:purpose" = "ai-inference"
  }
}

resource "aws_eks_cluster" "imaging" {
  name     = "imaging-${var.tenant_id}-${var.cell_id}"
  role_arn = aws_iam_role.eks.arn
  version  = "1.30"
  vpc_config {
    subnet_ids              = [aws_subnet.ai_inference.id]
    endpoint_private_access = true
    endpoint_public_access  = false
  }
  encryption_config {
    provider {
      key_arn = aws_kms_key.imaging.arn
    }
    resources = ["secrets"]
  }
}

resource "aws_kms_key" "imaging" {
  description = "Imaging at-rest envelope key (BYOK opt-in per ADR-0255 §D-4)"
  enable_key_rotation = true
}

resource "aws_eks_node_group" "ai_inference_gpu" {
  cluster_name    = aws_eks_cluster.imaging.name
  node_group_name = "ai-inference-gpu"
  node_role_arn   = aws_iam_role.node.arn
  subnet_ids      = [aws_subnet.ai_inference.id]
  instance_types  = ["g5.2xlarge"]
  scaling_config {
    desired_size = 2
    max_size     = 20
    min_size     = 1
  }
  ami_type = "AL2023_x86_64_NVIDIA"
}

resource "aws_iam_role" "eks" {
  name = "imaging-eks-${var.tenant_id}-${var.cell_id}"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Principal = { Service = "eks.amazonaws.com" }
      Action = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role" "node" {
  name = "imaging-node-${var.tenant_id}-${var.cell_id}"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Principal = { Service = "ec2.amazonaws.com" }
      Action = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy_attachment" "node_worker" {
  role       = aws_iam_role.node.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSWorkerNodePolicy"
}

output "cluster_endpoint" {
  value     = aws_eks_cluster.imaging.endpoint
  sensitive = true
}

output "kms_key_arn" {
  value = aws_kms_key.imaging.arn
}
