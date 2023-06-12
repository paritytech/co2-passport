const fs = require("fs");

const { CodePromise, ContractPromise } = require("@polkadot/api-contract");
const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { BN } = require("@polkadot/util");

const { txPaymentInfo } = require("useink/core");
const { formatBalance } = require('useink');

// require uses the path relative to the file it is called from
const CONTRACT_ABI = require("./target/ink/asset_co2_emissions.json");

// Max gas limit
const REF_TIME = new BN(300_000_000_000);
const PROOF_SIZE = new BN(1000000);

// Unlimited storage deposit
const STORAGE_DEPOSIT_LIMIT = null;

async function dryRun(contract, sender, message, ...params) {
	const { gasRequired, storageDeposit, result, output } =
		await contract.query[message](
			sender.address,
			...params
		);

	// the gas consumed for contract execution
	console.log(gasRequired.toJSON());

    console.log(storageDeposit.toJSON());

	// check if the call was successful
	if (result.isOk) {
		// output the return value
		// console.log("Success", output.toJSON().ok);
    } else {
        console.error("Error", result.asErr.toJSON());
	}
}

async function main() {
    // Shibuya RPC
    const wsProvider = new WsProvider("wss://rpc.shibuya.astar.network");

    // Rococo RPC
    // const wsProvider = new WsProvider("wss://rococo-contracts-rpc.polkadot.io");

    const api = await ApiPromise.create({ provider: wsProvider });

    // Shibuya Contract Address
    const contractAddress = "Z9RhEdrpQVvfMRXVKaVb1tpxzZkKNBt5DeUxqgQ6BXtAvuY";

    // Rococo Contract Address
    // const contractAddress = "5DG8tf9FG5ZkfqPk9kXBfVtwcq1Aaevah71KTDTTY1eGijid";
    
    const contract = new ContractPromise(api, CONTRACT_ABI, contractAddress);

	let keyring = new Keyring({ type: "sr25519" });
	const alice = keyring.addFromUri("//Alice");

	const txOptions = {
		gasLimit: api.registry.createType("WeightV2", {
			refTime: REF_TIME,
			proofSize: PROOF_SIZE,
		}),
		STORAGE_DEPOSIT_LIMIT,
    };

    // Shibuya Address
    const testingAddress = "aTpKaUyG6uFSNoctp96noX1WWQWsydJXf9p99VFaT26c9mW";
    
    // Rococo Address
    // const testingAddress = "5GbDtHWtUsG9DYmnQWgym9MjRThPukvZWTefuSHPi62927YS";

    const message = "assetCO2Emissions::blast";

    const assetOwner = "5CdTEjkVG3XKu77B5mGXBNk7fgj6GQF16WHHWF822pF18AMb";
    const assetMetadata = "{\"weight\": 1000}";
    const assetEmissions = [
        {
          "category": "Process",
          "dataSource": "Nexigen",
          "value": 12,
          "balanced": true,
          "date": 1686559847
        }
    ];
    const assetParent = null;

	await dryRun(contract, testingAddress, message, txOptions, assetOwner, assetMetadata, assetEmissions, assetParent);
}

main();
