set dotenv-load

migrate:
  @echo "Migrating database ... "
  usql $(echo $DATABASE_URL) -f ./schema/db.sql

connect:
  @echo "Connecting to database ... "
  usql $(echo $DATABASE_URL)
