#!/bin/sh

# docker/prod/entrypoint.sh
# If #!/bin/sh is not placed at the very top, it will cause an error!

echo "start supercronic"
exec /usr/local/bin/supercronic /app/crontab

