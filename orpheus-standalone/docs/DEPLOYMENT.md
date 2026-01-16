# Maestro Platform - Production Deployment Guide

Complete guide for deploying Maestro to production.

---

## Table of Contents

1. [Quick Start (Development)](#quick-start-development)
2. [Docker Deployment](#docker-deployment)
3. [Kubernetes Deployment](#kubernetes-deployment)
4. [Cloud Platform Specific](#cloud-platform-specific)
5. [Performance Tuning](#performance-tuning)
6. [Monitoring & Observability](#monitoring--observability)
7. [Security Hardening](#security-hardening)
8. [Backup & Disaster Recovery](#backup--disaster-recovery)

---

## Quick Start (Development)

### Prerequisites

- Docker & Docker Compose
- Node.js 18+
- Java 17+
- Rust 1.75+
- PostgreSQL 15+

### Start All Services

```bash
# Start the complete platform
./scripts/start-all.sh

# Test the audio pipeline
./scripts/test-audio-pipeline.sh

# Stop all services
./scripts/stop-all.sh
```

### Individual Services

```bash
# Data layer
cd maestro/backend
docker-compose up -d

# Backend API
cd maestro/backend
./gradlew bootRun

# Audio service
cd maestro-audio
cargo run --release --bin maestro-audio-server

# Web app
cd maestro/packages/app-web
npm run dev
```

---

## Docker Deployment

### Single-Host Docker Compose

```bash
# Build and start all services
docker-compose -f docker-compose.full-stack.yml up -d

# View logs
docker-compose -f docker-compose.full-stack.yml logs -f

# Stop all services
docker-compose -f docker-compose.full-stack.yml down
```

### Build Images

```bash
# Backend
cd maestro/backend
docker build -t maestro-backend:latest .

# Audio service
cd maestro-audio
docker build -t maestro-audio-service:latest .

# Web app
cd maestro/packages/app-web
docker build -t maestro-web:latest .
```

### Push to Registry

```bash
# Tag for your registry
docker tag maestro-backend:latest your-registry.com/maestro-backend:v1.0.0
docker tag maestro-audio-service:latest your-registry.com/maestro-audio-service:v1.0.0
docker tag maestro-web:latest your-registry.com/maestro-web:v1.0.0

# Push
docker push your-registry.com/maestro-backend:v1.0.0
docker push your-registry.com/maestro-audio-service:v1.0.0
docker push your-registry.com/maestro-web:v1.0.0
```

---

## Kubernetes Deployment

### Create Namespace

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: maestro
```

### PostgreSQL StatefulSet

```yaml
# k8s/postgres.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
  namespace: maestro
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 20Gi
---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: maestro
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        ports:
        - containerPort: 5432
        env:
        - name: POSTGRES_DB
          value: maestro
        - name: POSTGRES_USER
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: username
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: maestro
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
```

### Backend Deployment

```yaml
# k8s/backend.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: maestro-backend
  namespace: maestro
spec:
  replicas: 3
  selector:
    matchLabels:
      app: maestro-backend
  template:
    metadata:
      labels:
        app: maestro-backend
    spec:
      containers:
      - name: backend
        image: your-registry.com/maestro-backend:v1.0.0
        ports:
        - containerPort: 8080
        env:
        - name: DB_HOST
          value: postgres
        - name: DB_PORT
          value: "5432"
        - name: DB_NAME
          value: maestro
        - name: DB_USER
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: username
        - name: DB_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        - name: REDIS_HOST
          value: redis
        - name: AUDIO_SERVICE_HOST
          value: audio-service
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /actuator/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /actuator/health
            port: 8080
          initialDelaySeconds: 20
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: backend
  namespace: maestro
spec:
  selector:
    app: maestro-backend
  type: LoadBalancer
  ports:
  - port: 8080
    targetPort: 8080
```

### Audio Service Deployment

```yaml
# k8s/audio-service.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: maestro-audio-service
  namespace: maestro
spec:
  replicas: 2
  selector:
    matchLabels:
      app: maestro-audio-service
  template:
    metadata:
      labels:
        app: maestro-audio-service
    spec:
      containers:
      - name: audio-service
        image: your-registry.com/maestro-audio-service:v1.0.0
        ports:
        - containerPort: 50051
        env:
        - name: RUST_LOG
          value: info
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "512Mi"
            cpu: "1000m"
---
apiVersion: v1
kind: Service
metadata:
  name: audio-service
  namespace: maestro
spec:
  selector:
    app: maestro-audio-service
  type: ClusterIP
  ports:
  - port: 50051
    targetPort: 50051
    protocol: TCP
```

### Apply Kubernetes Manifests

```bash
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/postgres-secret.yaml
kubectl apply -f k8s/postgres.yaml
kubectl apply -f k8s/redis.yaml
kubectl apply -f k8s/backend.yaml
kubectl apply -f k8s/audio-service.yaml
kubectl apply -f k8s/web.yaml
```

---

## Cloud Platform Specific

### AWS EKS

```bash
# Create EKS cluster
eksctl create cluster \
  --name maestro-cluster \
  --region us-east-1 \
  --nodegroup-name standard-workers \
  --node-type t3.medium \
  --nodes 3 \
  --nodes-min 2 \
  --nodes-max 5

# Deploy RDS PostgreSQL
# Use AWS RDS console or Terraform

# Deploy ElastiCache Redis
# Use AWS ElastiCache console or Terraform

# Deploy to EKS
kubectl apply -f k8s/
```

### Google Cloud GKE

```bash
# Create GKE cluster
gcloud container clusters create maestro-cluster \
  --num-nodes=3 \
  --machine-type=n1-standard-2 \
  --region=us-central1

# Deploy Cloud SQL PostgreSQL
# Use Google Cloud Console or Terraform

# Deploy Memorystore Redis
# Use Google Cloud Console or Terraform

# Deploy to GKE
kubectl apply -f k8s/
```

### Azure AKS

```bash
# Create AKS cluster
az aks create \
  --resource-group maestro-rg \
  --name maestro-cluster \
  --node-count 3 \
  --node-vm-size Standard_D2s_v3

# Deploy Azure Database for PostgreSQL
# Use Azure Portal or Terraform

# Deploy Azure Cache for Redis
# Use Azure Portal or Terraform

# Deploy to AKS
kubectl apply -f k8s/
```

---

## Performance Tuning

### Backend (Spring Boot)

```yaml
# application-prod.yml
spring:
  datasource:
    hikari:
      maximum-pool-size: 20
      minimum-idle: 5
      connection-timeout: 30000
  jpa:
    properties:
      hibernate:
        jdbc:
          batch_size: 20
        order_inserts: true
        order_updates: true

server:
  tomcat:
    threads:
      max: 200
      min-spare: 10
```

### Audio Service (Rust)

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = 'abort'
```

### Database

```sql
-- PostgreSQL tuning
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET effective_io_concurrency = 200;
```

---

## Monitoring & Observability

### Prometheus & Grafana

```yaml
# k8s/prometheus.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: maestro
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
    scrape_configs:
      - job_name: 'maestro-backend'
        kubernetes_sd_configs:
          - role: pod
        relabel_configs:
          - source_labels: [__meta_kubernetes_pod_label_app]
            regex: maestro-backend
            action: keep
```

### Application Metrics

Backend exposes metrics at `/actuator/prometheus`

Custom metrics:
- Request rate
- Response time (p50, p95, p99)
- Error rate
- Audio processing latency
- Active connections

### Logging

```yaml
# Use ELK Stack or Cloud-native logging
# Fluentd/Fluent Bit → Elasticsearch → Kibana
```

---

## Security Hardening

### Enable JWT Authentication

Replace dev auth filter in `SecurityConfig.kt`:

```kotlin
@Bean
fun jwtAuthenticationFilter(): JwtAuthenticationFilter {
    return JwtAuthenticationFilter(jwtService)
}

@Bean
fun securityFilterChain(http: HttpSecurity): SecurityFilterChain {
    http
        .csrf { it.disable() }
        .authorizeHttpRequests { auth ->
            auth
                .requestMatchers("/api/v1/auth/**").permitAll()
                .anyRequest().authenticated()
        }
        .addFilterBefore(jwtAuthenticationFilter(), UsernamePasswordAuthenticationFilter::class.java)

    return http.build()
}
```

### HTTPS/TLS

```yaml
# Use cert-manager for Let's Encrypt certificates
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: maestro-tls
  namespace: maestro
spec:
  secretName: maestro-tls-secret
  issuer:
    name: letsencrypt-prod
    kind: ClusterIssuer
  dnsNames:
  - maestro.example.com
```

### Network Policies

```yaml
# k8s/network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: maestro-network-policy
  namespace: maestro
spec:
  podSelector:
    matchLabels:
      app: maestro-backend
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: maestro-web
    ports:
    - protocol: TCP
      port: 8080
```

---

## Backup & Disaster Recovery

### PostgreSQL Backups

```bash
# Automated daily backups
0 2 * * * pg_dump -U postgres maestro | gzip > /backups/maestro-$(date +\%Y\%m\%d).sql.gz

# Retention: 30 days
find /backups -name "maestro-*.sql.gz" -mtime +30 -delete
```

### S3/MinIO Backups

```bash
# Sync to backup bucket
aws s3 sync s3://maestro-audio s3://maestro-audio-backup --storage-class GLACIER
```

### Disaster Recovery Plan

1. **RTO (Recovery Time Objective)**: 1 hour
2. **RPO (Recovery Point Objective)**: 24 hours
3. **Backup Strategy**:
   - Daily PostgreSQL dumps
   - Continuous S3 replication
   - Weekly full system snapshots

---

## Scaling Guidelines

### Horizontal Scaling

- **Backend**: Scale based on CPU (> 70%) and request rate
- **Audio Service**: Scale based on gRPC connection count
- **PostgreSQL**: Use read replicas for read-heavy workloads

### Vertical Scaling

- **Backend**: 2-4 CPU cores, 2-4 GB RAM per instance
- **Audio Service**: 1-2 CPU cores, 512 MB - 1 GB RAM per instance
- **PostgreSQL**: 4-8 CPU cores, 8-16 GB RAM

---

## Health Checks

### Backend

```bash
curl http://localhost:8080/actuator/health
```

### Audio Service

```bash
grpcurl -plaintext localhost:50051 list
```

### Database

```bash
psql -h localhost -U postgres -c "SELECT 1"
```

---

## Troubleshooting

### Common Issues

**Backend won't start**
- Check PostgreSQL connection
- Verify environment variables
- Check logs: `docker logs maestro-backend`

**Audio service connection refused**
- Verify port 50051 is open
- Check firewall rules
- Test with: `nc -zv localhost 50051`

**High latency**
- Check database query performance
- Monitor gRPC connection pool
- Verify network bandwidth

---

## Production Checklist

- [ ] JWT authentication enabled
- [ ] HTTPS/TLS configured
- [ ] Database backups automated
- [ ] Monitoring & alerting configured
- [ ] Resource limits set
- [ ] Network policies applied
- [ ] Secrets managed securely
- [ ] Logging centralized
- [ ] Auto-scaling configured
- [ ] Disaster recovery tested

---

*Last Updated: Final Session - Production Ready*
