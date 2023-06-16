require("dotenv").config();

const { execSync } = require("child_process");

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

async function calculateFees(
	info,
	contract,
	sender,
	message,
	txOptions,
	...params
) {
	console.log(
		"\n-------------------------------------------------------------------------------------------------"
	);
	console.log("Calculating fees for:", info);
	console.log("Dry run for: { message:", message, " sender:", sender, "}");

	const { gasRequired, storageDeposit, result } = await dryRun(
		contract,
		sender,
		message,
		txOptions,
		...params
	);

	console.log("Gas Required:", gasRequired.toHuman());
	console.log("Storage Deposit:", storageDeposit.toHuman());

	if (result.isOk) {
		await doCalculateFees(contract, sender, message, txOptions, ...params);
	} else {
		console.error("Error:", result.asErr.toJSON());
	}
}

// eslint-disable-next-line  no-unused-vars
async function queryState(contract, sender, message, txOptions, ...params) {
	console.log(
		"\n-------------------------------------------------------------------------------------------------"
	);
	console.log(
		"Querying state for: { message:",
		message,
		" sender:",
		sender,
		"}"
	);

	const { result, output } = await dryRun(
		contract,
		sender,
		message,
		txOptions,
		...params
	);
	if (result.isOk) {
		console.log("Output:", output.toJSON().ok);
	} else {
		console.error("Error:", result.asErr.toJSON());
	}
}

async function dryRun(contract, sender, message, txOptions, ...params) {
	return await contract.query[message](sender, txOptions, ...params);
}

async function callContract(
	info,
	contract,
	sender,
	message,
	txOptions,
	...params
) {
	console.log(
		"\n-------------------------------------------------------------------------------------------------"
	);
	console.log("Calling contract:", info);
	let extrinsic = contract.tx[message](txOptions, ...params);
	await signAndSend(sender, extrinsic);
}

async function waitForBlock() {
	console.log("Waiting for a block for 15 seconds...");
	execSync("sleep 15");
}

async function signAndSend(wallet, extrinsic) {
	await extrinsic.signAndSend(wallet, (result) => {
		const status = result.status;
		const dispatchError = result.dispatchError;

		if (status.isInBlock || status.isFinalized) {
			if (dispatchError) {
				console.log("Contract Call error:", result.asErr.toJSON());
			} else {
				console.log("Contract Call successfull!");
			}
		}
	});
}

async function doCalculateFees(
	contract,
	sender,
	message,
	txOptions,
	...params
) {
	const extrinsic = contract.tx[message](txOptions, ...params);
	const result = await extrinsic.paymentInfo(sender);
	console.log("Expected txn fee:", result.partialFee.toHuman());
}

async function main() {
	const endpoint = process.env.NODE_RPC;
	const wsProvider = new WsProvider(endpoint);
	const api = await ApiPromise.create({ provider: wsProvider });

	let keyring = new Keyring({ type: "sr25519" });
	// eslint-disable-next-line  no-unused-vars
	const account = keyring.addFromUri("//Bob");

	const contractAddress = process.env.CONTRACT_ADDRESS;
	const contract = new ContractPromise(api, CONTRACT_ABI, contractAddress);

	const txOptions = {
		gasLimit: api.registry.createType("WeightV2", {
			refTime: REF_TIME,
			proofSize: PROOF_SIZE,
		}),
		STORAGE_DEPOSIT_LIMIT,
	};

	const testingAddress = process.env.TESTING_ADDRESS;

	const blastMessage = "assetCO2Emissions::blast";
	const transferMessage = "assetCO2Emissions::transfer";
	const pauseMessage = "assetCO2Emissions::pause";
	const addEmissionsMessage = "assetCO2Emissions::addEmissions";

	const assetOwner = testingAddress;
	const assetMetadata =
		"1234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678";
	const CO2EmissionDataSource =
		"12345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678123456781234567812345678";
	const CO2EmissionItem = {
		category: "Process",
		dataSource: CO2EmissionDataSource,
		value: 1000,
		balanced: true,
		date: 1686559847,
	};

    const pausedAssetId = 1;
    const assetId = 2;

	const parent = {
		parentId: pausedAssetId,
		relation: 1000,
    };

	await calculateFees(
		"Blast, single emission, no parent",
		contract,
		testingAddress,
		blastMessage,
		txOptions,
		assetOwner,
		assetMetadata,
		[CO2EmissionItem],
		null
	);

	await calculateFees(
		"Blast, 2 emissions, no parent",
		contract,
		testingAddress,
		blastMessage,
		txOptions,
		assetOwner,
		assetMetadata,
		[CO2EmissionItem, CO2EmissionItem],
		null
	);

	await calculateFees(
		"Blast, 3 emissions, no parent",
		contract,
		testingAddress,
		blastMessage,
		txOptions,
		assetOwner,
		assetMetadata,
		[CO2EmissionItem, CO2EmissionItem, CO2EmissionItem],
		null
	);

	await calculateFees(
		"Blast, single emission, parent specified",
		contract,
		testingAddress,
		blastMessage,
		txOptions,
		assetOwner,
		assetMetadata,
		[CO2EmissionItem],
		parent
	);

	await calculateFees(
		"Blast, 2 emissions, parent specified",
		contract,
		testingAddress,
		blastMessage,
		txOptions,
		assetOwner,
		assetMetadata,
		[CO2EmissionItem, CO2EmissionItem],
		parent
	);

	await calculateFees(
		"Blast, 3 emissions, parent specified",
		contract,
		testingAddress,
		blastMessage,
		txOptions,
		assetOwner,
		assetMetadata,
		[CO2EmissionItem, CO2EmissionItem, CO2EmissionItem],
		parent
	);


	await calculateFees(
		"Add Emissions",
		contract,
		testingAddress,
		addEmissionsMessage,
		txOptions,
		assetId,
		CO2EmissionItem
	);

	await calculateFees(
		"Transfer, 1 emission",
		contract,
		testingAddress,
		transferMessage,
		txOptions,
		"bgrbE6JxaHBk2tS7JH4rjcicyMki4t1E9M6YivFzCUq2YDg",
		assetId,
		[CO2EmissionItem]
	);

	await calculateFees(
		"Transfer, 2 emissions",
		contract,
		testingAddress,
		transferMessage,
		txOptions,
		"bgrbE6JxaHBk2tS7JH4rjcicyMki4t1E9M6YivFzCUq2YDg",
		assetId,
		[CO2EmissionItem, CO2EmissionItem]
	);
	await calculateFees(
		"Transfer, 3 emission",
		contract,
		testingAddress,
		transferMessage,
		txOptions,
		"bgrbE6JxaHBk2tS7JH4rjcicyMki4t1E9M6YivFzCUq2YDg",
		assetId,
		[CO2EmissionItem, CO2EmissionItem, CO2EmissionItem]
	);

	await calculateFees(
		"Pause",
		contract,
		testingAddress,
		pauseMessage,
		txOptions,
		assetId
	);

}

main();
