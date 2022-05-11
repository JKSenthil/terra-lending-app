import json
import base64

from terra_sdk.client.lcd import Wallet
from terra_sdk.client.lcd.api.tx import CreateTxOptions
from terra_sdk.util.contract import read_file_as_b64, get_code_id, get_contract_address
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract

def store_contract(lt, deployer: Wallet, contract_name: str) -> str:
    """Uploads contract, returns code ID"""
    contract_bytes = read_file_as_b64(f"../artifacts/{contract_name}.wasm")
    store_code = MsgStoreCode(
        deployer.key.acc_address,
        contract_bytes
    )
    tx = deployer.create_and_sign_tx(
        CreateTxOptions(msgs=[store_code])
    )
    result = lt.tx.broadcast(tx)
    code_id = get_code_id(result)
    return code_id

def instantiate_contract(lt, deployer, code_id: str, init_msg) -> str:
    """Instantiates a new contract with code_id and init_msg, returns address"""
    instantiate = MsgInstantiateContract(
        admin=None,
        sender=deployer.key.acc_address,
        code_id=code_id,
        init_msg=init_msg
    )
    tx = deployer.create_and_sign_tx(
        CreateTxOptions(msgs=[instantiate])
    )
    result = lt.tx.broadcast(tx)
    contract_address = get_contract_address(result)
    return contract_address

def execute_contract(lt, sender: Wallet, contract_addr: str, execute_msg):
    execute = MsgExecuteContract(
        sender=sender.key.acc_address,
        contract=contract_addr,
        execute_msg=execute_msg
    )
    tx = sender.create_and_sign_tx(
        CreateTxOptions(msgs=[execute])
    )
    result = lt.tx.broadcast(tx)
    return result

def construct_binary_msg(msg):
    msg = json.dumps(msg)
    msg = base64.b64encode(msg.encode('utf-8'))
    msg = msg.decode()
    return msg