const { ContractPromise } = require("@polkadot/api-contract");
const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { BN } = require("@polkadot/util");

// require uses the path relative to the file it is called from
const CONTRACT_ABI = require("./target/ink/asset_co2_emissions.json");

// Max gas limit
const REF_TIME = new BN(300_000_000_000);
const PROOF_SIZE = new BN(1_000_000);
// Unlimited storage deposit
const STORAGE_DEPOSIT_LIMIT = null;

async function dryRun(contract, sender, message, ...params) {
	const { gasRequired, storageDeposit, result, output } =
		await contract.query[message](sender, ...params);

	// the gas consumed for contract execution
	console.log(gasRequired.toJSON());

	console.log(storageDeposit.toJSON());

	// check if the call was successful
	if (result.isOk) {
		// output the return value
		console.log("Success!!! : ", output.toJSON().ok);
	} else {
		console.error("Error!!! : ", result.asErr.toJSON());
	}
}

async function main() {
	// Shibuya RPC
	// const wsProvider = new WsProvider("wss://rpc.shibuya.astar.network");

	// Rococo RPC
	const wsProvider = new WsProvider("wss://rococo-contracts-rpc.polkadot.io");

	// Local Node RPC
	// const wsProvider = new WsProvider("ws://127.0.0.1:9944");

	const api = await ApiPromise.create({ provider: wsProvider });

	// Shibuya Contract Address
	// const contractAddress = "Z9RhEdrpQVvfMRXVKaVb1tpxzZkKNBt5DeUxqgQ6BXtAvuY";

	// Rococo Contract Address
	const contractAddress = "5HTY8YRLaLwhxzQPJACUiKYrceNCmrB1RgjPJgNDKn1qZp7b";

	// TODO set Local Node Contract Address Here
	// Local Node Contract Address
	// const contractAddress = "5Cip81QsAXC4iyW6V5NwDFnBHij3e9d5DqTW6AnZzdC9iWZG";
	// "5DG9oUoaWqnedg7dj964tVYHrUUsxL7C5nLoiUWTxSSitSig";

	const contract = new ContractPromise(api, CONTRACT_ABI, contractAddress);

	let keyring = new Keyring({ type: "sr25519" });
	// eslint-disable-next-line  no-unused-vars
	const alice = keyring.addFromUri("//Alice");

	const txOptions = {
		gasLimit: api.registry.createType("WeightV2", {
			refTime: REF_TIME,
			proofSize: PROOF_SIZE,
		}),
		STORAGE_DEPOSIT_LIMIT,
	};

	// Shibuya Account Address
	// const testingAddress = "aTpKaUyG6uFSNoctp96noX1WWQWsydJXf9p99VFaT26c9mW";

	// Rococo Account Address
	const testingAddress = "5GbDtHWtUsG9DYmnQWgym9MjRThPukvZWTefuSHPi62927YS";

	// Local Node Account Address
	// const testingAddress = alice.address;

	const message = "assetCO2Emissions::blast";

	const assetOwner = testingAddress;
	const assetMetadata =
		'{"weight": 1666, "batch_id": "CXW123ABCF", "item_num": "CXW123ABCF", "certified": true, "scrap_percentage": 75, "renewable_energy_mix": [{"mix": 30, "source": "hydro"}]}';

	const assetEmissions = [
		{
			category: "Process",
			dataSource: "Nexigen",
			value: 12,
			balanced: true,
			date: 1686559847,
		},
	];

	const assetParent = null;

	await dryRun(
		contract,
		testingAddress,
		"assetCO2Emissions::listAssets",
		txOptions,
		testingAddress
	);
	await dryRun(
		contract,
		testingAddress,
		message,
		txOptions,
		assetOwner,
		assetMetadata,
		assetEmissions,
		assetParent
	);
}

main();
