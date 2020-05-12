#! /bin/bash
RPC_PORT_1="9966"
RPC_PORT_2="9967"

PHRASE_1=""
SR_KEY_1=""
ED_KEY_1=""

PHRASE_2=""
SR_KEY_2=""
ED_KEY_2=""

KEY="gran"
GRAN_JSON_1='{
        "jsonrpc":"2.0",
                "id":1,
                "method":"author_insertKey",
                "params": [
                        "'$KEY'",
                "'$PHRASE_1'",
                "'$ED_KEY_1'"
                ]
}
'
GRAN_JSON_2='{
        "jsonrpc":"2.0",
                "id":1,
                "method":"author_insertKey",
                "params": [
                        "'$KEY'",
                "'$PHRASE_2'",
                "'$ED_KEY_2'"
                ]
}
'

# insert grandpa key
curl http://localhost:$RPC_PORT_1 -H "Content-Type:application/json;charset=utf-8" -d "$GRAN_JSON_1"
curl http://localhost:$RPC_PORT_2 -H "Content-Type:application/json;charset=utf-8" -d "$GRAN_JSON_2"

KEY="aura"
AURA_JSON_1='{
        "jsonrpc":"2.0",
                "id":1,
                "method":"author_insertKey",
                "params": [
                        "'$KEY'",
                "'$PHRASE_1'",
                "'$SR_KEY_1'"
                ]
}
'
AURA_JSON_2='{
        "jsonrpc":"2.0",
                "id":1,
                "method":"author_insertKey",
                "params": [
                        "'$KEY'",
                "'$PHRASE_2'",
                "'$SR_KEY_2'"
                ]
}
'

# insert aura key
curl http://localhost:$RPC_PORT_1 -H "Content-Type:application/json;charset=utf-8" -d "$AURA_JSON_1"
curl http://localhost:$RPC_PORT_2 -H "Content-Type:application/json;charset=utf-8" -d "$AURA_JSON_2"


