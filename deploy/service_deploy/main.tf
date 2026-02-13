terraform {
  # No backend "gcs" {} — tfstate is intentionally not persisted.
  # This module is used as an orchestration script, not as traditional
  # Terraform infrastructure. Resources are recreated on every run
  # (triggers_replace = timestamp()), so previous state is irrelevant.
  # See readme.md "Terraform State" section for details.
  required_version = ">= 1.0"
}

resource "terraform_data" "redeploy_sh" {
  triggers_replace = {
    always = timestamp()
  }

  # Connect to host server
  connection {
    type        = "ssh"
    user        = "root"
    private_key = data.local_sensitive_file.ssh_private_key.content
    host        = var.HOST_SERVER_IP
    timeout     = "1m"
  }

  # Cloud-init wait
  provisioner "remote-exec" {
    inline = [
      # Wait cloud-init to finish
      "bash -lc 'command -v cloud-init >/dev/null 2>&1 && timeout 7m cloud-init status --wait || true'",
      # Check the correct hostname machine
      "test \"$(hostname)\" = \"${var.HOST_SERVER_NAME}\"",
      # Verify k3s is installed and kubectl is functional
      "kubectl version --short 2>/dev/null || kubectl version"
    ]
  }

  # ===============================================================================================
  # Files copy

  # Project folder create
  provisioner "remote-exec" {
    inline = [ 
      "mkdir -p /opt/services/${var.PROJECT_NAME}_${var.DEPLOYMENT_MODE}",
      "mkdir -p /opt/services/${var.PROJECT_NAME}_${var.DEPLOYMENT_MODE}/secrets",
      "mkdir -p /opt/services/${var.PROJECT_NAME}_${var.DEPLOYMENT_MODE}/k8s",
    ]
  }

  # k8s deployment
  provisioner "file" {
    content = templatefile("${path.module}/../k8s/deployment.yaml", {
      tag     = "${var.TAG}"
      version = "${var.VERSION}"
      port_back = "${var.SERVER_PORT}"
    })
    destination = "/opt/services/${var.PROJECT_NAME}_${var.DEPLOYMENT_MODE}/k8s/deployment.yaml"
  } 

  # k8s ingress
  provisioner "file" {
    content = templatefile("${path.module}/../k8s/ingress.yaml", {
      project_domain = "${var.PROJECT_DOMAIN}"
      port_back = "${var.SERVER_PORT}"
    })

    destination = "/opt/services/${var.PROJECT_NAME}_${var.DEPLOYMENT_MODE}/k8s/ingress.yaml"
  }

  # k8s kustomization
  provisioner "file" {
    source      = "${path.module}/../k8s/kustomization.yaml"
    destination = "/opt/services/${var.PROJECT_NAME}_${var.DEPLOYMENT_MODE}/k8s/kustomization.yaml"
  }

  # k8s service
  provisioner "file" {
    content = templatefile("${path.module}/../k8s/service.yaml", {
       port_back = "${var.SERVER_PORT}"
    })
    destination = "/opt/services/${var.PROJECT_NAME}_${var.DEPLOYMENT_MODE}/k8s/service.yaml"
  }

  # cluster-issuer.yaml
  provisioner "file" {
    content = templatefile("${path.module}/../k8s/cluster-issuer.yaml", {
      project_cert_email = "${var.PROJECT_CERT_EMAIL}"
    })

    destination = "/opt/services/${var.PROJECT_NAME}_${var.DEPLOYMENT_MODE}/k8s/cluster-issuer.yaml"
  }

  # redeploy.sh
  provisioner "file" {  
    source      = "${path.module}/../redeploy.sh"
    destination = "/opt/services/${var.PROJECT_NAME}_${var.DEPLOYMENT_MODE}/redeploy.sh"
  }

  # env generate
  provisioner "file" {
    content = join("\n", concat(
      [
        for k, v in var.PROJECT_MAP_VARIABLES :
        "${k}=${v}"
      ]
    ))

    destination = "/opt/services/${var.PROJECT_NAME}_${var.DEPLOYMENT_MODE}/secrets/env"
  }

  # redeploy.sh start 
  provisioner "file" {
    source      = var.GOOGLE_APPLICATION_CREDENTIALS
    destination = "/root/.sa.json"
  }

  provisioner "remote-exec" {
    inline = [
      "set -e",
      "chmod 400 /root/.sa.json",
      "bash /opt/services/${var.PROJECT_NAME}_${var.DEPLOYMENT_MODE}/redeploy.sh < /root/.sa.json",
      "shred -vfz -n 3 /root/.sa.json"
    ]
  }
}
