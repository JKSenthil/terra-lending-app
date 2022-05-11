import pickle
import argparse

from terra_sdk.client.lcd import LCDClient
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.client.localterra import LocalTerra
from helpers import execute_contract, construct_binary_msg

def read_config(filename):
    config = pickle.load(open(filename, "rb"))
    return config["generic_token_code_id"], config["generic_token_addr"], config["lending_token_code_id"], config["lending_token_addr"], config["lending_protocol_code_id"], config["lending_protocol_addr"]

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("network", help="localterra|bombay",type=str)
    args = parser.parse_args()

    if args.network == "localterra":
        lt = LocalTerra()
        deployer = lt.wallets["test1"]
        generic_token_code_id, generic_token_addr, lending_token_code_id, lending_token_addr, lending_protocol_code_id, lending_protocol_addr \
            = read_config("config_localterra.p")
    elif args.network == "bombay":
        lt = LCDClient(url="https://lcd.terra.dev", chain_id="bombay-12")
        mnemonic = open("mnemonic.txt", "r").read()
        mk = MnemonicKey(mnemonic=mnemonic)
        deployer = lt.wallet(mk)
        generic_token_code_id, generic_token_addr, lending_token_code_id, lending_token_addr, lending_protocol_code_id, lending_protocol_addr \
            = read_config("config_bombay.p")
    else:
        print("valid network required")
        exit(1)
    
    msg = {"Deposit": {}}
    results = execute_contract(
        lt,
        deployer,
        generic_token_addr,
        {
            "send": {
                "contract": lending_protocol_addr,
                "amount": f"{2000}",
                "msg": construct_binary_msg(msg)
            }
        }
    )

    balance = lt.wasm.contract_query(
            generic_token_addr, {
                "balance": {"address": lending_protocol_addr}
            }
    )['balance']
    assert(balance == '2000')

    results = execute_contract(
        lt,
        deployer,
        lending_protocol_addr,
        {
            "withdraw": {
                "amount": "1000"
            }
        }
    )
    
    balance = lt.wasm.contract_query(
        generic_token_addr, {
            "balance": {"address": lending_protocol_addr}
        }
    )['balance']
    assert(balance == '1000')

    results = execute_contract(
        lt,
        deployer,
        lending_protocol_addr,
        {
            "borrow": {
                "amount": "500"
            }
        }
    )

    balance = lt.wasm.contract_query(
        lending_token_addr, {
            "balance": {"address": deployer.key.acc_address}
        }
    )['balance']
    assert(balance == '600')