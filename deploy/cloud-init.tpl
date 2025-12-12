#cloud-config

users:
  - name: root  
    shell: /bin/bash
    sudo: ["ALL=(ALL) NOPASSWD: ALL"]
    lock_passwd: true
    ssh_authorized_keys:
      - ${MASTER_SSH_KEY}

packages:
  - htop
  - sqlite3
  - libsqlite3-dev

write_files:
  - path: /root/service_account.json.b64
    permissions: '0600'
    owner: root:root
    content: |
      ${service_account_creds_b64}

  - path: /root/init.sh
    permissions: '0700'
    owner: root:root
    content: |
      #!/bin/bash
      set -e

      # decode service account creds
      base64 -d /root/service_account.json.b64 > /root/service_account.json
      chmod 600 /root/service_account.json

      # env for redeploy
      {
        echo "DOCKER_IMAGE=${tag}"
        echo "DOCKER_IMAGE_NAME=${image_name}"
        echo "JWT_SECRET=${jwt_secret}"
        echo "IRON_SECRETS_MASTER_KEY=${iron_secrets_master_key}"
        echo "DATABASE_URL=${database_url}"
      } >> /etc/environment

      apt update
      apt install -y apt-transport-https ca-certificates curl software-properties-common gnupg

      # docker repo
      mkdir -p /etc/apt/keyrings
      curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
      chmod a+r /etc/apt/keyrings/docker.gpg
      echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo $VERSION_CODENAME) stable" > /etc/apt/sources.list.d/docker.list
      apt update
      apt install -y docker-ce

      # gcloud
      curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg
      echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" | tee -a /etc/apt/sources.list.d/google-cloud-sdk.list
      apt-get update
      apt-get install -y google-cloud-cli

      gcloud auth activate-service-account --key-file=/root/service_account.json
      gcloud auth configure-docker ${location}-docker.pkg.dev --quiet

      echo "Instance init completed."


runcmd:
  - /root/init.sh > /var/log/deploy-init.log 2>&1
