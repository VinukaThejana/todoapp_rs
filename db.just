set dotenv-load

migrate:
  @echo "Migrating database ... "
  usql $(echo $DATABASE_URL) -f ./schema/db.sql

connect:
  @echo "Connecting to database ... "
  usql $(echo $DATABASE_URL)

generate:
  @echo "Generating entities ... "
  sea-orm-cli generate entity -u $(echo $DATABASE_URL) -o src/entity
