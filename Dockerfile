FROM rust:1.66

# 必要なパッケージ (git, bash-completion) をインストール
RUN apt-get update && apt-get install -y \
    git \
    bash-completion \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# bashrc に bash-completion の読み込み設定を追加（必要に応じて）
RUN echo "if [ -f /etc/bash_completion ]; then . /etc/bash_completion; fi" >> /root/.bashrc

CMD ["/bin/bash"]

