.PHONY: docker-build docker-up docker-down docker-logs docker-restart docker-clean docker-scale help

# Docker targets for easy deployment management

help:
	@echo "Telescrap Docker Commands"
	@echo ""
	@echo "Build targets:"
	@echo "  make docker-build          - Build all Docker images"
	@echo "  make docker-build-no-cache - Build all Docker images without cache"
	@echo ""
	@echo "Running targets:"
	@echo "  make docker-up             - Start all services in detached mode"
	@echo "  make docker-down           - Stop and remove all services"
	@echo "  make docker-restart        - Restart all services"
	@echo "  make docker-ps             - Show running services status"
	@echo ""
	@echo "Logging targets:"
	@echo "  make docker-logs           - Show all logs (follow)"
	@echo "  make docker-logs-scanner-1 - Show scanner-1 logs (follow)"
	@echo "  make docker-logs-scanner-2 - Show scanner-2 logs (follow)"
	@echo ""
	@echo "Service targets:"
	@echo "  make docker-restart-scanner-1 - Restart scanner-1 only"
	@echo "  make docker-restart-scanner-2 - Restart scanner-2 only"
	@echo "  make docker-stop-scanner      - Stop all scanner instances"
	@echo "  make docker-start-scanner     - Start all scanner instances"
	@echo ""
	@echo "Maintenance targets:"
	@echo "  make docker-clean             - Remove stopped containers and unused images"
	@echo "  make docker-prune             - Remove all unused Docker resources"
	@echo "  make docker-stats             - Show resource usage"
	@echo "  make docker-clean-volumes     - Clean up volumes and containers"
	@echo ""
	@echo "Configuration:"
	@echo "  make docker-env               - Copy .env.example to .env (if not exists)"
	@echo ""

docker-build:
	docker compose build

docker-build-no-cache:
	docker compose build --no-cache

docker-up:
	@if [ ! -f .env ]; then \
		echo "⚠️  .env file not found. Copying .env.example..."; \
		cp .env.example .env; \
		echo "✅ Created .env file. Please update it with your Telegram credentials."; \
	fi
	docker compose down --remove-orphans -v
	docker compose up -d
	@echo "✅ All services started!"
	@echo "📊 Scanner-1: http://localhost:3000"
	@echo "📊 Scanner-2: http://localhost:3001"
 --remove-orphans
docker-down:
	docker compose down
	@echo "✅ All services stopped"

docker-restart:
	docker compose restart
	@echo "✅ All services restarted"

docker-restart-admin:
	docker compose restart admin
	@echo "✅ Admin panel restarted"

docker-restart-scanner-1:
	docker compose restart scanner-1
	@echo "✅ Scanner-1 restarted"

docker-restart-scanner-2:
	docker compose restart scanner-2
	@echo "✅ Scanner-2 restarted"

docker-stop-scanner:
	docker compose stop scanner-1 scanner-2 scanner-3
	@echo "✅ All scanner instances stopped"

docker-start-scanner:
	docker compose start scanner-1 scanner-
	@echo "✅ All scanner instances stopped"

docker-start-scanner:
	docker compose start scanner-1 scanner-2

docker-logs:
	docker compose logs -f

docker-logs-admin:
	docker compose logs -f admin

docker-logs-scanner-1:
	docker compose logs -f scanner-1

docker-logs-scanner-2:
	docker compose logs -f scanner-2

docker-stats:
	docker stats

docker-clean:
	docker compose down
	docker container prune -f
	docker image prune -f
	@echo "✅ Docker cle --remove-orphans
	docker container prune -f
	docker image prune -f
	@echo "✅ Docker cleanup completed"

docker-prune:
	docker system prune -a -f
	@echo "✅ Full Docker prune completed"

docker-clean-volumes:
	docker compose down --remove-orphans -v
	docker volume prune -f
	@echo "✅ Volumes and containers clean
		cp .env.example .env; \
		echo "✅ Created .env file from .env.example"; \
	else \
		echo "ℹ️  .env file already exists"; \
	fi
scanner-1:
	docker compose exec scanner-1 /bin/bash

docker-shell-scanner-2:
	docker compose exec scanner-2
docker-shell-scanner-1:
	docker compose exec scanner-1 /bin/bash

# Push to registry (replace with your registry)
docker-push:
	@echo "Building for multiple platforms..."
	docker buildx build --platform linux/amd64,linux/arm64 -t your-registry/telescrap-sr:latest --push .
	@echo "✅ Images pushed to registry"

.DEFAULT_GOAL := help
