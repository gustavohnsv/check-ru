#!/bin/bash

# Project base path
BASE_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$BASE_PATH"

echo "Compilando o projeto..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Erro na compilação. Abortando."
    exit 1
fi

BINARY_PATH="$BASE_PATH/target/release/check-ru"

# Alias creation
SHELL_RC=""
if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
    SHELL_RC="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ] || [ -f "$HOME/.bashrc" ]; then
    SHELL_RC="$HOME/.bashrc"
fi

if [ -n "$SHELL_RC" ]; then
    if ! grep -q "alias ru=" "$SHELL_RC"; then
        echo "Adicionando alias 'ru' em $SHELL_RC..."
        echo "alias ru='$BINARY_PATH'" >> "$SHELL_RC"
        echo "Alias adicionado! Reinicie o terminal ou execute 'source $SHELL_RC'."
    else
        echo "Alias 'ru' já existe em $SHELL_RC."
    fi
fi

# Systemd user service for auto-fetch
SERVICE_DIR="$HOME/.config/systemd/user"
mkdir -p "$SERVICE_DIR"

echo "Criando serviço de atualização automática..."
cat <<EOF > "$SERVICE_DIR/ru-fetch.service"
[Unit]
Description=Atualiza o cardápio do RU (check-ru) no boot
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
ExecStart=$BINARY_PATH --fetch
RemainAfterExit=yes

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable ru-fetch.service

echo "Setup concluído com sucesso!"
