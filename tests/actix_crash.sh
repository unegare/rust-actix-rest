#!/bin/bash

seq 1024 | xargs -i curl -i -X POST -F url="http://unegare.info/i.jpg" localhost:8080/upload &
seq 1024 | xargs -i curl -i -X POST -F url="http://unegare.info/i.jpg" localhost:8080/upload &
seq 1024 | xargs -i curl -i -X POST -F url="http://unegare.info/i.jpg" localhost:8080/upload &
seq 1024 | xargs -i curl -i -X POST -F url="http://unegare.info/i.jpg" localhost:8080/upload &
