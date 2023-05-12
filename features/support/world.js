const fs = require('fs');
const { setWorldConstructor } = require("cucumber");

const { CodePromise, ContractPromise } = require("@polkadot/api-contract");
const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { mnemonicGenerate } = require("@polkadot/util-crypto")
const { BN, BN_ONE } = require("@polkadot/util");

const contractAbi = require("/Users/bidzyyys/work/bcg/bcg-co2-passport/target/ink/asset_co2_emissions.json");
const contract = JSON.parse(fs.readFileSync("/Users/bidzyyys/work/bcg/bcg-co2-passport/target/ink/asset_co2_emissions.contract"));

const REF_TIME = new BN(327_248_002);
const PROOF_SIZE = new BN(100);

class UserStoryWorld {
    constructor() {
        this.api = null;
        this.contract = null;
        this.sudo = null;
        this.firstOperand = '';
        this.secondOperand = '';
        this.result = '';
        this.accounts = {};
        this.keyring = new Keyring({ type: 'sr25519' });
        this.sendTxOptions = null;
    }

    async prepareEnvironment() {
        this.sudo = this.keyring.addFromUri("//Alice");

        const wsProvider = new WsProvider("ws://127.0.0.1:9944");
        this.api = await ApiPromise.create({ provider: wsProvider });

        const storageDepositLimit = null;
        this.sendTxOptions = {
            gasLimit: this.api.registry.createType("WeightV2", {
                refTime: REF_TIME,
                proofSize: PROOF_SIZE
            }),
            storageDepositLimit
        };

        await this.deploySmartContract();
        await this.initiateAccountWithBalance("Seller", 1_000_000_000);
        await this.initiateAccountWithBalance("Buyer", 1_000_000_000);
        await this.initiateAccountWithBalance("Eve", 1_000_000_000);

        this.setFirstOperand("I");
    }

    async deploySmartContract() {
        const code = new CodePromise(this.api, contractAbi, contract.source.wasm);

        const tx = code.tx.new(this.sendTxOptions);

        await this.signAndSend(this.sudo, tx, result => this.setContractAddress(result));
    }

    async blastAsset(account_name, metadata, emission_category, emissions, date) {
        const sender = this.accounts[account_name];

        const assetOwner = sender.address;
        const assetParent = null;
        const assetEmissions = [
            {
                "category": emissions_category,
                "primary": true,
                "balanced": true,
                "date": date,
                "emissions": emissions
            }
        ];

        let blastExtrinsic = this.contract.tx["assetCO2Emissions::blast"](
            this.sendTxOptions,
            owner, metadata, emissions, parent
        );

        signAndSend(sender, tx, doNothing);
    }

    async initiateAccountWithBalance(account_name, balance) {
        const account = this.keyring.addFromUri(mnemonicGenerate());
        this.accounts[account_name] = account;

        const extrinsic = this.api.tx.sudo.sudo(this.api.tx.balances.setBalance(account.address, balance, 0));

        await this.signAndSend(this.sudo, extrinsic, this.doNothing);
    }

    setFirstOperand(number) {
        this.firstOperand = number;
    }

    addTo(operand) {
        this.secondOperand = operand;
        this.result = 'II';
    }

    doNothing(result) {
        console.log(result);
    }

    setContractAddress( { contract } ) {
        const contractAddress = contract.address.toString();

        // Proper way of creating Contract JS Object
        this.contract = new ContractPromise(this.api, contractAbi, contractAddress);
    }

    async signAndSend(wallet, extrinsic, callback) {
        await extrinsic.signAndSend(wallet, (result) => {
            const status = result.status;
            const dispatchError = result.dispatchError;

            if (status.isInBlock || status.isFinalized) {
                if (dispatchError) {
                    throw new Error(`Tx failed with error: ${dispatchError.toJSON()}`);
                }  else {
                    callback(result);
                }
            }
        });
    }
}

setWorldConstructor(UserStoryWorld);
