#!/bin/bash

SCRIPTPATH="$( cd "$(dirname "$0")" ; pwd -P )"

curl -i -X POST -F "files[]=@${SCRIPTPATH}/i1.jpg" -F "files[]=@${SCRIPTPATH}/i2.jpg" localhost:8080/upload

echo ""

{ echo -n "{\"arr\":[{\"filename\":\"qq\", \"data\":\""; base64 < ${SCRIPTPATH}/img.png; echo -n "\"}]}"; } | curl -i -X POST -H "Content-Type: application/json" -d @- localhost:8080/upload
