FROM rust:1.66

# Gitなどの必要なパッケージをインストール
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*

WORKDIR /app

CMD ["/bin/bash"]

