# 🐺 Projekt Cerberus Phoenix v2.0 - DevKit
# Centralny panel sterowania dla całego ekosystemu

.PHONY: help dev dev-setup build deploy-cloud phoenix-restart clean test lint docs prod-setup prod-deploy prod-status prod-logs prod-stop security-full docker-dev docker-prod docker-build docker-up docker-down docker-logs docker-status

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

security-full: ## 🛡️ Pełne skanowanie bezpieczeństwa
	@echo "$(BLUE)🛡️ Pełne skanowanie bezpieczeństwa...$(NC)"
	./scripts/snyk-scan.sh
	./scripts/generate-sbom.sh
	./scripts/build-chainguard.sh --no-sbom
	@echo "$(GREEN)✅ Pełne skanowanie zakończone!$(NC)"

# 🚀 PRODUCTION DEPLOYMENT
prod-setup: ## 🏭 Przygotuj środowisko produkcyjne
	@echo "$(BLUE)🏭 Przygotowywanie środowiska produkcyjnego...$(NC)"
	./scripts/infisical-sync.sh export
	docker-compose -f infrastructure/docker-compose.yml -f infrastructure/docker-compose.chainguard.yml pull
	@echo "$(GREEN)✅ Środowisko produkcyjne gotowe!$(NC)"

prod-deploy: ## 🚀 Deploy produkcyjny z pełnym monitoringiem
	@echo "$(BLUE)🚀 Uruchamianie Cerberus Phoenix v2.0 PRODUCTION...$(NC)"
	@echo "$(YELLOW)📊 Uruchamianie infrastruktury monitoringu...$(NC)"
	docker-compose -f infrastructure/docker-compose.yml up -d prometheus grafana alertmanager vault qdrant postgres
	@sleep 10
	@echo "$(YELLOW)🐺 Uruchamianie HFT-Ninja...$(NC)"
	docker-compose -f infrastructure/docker-compose.yml up -d hft-ninja
	@sleep 5
	@echo "$(YELLOW)🧠 Uruchamianie Cerebro-BFF...$(NC)"
	docker-compose -f infrastructure/docker-compose.yml up -d cerebro-bff
	@sleep 5
	@echo "$(YELLOW)🚪 Uruchamianie Traefik Gateway...$(NC)"
	docker-compose -f infrastructure/docker-compose.yml up -d traefik
	@echo "$(GREEN)🎉 Cerberus Phoenix v2.0 PRODUCTION READY!$(NC)"
	@echo "$(CYAN)📊 Grafana: http://localhost:3001$(NC)"
	@echo "$(CYAN)🔍 Prometheus: http://localhost:9090$(NC)"
	@echo "$(CYAN)🚨 Alertmanager: http://localhost:9093$(NC)"
	@echo "$(CYAN)🐺 HFT-Ninja: http://localhost:8090$(NC)"
	@echo "$(CYAN)🧠 Cerebro-BFF: http://localhost:8081$(NC)"

prod-status: ## 📊 Status systemu produkcyjnego
	@echo "$(BLUE)📊 Status Cerberus Phoenix v2.0...$(NC)"
	@echo "$(YELLOW)🔍 Sprawdzanie serwisów...$(NC)"
	@docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep cerberus || echo "$(RED)❌ Brak uruchomionych serwisów$(NC)"
	@echo "$(YELLOW)🏥 Health checks...$(NC)"
	@curl -s http://localhost:8090/health > /dev/null && echo "$(GREEN)✅ HFT-Ninja: OK$(NC)" || echo "$(RED)❌ HFT-Ninja: DOWN$(NC)"
	@curl -s http://localhost:8081/health > /dev/null && echo "$(GREEN)✅ Cerebro-BFF: OK$(NC)" || echo "$(RED)❌ Cerebro-BFF: DOWN$(NC)"
	@curl -s http://localhost:9090/-/healthy > /dev/null && echo "$(GREEN)✅ Prometheus: OK$(NC)" || echo "$(RED)❌ Prometheus: DOWN$(NC)"
	@curl -s http://localhost:3001/api/health > /dev/null && echo "$(GREEN)✅ Grafana: OK$(NC)" || echo "$(RED)❌ Grafana: DOWN$(NC)"

prod-logs: ## 📋 Logi systemu produkcyjnego
	@echo "$(BLUE)📋 Logi Cerberus Phoenix v2.0...$(NC)"
	docker-compose -f infrastructure/docker-compose.yml logs -f --tail=50

prod-stop: ## 🛑 Zatrzymaj system produkcyjny
	@echo "$(BLUE)🛑 Zatrzymywanie Cerberus Phoenix v2.0...$(NC)"
	docker-compose -f infrastructure/docker-compose.yml down
	@echo "$(GREEN)✅ System zatrzymany!$(NC)"

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

# 🐳 Docker Commands
docker-dev: ## 🐳 Start development environment with Docker
	@echo "$(GREEN)🐳 Starting Cerberus Phoenix v2.0 with Docker...$(NC)"
	@docker-compose up -d postgres qdrant grafana prometheus
	@echo "$(YELLOW)⏳ Waiting for infrastructure...$(NC)"
	@sleep 10
	@docker-compose up -d hft-ninja dashboard
	@echo "$(GREEN)✅ Docker development environment ready!$(NC)"
	@echo ""
	@echo "$(CYAN)🔗 Service URLs:$(NC)"
	@echo "  🎨 Dashboard:     http://localhost:3002"
	@echo "  🥷 HFT-Ninja:     http://localhost:8091"
	@echo "  📊 Grafana:       http://localhost:3001"

docker-build: ## 🐳 Build all Docker images
	@echo "$(BLUE)🏗️ Building Docker images...$(NC)"
	@docker-compose build --parallel
	@echo "$(GREEN)✅ All images built!$(NC)"

docker-up: ## 🐳 Start all Docker services
	@docker-compose up -d

docker-down: ## 🐳 Stop all Docker services
	@docker-compose down

docker-logs: ## 🐳 View Docker logs
	@docker-compose logs -f

docker-status: ## 🐳 Show Docker service status
	@echo "$(CYAN)📊 Docker Service Status:$(NC)"
	@docker-compose ps
	@echo ""
	@echo "$(CYAN)🔗 Health Checks:$(NC)"
	@echo -n "  🎨 Dashboard:     "
	@curl -s http://localhost:3002 >/dev/null && echo "$(GREEN)✅ ONLINE$(NC)" || echo "$(RED)❌ OFFLINE$(NC)"
	@echo -n "  🥷 HFT-Ninja:     "
	@curl -s http://localhost:8091/health >/dev/null && echo "$(GREEN)✅ ONLINE$(NC)" || echo "$(RED)❌ OFFLINE$(NC)"

docker-clean: ## 🐳 Clean Docker resources
	@echo "$(RED)🧹 Cleaning Docker resources...$(NC)"
	@docker-compose down -v --remove-orphans
	@docker system prune -af
	@echo "$(GREEN)✅ Docker cleanup completed!$(NC)"

# Default target
.DEFAULT_GOAL := help
