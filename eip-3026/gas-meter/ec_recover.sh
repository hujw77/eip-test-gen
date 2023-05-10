#!/bin/env/bash

cat ec_recover.json | jq '["id", "time", "unit"],([.id, .typical.estimate, .typical.unit]) | @csv' > ec_recover.csv

