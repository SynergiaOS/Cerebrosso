#!/bin/bash
# 🗄️ PostgreSQL Multiple Database Initialization Script
# Creates multiple databases for Cerberus Phoenix v2.0

set -e
set -u

function create_user_and_database() {
	local database=$1
	local user=$2
	local password=$3
	echo "  Creating user '$user' and database '$database'"
	psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname="$POSTGRES_DB" <<-EOSQL
	    DO \$\$
	    BEGIN
	        IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = '$user') THEN
	            CREATE USER $user WITH PASSWORD '$password';
	        END IF;
	    END
	    \$\$;

	    SELECT 'CREATE DATABASE $database' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '$database')\gexec
	    GRANT ALL PRIVILEGES ON DATABASE $database TO $user;
EOSQL
}

if [ -n "$POSTGRES_MULTIPLE_DATABASES" ]; then
	echo "Multiple database creation requested: $POSTGRES_MULTIPLE_DATABASES"
	for db in $(echo $POSTGRES_MULTIPLE_DATABASES | tr ',' ' '); do
		case $db in
			"cerberus_phoenix")
				create_user_and_database $db "cerberus" "phoenix"
				;;
			"kestra")
				echo "  Creating user 'kestra' for database '$db'"
				create_user_and_database $db "kestra" "kestra"
				;;
			*)
				echo "  Creating database '$db' with default user"
				createdb -U "$POSTGRES_USER" "$db"
				;;
		esac
	done
	echo "Multiple databases created"
fi
