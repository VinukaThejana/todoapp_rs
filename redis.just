set dotenv-load

connect:
  iredis --url $(echo $REDIS_URL)
