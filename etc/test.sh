#!/bin/bash

set -e

CMD="cargo run -q --"

TX_ID=$($CMD ctl push R_ITEM_ADD '{"user_id":"jsoverson", "content": "# HELLO WORLD\n\nThis is my blog post"}' --output json | jq -r '.[0].tx_id')

echo "Transaction ID of ITEM_ADD: $TX_ID"

BLOG_ID=$($CMD ctl take R_LOG_1 output --tx_id $TX_ID --output json --encoder messagepack | jq -r ".response")
echo "Blog id: $BLOG_ID"

TX_ID=$($CMD ctl push R_ITEM_GET "{\"user_id\": \"jsoverson\", \"content_id\":\"$BLOG_ID\"}" --output json | jq -r '.[0].tx_id')
echo "Transaction ID of ITEM_GET: $TX_ID"

MARKDOWN=$($CMD ctl take R_RENDER_MARKDOWN output --tx_id $TX_ID --output json --encoder messagepack | jq -r ".response")

echo "Rendered markdown"

expected="<h1>HELLO WORLD</h1>
<p>This is my blog post</p>"

echo $MARKDOWN

if [[ "$MARKDOWN" == "$expected" ]]; then
  echo "OK"
else
  echo "NOT OK"
  echo "Expected:"
  echo $expected
  echo "Actual:"
  echo $MARKDOWN
  exit 1
fi

MD5=$($CMD ctl take R_MD5 output --tx_id $TX_ID --output json --encoder messagepack | jq -r ".response")
echo "MD5: $MD5"

expected="92618cf8870f06e674e966bfdc2bf974"

if [[ "$MD5" == "$expected" ]]; then
  echo "OK"
else
  echo "NOT OK"
  echo "Expected:"
  echo $expected
  echo "Actual:"
  echo $MD5
  exit 1
fi
