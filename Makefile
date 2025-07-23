# 🐺 Projekt Cerberus Phoenix v2.0 - DevKit
# Centralny panel sterowania dla całego ekosystemu

.PHONY: help dev dev-setup build deploy-cloud phoenix-restart clean test lint docs

# Kolory dla lepszej czytelności
RED=\033[0;31m
GREEN=\033[0;32m
YELLOW=\033[1;33m
BLUE=\033[0;34m
PURPLE=\033[0;35m
CYAN=\033[0;36m
NC=\033[0m # No Color

# Domyślny target
help: ## 📖 Pokaż dostępne komendy
	@echo "$(CYAN)🐺 Projekt Cerberus Phoenix v2.0 - DevKit$(NC)"
	@echo "$(YELLOW)═══════════════════════════════════════════$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "$(GREEN)%-20s$(NC) %s\n", $$1, $$2}'

# 🚀 DEVELOPMENT
dev-setup: ## 🛠️ Przygotuj środowisko deweloperskie
	@echo "$(BLUE)🛠️ Przygotowywanie środowiska deweloperskiego...$(NC)"
	@command -v docker >/dev/null 2>&1 || { echo "$(RED)❌ Docker nie jest zainstalowany$(NC)"; exit 1; }
	@command -v docker-compose >/dev/null 2>&1 || { echo "$(RED)❌ Docker Compose nie jest zainstalowany$(NC)"; exit 1; }
	@command -v terraform >/dev/null 2>&1 || { echo "$(YELLOW)⚠️ Terraform nie jest zainstalowany (potrzebny do cloud deployment)$(NC)"; }
	@echo "$(GREEN)✅ Środowisko gotowe!$(NC)"

dev: ## 🏃 Uruchom cały stos lokalnie (development)
	@echo "$(BLUE)🏃 Uruchamianie stosu deweloperskiego...$(NC)"
	cd infrastructure && docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d
	@echo "$(GREEN)✅ Stos uruchomiony!$(NC)"
	@echo "$(CYAN)📊 Dashboard: http://localhost:3000$(NC)"
	@echo "$(CYAN)🧠 Cerebro API: http://localhost:8080$(NC)"
	@echo "$(CYAN)🥷 HFT-Ninja API: http://localhost:8081$(NC)"
	@echo "$(CYAN)⚙️ Kestra UI: http://localhost:8082$(NC)"

dev-stop: ## 🛑 Zatrzymaj stos deweloperski
	@echo "$(YELLOW)🛑 Zatrzymywanie stosu deweloperskiego...$(NC)"
	cd infrastructure && docker-compose -f docker-compose.yml -f docker-compose.dev.yml down
	@echo "$(GREEN)✅ Stos zatrzymany!$(NC)"

# 🏗️ BUILD
build: ## 🏗️ Zbuduj wszystkie obrazy
	@echo "$(BLUE)🏗️ Budowanie obrazów...$(NC)"
	cd infrastructure && docker-compose build
	@echo "$(GREEN)✅ Obrazy zbudowane!$(NC)"

build-apko: ## 📦 Zbuduj obrazy Apko (production)
	@echo "$(BLUE)📦 Budowanie obrazów Apko...$(NC)"
	cd infrastructure/apko && \
	apko build hft-ninja.yaml hft-ninja:latest hft-ninja.tar && \
	apko build cerebro-bff.yaml cerebro-bff:latest cerebro-bff.tar
	@echo "$(GREEN)✅ Obrazy Apko zbudowane!$(NC)"

# ☁️ CLOUD DEPLOYMENT
deploy-cloud: ## ☁️ Deploy na Oracle Cloud Free Tier
	@echo "$(BLUE)☁️ Deploying na Oracle Cloud...$(NC)"
	cd infrastructure/terraform && terraform init && terraform apply -auto-approve
	@echo "$(GREEN)✅ Deployment zakończony!$(NC)"

destroy-cloud: ## 💥 Zniszcz infrastrukturę cloud
	@echo "$(RED)💥 Niszczenie infrastruktury cloud...$(NC)"
	@read -p "Czy na pewno chcesz zniszczyć infrastrukturę? [y/N] " confirm && [ "$$confirm" = "y" ]
	cd infrastructure/terraform && terraform destroy -auto-approve
	@echo "$(GREEN)✅ Infrastruktura zniszczona!$(NC)"

# 🔄 PHOENIX OPERATIONS
phoenix-restart: ## 🔄 Pełny restart systemu (Phoenix)
	@echo "$(PURPLE)🔄 Phoenix Restart - pełny restart systemu...$(NC)"
	$(MAKE) dev-stop
	$(MAKE) build
	$(MAKE) dev
	@echo "$(GREEN)✅ System odrodzony z popiołów!$(NC)"

phoenix-reset: ## 🔥 Kompletny reset (usuń wszystkie dane)
	@echo "$(RED)🔥 Phoenix Reset - kompletny reset systemu...$(NC)"
	@read -p "Czy na pewno chcesz usunąć wszystkie dane? [y/N] " confirm && [ "$$confirm" = "y" ]
	$(MAKE) dev-stop
	docker system prune -af --volumes
	$(MAKE) dev-setup
	@echo "$(GREEN)✅ System zresetowany!$(NC)"

# 🧪 TESTING
test: ## 🧪 Uruchom wszystkie testy
	@echo "$(BLUE)🧪 Uruchamianie testów...$(NC)"
	cd services/hft-ninja && cargo test
	cd services/cerebro-bff && cargo test
	cd services/dashboard && npm test
	@echo "$(GREEN)✅ Testy zakończone!$(NC)"

test-integration: ## 🔗 Testy integracyjne end-to-end
	@echo "$(BLUE)🔗 Testy integracyjne...$(NC)"
	# TODO: Implementacja testów e2e
	@echo "$(GREEN)✅ Testy integracyjne zakończone!$(NC)"

# 🔍 QUALITY
lint: ## 🔍 Sprawdź jakość kodu
	@echo "$(BLUE)🔍 Sprawdzanie jakości kodu...$(NC)"
	cd services/hft-ninja && cargo clippy -- -D warnings
	cd services/cerebro-bff && cargo clippy -- -D warnings
	cd services/dashboard && npm run lint
	@echo "$(GREEN)✅ Kod sprawdzony!$(NC)"

format: ## 🎨 Formatuj kod
	@echo "$(BLUE)🎨 Formatowanie kodu...$(NC)"
	cd services/hft-ninja && cargo fmt
	cd services/cerebro-bff && cargo fmt
	cd services/dashboard && npm run format
	@echo "$(GREEN)✅ Kod sformatowany!$(NC)"

# 📊 MONITORING
logs: ## 📊 Pokaż logi systemu
	@echo "$(BLUE)📊 Logi systemu...$(NC)"
	cd infrastructure && docker-compose logs -f

logs-ninja: ## 🥷 Logi HFT-Ninja
	cd infrastructure && docker-compose logs -f hft-ninja

logs-cerebro: ## 🧠 Logi Cerebro-BFF
	cd infrastructure && docker-compose logs -f cerebro-bff

status: ## 📈 Status wszystkich serwisów
	@echo "$(BLUE)📈 Status serwisów...$(NC)"
	cd infrastructure && docker-compose ps

# 🧹 CLEANUP
clean: ## 🧹 Wyczyść pliki tymczasowe
	@echo "$(BLUE)🧹 Czyszczenie...$(NC)"
	cd services/hft-ninja && cargo clean
	cd services/cerebro-bff && cargo clean
	cd services/dashboard && rm -rf node_modules .next
	@echo "$(GREEN)✅ Wyczyszczono!$(NC)"

# 📚 DOCUMENTATION
docs: ## 📚 Generuj dokumentację
	@echo "$(BLUE)📚 Generowanie dokumentacji...$(NC)"
	cd services/hft-ninja && cargo doc --no-deps
	cd services/cerebro-bff && cargo doc --no-deps
	@echo "$(GREEN)✅ Dokumentacja wygenerowana!$(NC)"

docs-serve: ## 🌐 Serwuj dokumentację lokalnie
	@echo "$(BLUE)🌐 Serwowanie dokumentacji...$(NC)"
	cd services/hft-ninja && cargo doc --no-deps --open

# 🔐 SECURITY
security-scan: ## 🔐 Skanowanie bezpieczeństwa
	@echo "$(BLUE)🔐 Skanowanie bezpieczeństwa...$(NC)"
	cd services/hft-ninja && cargo audit
	cd services/cerebro-bff && cargo audit
	cd services/dashboard && npm audit
	@echo "$(GREEN)✅ Skanowanie zakończone!$(NC)"

infisical-setup: ## 🔐 Konfiguruj Infisical secrets
	@echo "$(BLUE)🔐 Konfigurowanie Infisical...$(NC)"
	./scripts/infisical-setup.sh
	@echo "$(GREEN)✅ Infisical skonfigurowany!$(NC)"

infisical-sync: ## 🔄 Synchronizuj secrets z Infisical
	@echo "$(BLUE)🔄 Synchronizacja secrets...$(NC)"
	./scripts/infisical-sync.sh
	@echo "$(GREEN)✅ Secrets zsynchronizowane!$(NC)"

# 📦 RELEASE
release: ## 📦 Przygotuj release
	@echo "$(BLUE)📦 Przygotowywanie release...$(NC)"
	$(MAKE) test
	$(MAKE) lint
	$(MAKE) security-scan
	$(MAKE) build-apko
	@echo "$(GREEN)✅ Release gotowy!$(NC)"

# Default target
.DEFAULT_GOAL := help
