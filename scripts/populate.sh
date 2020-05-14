#! /bin/bash
RPC_PORT_1="9966"
RPC_PORT_2="9967"

PHRASE_1="weird custom age profit front expose bicycle unfold cost clean make save"
SR_KEY_1="0x1e1e190eb5a4295b981e43c9f6aa6e3f2703be50e2dffcd570caf5eaecfbb81a"
ED_KEY_1="0x14d90002fa5ff0e7a1817d021ee152761b2a9bae375c4e39ee572cc1bbfb70be"

PHRASE_2="fan energy discover library trigger prevent north easily toe earn subway price"
SR_KEY_2="0x189213d79bd6b04340fb3a38ea62b2e57e69c0706d90da53c5bbcf738d668868"
ED_KEY_2="0x9120d1a0ba0179eecbf7828e1dfb5337171db0514ba4298afa1aebc79ad9e31b"

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


