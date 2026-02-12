# Kubernetes Manifests

This directory contains Kubernetes manifests for deploying the application using Kustomize.

## Features

- **Zero-Downtime Deployments** - Rolling updates with 2 replicas ensure continuous availability
- **Automatic Rollback** - Failed deployments automatically revert to the previous version
- **Health Monitoring** - Readiness and liveness probes ensure only healthy pods serve traffic
- **Ingress Routing** - Domain-based routing via Traefik ingress controller

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `cluster-issuer.yaml` | Cluster-wide Let's Encrypt ACME issuer for automatic TLS certificates via cert-manager |
| `deployment.yaml` | Define pod deployment configuration with replicas and health checks |
| `service.yaml` | Define ClusterIP service for internal routing |
| `ingress.yaml` | Configure external traffic routing via Traefik |
| `kustomization.yaml` | Manage Kubernetes manifest composition |

## Deployment Configuration

### Replicas & Rolling Updates

```yaml
spec:
  replicas: 2  # Ensures zero-downtime during updates
```

During deployment:
1. New pods are created with the updated image
2. Readiness probe confirms new pods are healthy
3. Traffic shifts to new pods
4. Old pods are terminated

### Health Probes

**Readiness Probe** - Determines when pod can receive traffic:
```yaml
readinessProbe:
  httpGet:
    path: /
    port: 80
  initialDelaySeconds: 3
  periodSeconds: 5
```

**Liveness Probe** - Detects stuck/unhealthy pods for restart:
```yaml
livenessProbe:
  httpGet:
    path: /
    port: 80
  initialDelaySeconds: 10
  periodSeconds: 10
```

### Ingress Routing

Routes external traffic based on domain:
```yaml
spec:
  rules:
    - host: ${project_domain}
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: app
                port:
                  number: 80
```

## Usage

These manifests are applied via the `redeploy.sh` script:

```bash
# Apply all manifests in namespace
kubectl apply -k ./k8s -n ${NAMESPACE}

# Check rollout status
kubectl rollout status deployment/app -n ${NAMESPACE}

# Manual rollback if needed
kubectl rollout undo deployment/app -n ${NAMESPACE}
```

## Customization

Kustomize allows environment-specific customization without modifying base manifests:

```yaml
# kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - deployment.yaml
  - service.yaml
  - ingress.yaml

images:
  - name: app
    newName: ${registry}/${image}
    newTag: ${version}
```

## Troubleshooting

### Useful Commands

| Action | Command |
|--------|---------|
| View pod status | `kubectl get pods -n ${NAMESPACE}` |
| View pod logs | `kubectl logs -n ${NAMESPACE} <pod-name>` |
| Describe deployment | `kubectl describe deployment/app -n ${NAMESPACE}` |
| View events | `kubectl get events -n ${NAMESPACE} --sort-by=.lastTimestamp` |
| Check ingress | `kubectl get ingress -n ${NAMESPACE}` |
| Manual rollback | `kubectl rollout undo deployment/app -n ${NAMESPACE}` |

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| `ImagePullBackOff` | Registry auth failed or image tag not found | Check `gar-auth` secret exists: `kubectl get secret gar-auth -n ${NAMESPACE}`. Verify image tag in deployment matches pushed tag |
| `CrashLoopBackOff` | Application crashes on startup | Check logs: `kubectl logs -n ${NAMESPACE} <pod-name> --previous` |
| Rollout timeout (120s) | Pods not passing readiness probe | Check probe endpoint responds: `kubectl exec <pod> -n ${NAMESPACE} -- curl -f localhost:80/` |
| Auto rollback triggered | New version failed health checks | `redeploy.sh` runs `kubectl rollout undo` automatically. Check events for failure reason |
| Ingress returns 404 | Ingress rule mismatch or service not ready | Verify host matches `PROJECT_DOMAIN` and service has ready endpoints: `kubectl get endpoints app -n ${NAMESPACE}` |
| Pods stuck in `Pending` | Insufficient node resources | Check node capacity: `kubectl describe node` and pod requests in `deployment.yaml` |
