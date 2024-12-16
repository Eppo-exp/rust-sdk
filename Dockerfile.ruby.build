FROM ruby:3.3

ARG WORKDIR

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    libssl-dev \
    zlib1g-dev \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

RUN apt-get install ca-certificates curl
RUN install -m 0755 -d /etc/apt/keyrings
RUN curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
RUN chmod a+r /etc/apt/keyrings/docker.asc

RUN echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  tee /etc/apt/sources.list.d/docker.list > /dev/null
RUN apt-get update
RUN apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

WORKDIR $WORKDIR

RUN gem install rb_sys

COPY . .

CMD ["/bin/bash", "build.sh"]
