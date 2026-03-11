# Check RU 🍽️

Uma ferramenta CLI/TUI moderna escrita em Rust para visualizar o cardápio dos Restaurantes Universitários (RU) da USP.

> [!IMPORTANT]
> **Compatibilidade:** Atualmente, este projeto foi desenvolvido e testado exclusivamente para **Linux**. O script de instalação e o serviço de atualização automática dependem do `bash` e do `systemd`.

## 🚀 Funcionalidades
- **Interface Gráfica no Terminal (TUI):** Visualização semanal completa com temas customizáveis.
- **Modo Diário CLI:** Exibição rápida do cardápio do dia diretamente no console (sem abrir a interface gráfica) via opção `daily_check`.
- **Temas Customizáveis:** Suporte a múltiplos temas JSON na pasta `themes/`.
- **Cache Local:** Funciona offline após a primeira busca do dia.
- **Atualização Automática:** Serviço configurável para buscar o cardápio no boot.

## 🛠️ Pré-requisitos
- [Rust](https://www.rust-lang.org/tools/install) (cargo, rustc)
- Conexão com a internet (para a primeira carga e atualizações)

## 📦 Instalação e Configuração
O projeto inclui um script de `setup.sh` para facilitar a vida:

```bash
./setup.sh
```

Este script irá:
1. Compilar o projeto em modo release.
2. Criar um alias `ru` no seu `.bashrc` ou `.zshrc`.
3. Configurar um serviço `systemd` de usuário para atualizar o cardápio automaticamente no login.

## ⌨️ Comandos e Uso
Após o setup, você pode usar:
- `ru`: Abre a interface TUI ou exibe o cardápio do dia (dependendo do `config.json`).
- `ru --fetch`: Força a atualização do cache local.

### Teclas na TUI:
- `Left/Right`: Navega entre os dias da semana.
- `Tab`: Alterna entre os temas instalados.
- `Esc`: Mostra/Esconde a ajuda.
- `q`: Sai do programa.

## ⚙️ Configuração (`config.json`)
Localizado na raiz do projeto:
```json
{
  "theme_name": "Dark",
  "daily_check": false,
  "unit_code": 13
}
```
- `theme_name`: Nome do tema (ex: "Dark", "Ocean", "Matrix").
- `daily_check`: Se `true`, exibe apenas o cardápio do dia e sai.
- `unit_code`: Código do restaurante (veja a lista abaixo).

## 🏢 Unidades Suportadas (`unit_code`)

| ID | Unidade | Compatibilidade |
|:---:|:--- |:---:|
| 1 | Central - São Paulo | Alta |
| 2 | Prefeitura - São Paulo | Alta |
| 3 | Física - São Paulo | Alta |
| 4 | Química - São Paulo | Alta |
| 5 | Bauru | Média |
| 6 | São Carlos | Média |
| 7 | Prefeitura - Piracicaba | Média |
| 8 | EACH - São Paulo | Alta |
| 9 | Ribeirão Preto | Média |
| 11 | Direito - São Paulo | Alta |
| 12 | Enfermagem - São Paulo | Alta |
| 13 | Poli - São Paulo | Alta |
| 14 | IPEN - São Paulo | Alta |
| 17 | Pirassununga | Média |
| 18 | Medicina - São Paulo | Alta |
| 19 | Saúde Pública - São Paulo | Alta |
| 20 | Lorena | Média |
| 23 | Oceanográfico - São Paulo | Alta |

**Legenda:**
- **Alta:** Cardápio segue o padrão rigoroso (Opções, Guarnição, PVT, etc.).
- **Média:** Cardápio pode ter nomes de pratos variados ou campos vazios ocasionalmente.
- **Baixa:** Unidade com formatação muito instável ou raramente atualizada.

## 📂 Árvore do Projeto
```text
.
├── src/
│   ├── main.rs      # Ponto de entrada e lógica da UI
│   ├── fetcher.rs   # Comunicação com a API da USP (DWR)
│   ├── menu.rs      # Estruturas de dados do cardápio
│   └── theme.rs     # Gerenciamento de temas e configuração
├── themes/          # Arquivos JSON de temas
├── config.json      # Configurações globais
├── menu.json        # Cache do cardápio baixado
├── Cargo.toml       # Dependências do Rust
└── setup.sh         # Script de instalação automatizada
```

---
Desenvolvido por @gustavohnsv
