const fs = require("fs");
const { setWorldConstructor } = require("cucumber");

const { CodePromise, ContractPromise } = require("@polkadot/api-contract");
const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { BN } = require("@polkadot/util");

// require uses the path relative to the file it is called from
const contractAbi = require("../../target/ink/asset_co2_emissions.json");
// fs.readFileSync uses the path relative to the cwd of the calling process (should be root of the project)
const contract = JSON.parse(
	fs.readFileSync("./target/ink/asset_co2_emissions.contract")
);

// Max gas limit
const REF_TIME = new BN(300_000_000_000);
const PROOF_SIZE = new BN(1000000);

// Unlimited storage deposit
const STORAGE_DEPOSIT_LIMIT = null;

const INITIAL_BALANCE = 100_000_000_000_000;

// dev mnemonic
const MNENOMIC =
	"bottom drive obey lake curtain smoke basket hold race lonely fit walk";

class UserStoryWorld {
	constructor() {
		this.api = null;
		this.contract = null;
		this.codeHash = null;
		this.sudo = null;
		this.result = "";
		this.events = [];
		this.readOutput = null;
		this.accounts = {};
		this.keyring = new Keyring({ type: "sr25519" });
		this.defaultTxOptions = null;
	}

	async prepareEnvironment() {
		this.sudo = this.keyring.addFromUri("//Alice");

		const wsProvider = new WsProvider("ws://127.0.0.1:9944");
		this.api = await ApiPromise.create({ provider: wsProvider });

		this.defaultTxOptions = {
			gasLimit: this.api.registry.createType("WeightV2", {
				refTime: REF_TIME,
				proofSize: PROOF_SIZE,
			}),
			STORAGE_DEPOSIT_LIMIT,
		};

		await this.deploySmartContract(contract);
		await this.initiateAccountWithBalance("Seller", INITIAL_BALANCE);
		await this.initiateAccountWithBalance("Buyer", INITIAL_BALANCE);
		await this.initiateAccountWithBalance("Eve", INITIAL_BALANCE);
	}

	async deploySmartContract(contract) {
		const code = new CodePromise(
			this.api,
			contractAbi,
			contract.source.wasm
		);

		const tx = code.tx.new(this.defaultTxOptions);

		// Wait for the smart contract to deploy, and contract address set
		await new Promise((resolve) => {
			this.signAndSend(this.sudo, tx, async (result) => {
				this.setContractAddress(result);
				await this.setCodeHash(this.contract.address);
				resolve();
			});
		});
	}

	async setCodeHash(contractAddress) {
		const { codeHash } = (
			await this.api.query.contracts.contractInfoOf(contractAddress)
		).toHuman();
		this.codeHash = codeHash;
	}

	async upgradeContract(contract) {
		let oldContract = this.contract;
		await this.deploySmartContract(contract);

		const { gasRequired, storageDeposit } = await this.dryRun(
			this.sudo,
			"setCode",
			this.defaultTxOptions,
			this.codeHash
		);

		let txOptions = {
			gasLimit: gasRequired,
			storageDeposit,
		};

		let upgradeExtrinsic = oldContract.tx["setCode"](
			txOptions,
			this.codeHash
		);

		await this.signAndSend(this.sudo, upgradeExtrinsic, () => {});
	}

	async setContractOwner(newOwner) {
		const sender = this.accounts[newOwner];

		const assetOwner = sender.address;

		const { gasRequired, storageDeposit, output } = await this.dryRun(
			sender,
			"setContractOwner",
			this.defaultTxOptions,
			assetOwner
		);

		// Check if contract error exists
		if (output.toJSON().ok && output.toJSON().ok.err) {
			return output.toJSON().ok.err;
		}

		let txOptions = {
			gasLimit: gasRequired,
			storageDeposit,
		};

		let setOwnerExtrinsic = this.contract.tx["setContractOwner"](
			txOptions,
			assetOwner
		);

		await new Promise((resolve) => {
			this.signAndSend(sender, setOwnerExtrinsic, (result) => {
				this.events = this.getEvents(result);
				resolve();
			});
		});
	}

	async blastAsset(account_name, metadata, assetParent, assetEmissions) {
		const sender = this.accounts[account_name];

		const assetOwner = sender.address;

		// do dry run to get the required gas and storage deposit
		const { gasRequired, storageDeposit, output } = await this.dryRun(
			sender,
			"assetCO2Emissions::blast",
			this.defaultTxOptions,
			assetOwner,
			metadata,
			assetEmissions,
			assetParent
		);

		// Check if contract error exists
		if (output.toJSON().ok && output.toJSON().ok.err) {
			return output.toJSON().ok.err;
		}

		let txOptions = {
			gasLimit: gasRequired,
			storageDeposit,
		};

		let blastExtrinsic = this.contract.tx["assetCO2Emissions::blast"](
			txOptions,
			assetOwner,
			metadata,
			assetEmissions,
			assetParent
		);

		await new Promise((resolve) => {
			this.signAndSend(sender, blastExtrinsic, (result) => {
				this.events = this.getEvents(result);
				resolve();
			});
		});
	}

	async transferAsset(senderName, assetId, recipientName, emissions) {
		const sender = this.accounts[senderName];
		const receiver = this.accounts[recipientName];

		const { gasRequired, storageDeposit, output } = await this.dryRun(
			sender,
			"assetCO2Emissions::transfer",
			this.defaultTxOptions,
			receiver.address,
			assetId,
			emissions
		);

		// Check if contract error exists
		if (output.toJSON().ok && output.toJSON().ok.err) {
			return output.toJSON().ok.err;
		}

		let txOptions = {
			gasLimit: gasRequired,
			storageDeposit,
		};

		let transferExtrinsic = this.contract.tx["assetCO2Emissions::transfer"](
			txOptions,
			receiver.address,
			assetId,
			emissions
		);

		await new Promise((resolve) => {
			this.signAndSend(sender, transferExtrinsic, (result) => {
				this.events = this.getEvents(result);
				resolve();
			});
		});
	}

	async addEmission(senderName, assetId, emission) {
		const sender = this.accounts[senderName];

		const { gasRequired, storageDeposit, output } = await this.dryRun(
			sender,
			"assetCO2Emissions::addEmissions",
			this.defaultTxOptions,
			assetId,
			emission
		);

		// Check if contract error exists
		if (output.toJSON().ok && output.toJSON().ok.err) {
			return output.toJSON().ok.err;
		}

		let txOptions = {
			gasLimit: gasRequired,
			storageDeposit,
		};

		let transferExtrinsic = this.contract.tx[
			"assetCO2Emissions::addEmissions"
		](txOptions, assetId, emission);

		await new Promise((resolve) => {
			this.signAndSend(sender, transferExtrinsic, (result) => {
				this.events = this.getEvents(result);
				resolve();
			});
		});
	}

	async pauseAsset(senderName, assetId) {
		const sender = this.accounts[senderName];

		const { gasRequired, storageDeposit } = await this.dryRun(
			sender,
			"assetCO2Emissions::pause",
			this.defaultTxOptions,
			assetId
		);

		let txOptions = {
			gasLimit: gasRequired,
			storageDeposit,
		};

		let pauseExtrinsic = this.contract.tx["assetCO2Emissions::pause"](
			txOptions,
			assetId
		);

		await new Promise((resolve) => {
			this.signAndSend(sender, pauseExtrinsic, (result) => {
				this.events = this.getEvents(result);
				resolve();
			});
		});
	}

	// Helper to split an asset into the given child assets
	async createAssetTree(caller, assets) {
		for (let [i, asset] of assets.entries()) {
			let emissions = [];
			for (const emission of asset.emissions) {
				emissions.push({
					category: emission.emission_category,
					primary: true,
					balanced: true,
					date: emission.date,
					emissions: emission.emissions,
				});
			}

			let assetParent = null;

			if (i > 0) {
				assetParent = [i, asset.metadata.weight];
				await this.pauseAsset(caller, i);
			}

			await this.blastAsset(
				caller,
				JSON.stringify(asset.metadata),
				assetParent,
				emissions
			);
		}
	}

	async getOwnerOf(senderName, assetId) {
		const sender = this.accounts[senderName];

		const { result, output } = await this.getContractState(
			sender,
			"assetCO2Emissions::ownerOf",
			this.defaultTxOptions,
			assetId
		);

		if (result.isOk) {
			this.readOutput = output.toJSON().ok;
			return this.readOutput;
		}
	}

	async getAsset(senderName, assetId) {
		const sender = this.accounts[senderName];

		const { result, output } = await this.getContractState(
			sender,
			"assetCO2Emissions::getAsset",
			this.defaultTxOptions,
			assetId
		);

		if (result.isOk) {
			this.readOutput = output.toJSON().ok;
			return this.readOutput;
		}
	}

	async queryEmissions(senderName, assetId) {
		const sender = this.accounts[senderName];

		const { result, output } = await this.getContractState(
			sender,
			"assetCO2Emissions::queryEmissions",
			this.defaultTxOptions,
			assetId
		);

		if (result.isOk) {
			this.readOutput = output.toJSON().ok;
			return this.readOutput;
		}
	}

	async initiateAccountWithBalance(accountName, balance) {
		const account = this.keyring.addFromUri(MNENOMIC + "//" + accountName);
		this.accounts[accountName] = account;

		const extrinsic = this.api.tx.sudo.sudo(
			this.api.tx.balances.setBalance(account.address, balance, 0)
		);

		await this.signAndSend(this.sudo, extrinsic, () => {});
	}

	getEvents(result) {
		let events = [];
		for (const event of result.contractEvents) {
			const eventItem = {
				event: {
					name: event.event.identifier,
					args: event.args.map((v) => v.toHuman()),
				},
			};
			events.push(eventItem);
		}
		return events;
	}

	setContractAddress({ contract }) {
		const contractAddress = contract.address.toString();

		// Proper way of creating Contract JS Object
		this.contract = new ContractPromise(
			this.api,
			contractAbi,
			contractAddress
		);
	}

	async getContractState(sender, message, ...params) {
		return await this.dryRun(sender, message, ...params);
	}

	async dryRun(sender, message, ...params) {
		const o = await this.contract.query[message](sender.address, ...params);

		if (!o.result.isOk) {
			let registryErr = this.api.registry.findMetaError(
				o.result.dispatchError.asModule
			);
			throw new Error(
				`Tx failed with error: ${JSON.stringify(registryErr, null, 2)}`
			);
		}

		return o;
	}

	async signAndSend(wallet, extrinsic, callback) {
		await extrinsic.signAndSend(wallet, (result) => {
			const status = result.status;
			const dispatchError = result.dispatchError;

			if (status.isInBlock || status.isFinalized) {
				if (dispatchError) {
					let registryErr = this.api.registry.findMetaError(
						dispatchError.asModule
					);
					throw new Error(
						`Tx failed with error: ${JSON.stringify(
							registryErr,
							null,
							2
						)}`
					);
				} else {
					callback(result);
				}
			}
		});
	}
}

setWorldConstructor(UserStoryWorld);
