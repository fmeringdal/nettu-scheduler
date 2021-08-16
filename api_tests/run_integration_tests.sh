#!/bin/bash
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" || exit ; pwd -P )
cd "$parent_path" || exit

docker-compose -f ../server/tests/integrations/docker-compose.yml -p nettu-scheduler-test up &
# cargo test
../server/integrations/wait-for.sh localhost:5000

response=$(curl --write-out '%{http_code}' --silent --output /dev/null http://localhost:5000/)
while [ "$response" -ne 200 ]
do
response=$(curl --write-out '%{http_code}' --silent --output /dev/null http://localhost:5000/)
echo "Waiting for localhost:5000 to be ready"
sleep 1
done

npm run api-test
docker-compose -f ../server/tests/integrations/docker-compose.yml -p nettu-scheduler-test down
