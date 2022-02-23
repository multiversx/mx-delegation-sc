#!/bin/bash

export PATH=$HOME/tools/wabt/bin:$PATH

./wat-build.sh latest_update
./wat-build.sh latest_full
