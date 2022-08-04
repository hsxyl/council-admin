#!/bin/bash
set -e

source ./variables.sh

cd ..
bash build.sh &&
cd scripts

if [ "$1" == "deploy" ]; then
  near deploy $ACCOUNT_ID ../res/$COUNCIL_ADMIN_CONTRACT_WASM new '{"dao_contract_id": "'$DAO_CONTRACT_ID'"}'
elif [ "$1" == "redeploy" ]; then
  near deploy $ACCOUNT_ID ../res/$COUNCIL_ADMIN_CONTRACT_WASM
elif [ "$1" == "clean" ]; then
  bash clear-state.sh && near deploy $ACCOUNT_ID ../res/$COUNCIL_ADMIN_CONTRACT_WASM new '{"dao_contract_id": "'$DAO_CONTRACT_ID'"}'
fi
