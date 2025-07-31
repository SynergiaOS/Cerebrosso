#!/bin/bash
# Cerberus Phoenix v2.0 - Convenient Aliases

# Deployment aliases
alias cerberus-deploy='./scripts/deploy-cerberus.sh'
alias cerberus-dev='./scripts/deploy-cerberus.sh -e development'
alias cerberus-prod='./scripts/deploy-cerberus.sh -e production -d -g'
alias cerberus-monitor='./scripts/deploy-cerberus.sh -m'

# Service management
alias cerberus-logs='docker-compose -f docker-compose.production.yml logs -f'
alias cerberus-status='docker-compose -f docker-compose.production.yml ps'
alias cerberus-stop='docker-compose -f docker-compose.production.yml down'
alias cerberus-restart='docker-compose -f docker-compose.production.yml restart'

# Quick access to services
alias cerberus-hft='curl -s http://localhost:8090/health | jq'
alias cerberus-cerebro='curl -s http://localhost:3000/health | jq'
alias cerberus-grafana='open http://localhost:3001'
alias cerberus-prometheus='open http://localhost:9090'

# Trading commands
alias cerberus-pnl='curl -s http://localhost:8090/api/dashboard/summary | jq'
alias cerberus-trades='curl -s http://localhost:8090/api/dashboard/trades | jq'

echo "🚀 Cerberus Phoenix v2.0 aliases loaded!"
echo "Use 'cerberus-deploy' to start deployment"
