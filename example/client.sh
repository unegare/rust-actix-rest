#!/bin/bash

SCRIPTPATH="$( cd "$(dirname "$0")" ; pwd -P )"

curl -i -X POST -F "files[]=@${SCRIPTPATH}/i1.jpg" -F "files[]=@${SCRIPTPATH}/i2.jpg" localhost:8080/upload

echo ""

{ echo -n "{\"binarr\":[{\"filename\":\"qq\", \"data\":\""; base64 < ${SCRIPTPATH}/img.png; echo -n "\"}], \"urls\":[\"http://unegare.info/i.jpg\"]}"; } | curl -i -X POST -H "Content-Type: application/json" -d @- localhost:8080/upload

echo ""

curl -i -X POST -F "url=\"http://unegare.info/i.jpg\"" localhost:8080/upload

echo ""

curl -i -X POST -F "url[]=\"http://unegare.info/i.jpg\"" -F "url[]=\"http://unegare.info/p2.jpg\"" localhost:8080/upload

echo ""
