#!/bin/bash

while true; do
  inotifywait -r -e modify,create,delete,move src && reset && cargo build
done
