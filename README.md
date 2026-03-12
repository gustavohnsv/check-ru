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

| ID | Unidade | Campus | Compatibilidade |
|:---:|:--- |:---:|:---:|
| 5 | Restaurante das Químicas | Butantã | Alta |
| 6 | Restaurante Central | Butantã | Alta |
| 7 | Restaurante PUSP-C | Butantã | Alta |
| 8 | Restaurante da Física | Butantã | Alta |
| 9 | Escola de Enfermagem | Quadrilátero Saúde | Alta |
| 11 | Faculdade de Direito | Largo São Francisco | Alta |
| 13 | EACH | USP Leste | Alta |
| 14 | Faculdade de Saúde Pública | Quadrilátero Saúde | Alta |
| 15 | Restaurante de Piracicaba (ESALQ) | Piracicaba | Média |
| 16 | Restaurante de Pirassununga | Pirassununga | Média |
| 17 | Restaurante de Lorena (EEL) | Lorena | Média |
| 18 | Faculdade de Medicina | Pinheiros | Alta |
| 19 | RU de Ribeirão Preto | Ribeirão Preto | Média |
| 20 | RU de Bauru | Bauru | Média |
| 21 | Restaurante de São Carlos (Área 1) | São Carlos | Média |
| 22 | Restaurante de São Carlos (Área 2) | São Carlos | Média |

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
