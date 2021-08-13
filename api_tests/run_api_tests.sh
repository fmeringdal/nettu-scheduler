#!/bin/bash
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")"  || exit ; pwd -P )
cd "$parent_path" || exit

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
cd ../server || exit
export CREATE_ACCOUNT_SECRET_CODE="FW4KbTC2loN1Ckr8KkIcwE3Av"
cargo run inmemory &
SERVER_PID=$!
cd ../api_tests || exit

response=$(curl --write-out '%{http_code}' --silent --output /dev/null http://localhost:5000/)
while [ "$response" -ne 200 ]
do
response=$(curl --write-out '%{http_code}' --silent --output /dev/null http://localhost:5000/)
echo "Waiting for localhost:5000 to be ready"
sleep 1
done

npm run api-test
kill $SERVER_PID
