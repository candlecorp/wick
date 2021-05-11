export WASH_RPC_HOST=nats.lan

wash ctl push R_ITEM_ADD '{"user_id":"jsoverson", "content": "# HELLO WORLD\n\nThis is my blog post"}' --output json
#wash ctl push R_ITEM_ADD "{\"content\":\"# HELLO WORLD\n\nThis is my blog post\"}" --tx_id c5ec32f0-d2db-417f-bcea-342171c70669

wash ctl take R_LOG_1 output --tx_id c5ec32f0-d2db-417f-bcea-342171c70669

wash ctl push R_ITEM_LIST '{"user_id": "jsoverson"}'

wash ctl push R_ITEM_GET '{"user_id": "jsoverson", "content_id":"112fae8b-fc1b-4460-9a58-843973bf65ae"}'

wash ctl take R_LOG_3 output --tx_id

wash ctl take R_RENDER_MARKDOWN output --tx_id a24a2e7c-1d48-457a-b38e-981d5d01d39a

wash ctl take R_MD5 output --tx_id
