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
		await this.initiateAccountWithBalance("Seller", 100_000_000_000_000);
		await this.initiateAccountWithBalance("Buyer", 100_000_000_000_000);
		await this.initiateAccountWithBalance("Eve", 100_000_000_000_000);
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

		const { result, output } = await this.dryRun(
			sender,
			"setContractOwner",
			this.defaultTxOptions,
			assetOwner
		);

		if (result.isOk) {
			this.readOutput = output.toJSON().ok;
		}
	}

	async blastAsset(account_name, metadata, assetParent, emissionInfo) {
		const sender = this.accounts[account_name];

		const assetOwner = sender.address;
		const assetEmissions = [
			{
				category: emissionInfo.category,
				primary: emissionInfo.primary,
				balanced: emissionInfo.balanced,
				date: emissionInfo.date,
				emissions: emissionInfo.emissions,
			},
		];

		// do dry run to get the required gas and storage deposit
		const { gasRequired, storageDeposit } = await this.dryRun(
			sender,
			"assetCO2Emissions::blast",
			this.defaultTxOptions,
			assetOwner,
			metadata,
			assetEmissions,
			assetParent
		);

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

	async transferAsset(
		senderName,
		assetId,
		recipientName,
		emissionCategory,
		emissions,
		date
	) {
		const sender = this.accounts[senderName];
		const receiver = this.accounts[recipientName];

		const assetEmissions = [
			{
				category: emissionCategory,
				primary: true,
				balanced: true,
				date: date,
				emissions: emissions,
			},
		];

		const { gasRequired, storageDeposit } = await this.dryRun(
			sender,
			"assetCO2Emissions::transfer",
			this.defaultTxOptions,
			receiver.address,
			assetId,
			assetEmissions
		);

		let txOptions = {
			gasLimit: gasRequired,
			storageDeposit,
		};

		let transferExtrinsic = this.contract.tx["assetCO2Emissions::transfer"](
			txOptions,
			receiver.address,
			assetId,
			assetEmissions
		);

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

	async queryEmissions(senderName, assetId) {
		const sender = this.accounts[senderName];

		const { result, output } = await this.dryRun(
			sender,
			"assetCO2Emissions::queryEmissions",
			this.defaultTxOptions,
			assetId
		);

		if (result.isOk) {
			this.readOutput = output.toJSON().ok;
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
