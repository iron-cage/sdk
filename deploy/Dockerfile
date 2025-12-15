FROM google/cloud-sdk:548.0.0-slim
ENV TF_VERSION=1.7.4

ENV HOME=/app/.gcloud-home
WORKDIR /

# Install docker
RUN curl -fsSL https://get.docker.com | sh && which docker && docker --version

# Install terraform
RUN apt update --allow-releaseinfo-change \
    && apt install -y wget jq unzip \
    && mkdir -p /usr/lib/terraform/${TF_VERSION} \
    && cd /usr/lib/terraform/${TF_VERSION} \
    && wget https://releases.hashicorp.com/terraform/${TF_VERSION}/terraform_${TF_VERSION}_linux_amd64.zip \
    && unzip terraform_${TF_VERSION}_linux_amd64.zip \
    && chmod 755 /usr/lib/terraform/${TF_VERSION}/terraform \
    && ln -s /usr/lib/terraform/${TF_VERSION}/terraform /usr/bin/terraform

RUN mkdir -p $HOME/.config/gcloud \
    && chmod -R 777 $HOME

WORKDIR /app

VOLUME /var/run/docker.sock

# make -f Makefile.deploy deploy
CMD [ "make", "-f", "Makefile.deploy", "deploy_in_container" ]
