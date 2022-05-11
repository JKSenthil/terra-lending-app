import pickle
import argparse

from terra_sdk.client.lcd import LCDClient
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.client.localterra import LocalTerra
from helpers import store_contract, instantiate_contract, execute_contract

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("network", help="localterra|bombay",type=str)
    args = parser.parse_args()

    if args.network == "localterra":
        lt = LocalTerra()
        deployer = lt.wallets["test1"]
    elif args.network == "bombay":
        lt = LCDClient(url="https://lcd.terra.dev", chain_id="bombay-12")
        mnemonic = open("mnemonic.txt", "r").read()
        mk = MnemonicKey(mnemonic=mnemonic)
        deployer = lt.wallet(mk)
    else:
        print("valid network required")
        exit(1)

    # deploy generic token
    generic_code_id = store_contract(lt, deployer, "lending_token")
    generic_token_addr = instantiate_contract(lt, deployer, generic_code_id, {
        "name": "Generic Token",
        "symbol": "GNT", 
        "decimals": 6,
        "initial_balances": [
            {"address": deployer.key.acc_address, "amount": f"{pow(10, 6)}"}
        ],
        "mint": {
            "minter": deployer.key.acc_address
        }
    })

    # deploy lending protocol contract
    lending_protocol_id = store_contract(lt, deployer, "lending_protocol")
    lending_protocol_addr = instantiate_contract(lt, deployer, lending_protocol_id, {
        "admin": deployer.key.acc_address,
        "generic_token": generic_token_addr
    })

    # deploy lending token
    lending_code_id = store_contract(lt, deployer, "lending_token")
    lending_token_addr = instantiate_contract(lt, deployer, lending_code_id, {
        "name": "Lending Token",
        "symbol": "LND", 
        "decimals": 6,
        "initial_balances": [
            {"address": deployer.key.acc_address, "amount": f"{100}"}
        ],
        "mint": {
            "minter": lending_protocol_addr
        }
    })

    # send lending token address in protocol
    results = execute_contract(
        lt, 
        deployer,
        lending_protocol_addr,
        {
            "set_lending_token_address": {
                "address": lending_token_addr
            }
        }
    )

    config = {
        "generic_token_code_id": generic_code_id,
        "generic_token_addr": generic_token_addr,
        "lending_token_code_id": lending_code_id,
        "lending_token_addr": lending_token_addr,
        "lending_protocol_code_id": lending_protocol_id,
        "lending_protocol_addr": lending_protocol_addr
    }

    pickle.dump(config, open(f"config_{args.network}.p", "wb" ))
