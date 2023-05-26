const { Given, When, Then } = require("cucumber");
const { expect } = require("chai");
const { hexToString } = require("@polkadot/util");
const fs = require("fs");

/* User Story 1 (us1) */

Given("I have the environment prepared.", async function () {
	await this.prepareEnvironment();
});

When(
	"{string} blasts an asset defined as the following:",
	async function (caller, jsonString) {
		let asset = JSON.parse(jsonString);
		let assetParent = null;
		await this.blastAsset(
			caller,
			JSON.stringify(asset.metadata),
			assetParent,
			asset.emissions
		);
	}
);

Then(
	"The asset {int} and emitted events will be the following:",
	async function (assetId, jsonString) {
		let { asset, events } = JSON.parse(jsonString);
		let assetDetails = await this.getAsset("Eve", assetId);

		expect(assetDetails).to.deep.equal(asset);
		expect(this.events).to.deep.equal(events);
	}
);

Given(
	"The {string} has blasted the asset with the following parameters:",
	async function (caller, jsonString) {
		let asset = JSON.parse(jsonString);

		let assetParent = null;

		await this.prepareEnvironment();
		await this.blastAsset(
			caller,
			asset.metadata,
			assetParent,
			asset.emissions
		);
	}
);

When(
	"{string} transfers asset with ID {int} to {string} with emissions of:",
	async function (seller, assetId, buyer, jsonString) {
		let emissions = JSON.parse(jsonString);
		await this.transferAsset(seller, assetId, buyer, emissions);
	}
);

Then(
	"{string} will be the new owner of asset {int}, the emissions and transfer events will be the following:",
	async function (sender, assetId, jsonString) {
		let { emissions, events } = JSON.parse(jsonString);

		expect(this.events).to.deep.equal(events);

		let ownerOf = await this.getOwnerOf(sender, assetId);
		let assetDetails = await this.getAsset(sender, assetId);
		expect(ownerOf).to.equal(this.accounts[sender].address);
		expect(assetDetails[2]).to.deep.equal(emissions);
	}
);

Given(
	"The {string} has blasted the following asset:",
	async function (caller, jsonString) {
		let asset = JSON.parse(jsonString);

		let assetParent = null;

		await this.prepareEnvironment();
		await this.blastAsset(
			caller,
			asset.metadata,
			assetParent,
			asset.emissions
		);
	}
);

When(
	"{string} adds the following emission to the asset with ID {int}:",
	async function (seller, assetId, jsonString) {
		let emission = JSON.parse(jsonString);

		await this.addEmission(seller, assetId, emission);
	}
);

Then("The asset {int} will be:", async function (assetId, jsonString) {
	let { emissions, events } = JSON.parse(jsonString);

	let assetDetails = await this.getAsset("Eve", assetId);

	expect(this.events).to.deep.equal(events);
	expect(assetDetails[2]).to.deep.equal(emissions);
});

/* User Story 2 (us2) */

Given(
	"A {string} creates an asset that is split into child assets where the assets are defined as:",
	async function (caller, jsonString) {
		let assets = JSON.parse(jsonString);

		await this.prepareEnvironment();
		await this.createAssetTree(caller, assets);
	}
);

When(
	"{string} performs a query on the asset with ID {int}",
	async function (caller, assetId) {
		await this.queryEmissions(caller, assetId);
	}
);
Then(
	"The emissions can be calculated offchain for {string} emissions between the dates {int} and {int} with the total equal to {float} based on the following:",
	function (
		filterCategory,
		filterDateFrom,
		filterDateTo,
		expectedEmissions,
		docString
	) {
		expect(this.readOutput).to.deep.equal(JSON.parse(docString));

		// The recursive formula is defined as:
		// let e_n = the emissions of the asset at the nth level
		// let w_n = the weight of the asset at nth level
		// let Te_n = the total emissions of the asset at the nth level
		// Te_0 = e_0
		// Te_1 = (w_1 / w_0) * Te_0 + e_1
		// Te_2 = (w_2 / w_1) * Te_1 + e_2
		// ...
		// Te_n = (w_n / w_n-1) * Te_n-1 + e_n

		let totalEmissions = 0;
		let prevWeight = 0;
		for (let asset of this.readOutput.reverse()) {
			let metadata = JSON.parse(hexToString(asset[1]));
			let emissions = asset[2];

			// Calculate total emissions for the current asset.
			// Removes emissions outside the date range of filterDateFrom and filterDateTo
			// and filters by category
			const totalAssetEmissions = emissions
				.filter((emission) => {
					return (
						emission.date > filterDateFrom &&
						emission.date < filterDateTo &&
						emission.category === filterCategory
					);
				})
				.reduce((total, emission) => total + emission.emissions, 0);

			// base case
			if (asset[3] === null) {
				totalEmissions = totalAssetEmissions;
				prevWeight = metadata.weight;
				continue;
			}
			// ratio / relation of child from parent
			let r = metadata.weight / prevWeight;
			totalEmissions = r * totalEmissions + totalAssetEmissions;
			prevWeight = metadata.weight;
		}

		expect(totalEmissions).to.equal(expectedEmissions);
	}
);

/* User Story 3 (us3) */
Given("The original contract is deployed", async function () {
	await this.prepareEnvironment();
});

When("The contract owner updgrades the contract", async function () {
	const contract = JSON.parse(
		fs.readFileSync(
			"./integration-tests/updated-contract/target/ink/updated_contract.contract"
		)
	);
	await this.deploySmartContract(contract);
	await this.upgradeContract(contract);
});

Then("The contract will be upgraded", async function () {
	// This should return `AlreadyPaused` error to showcase the contract upgraded
	await this.setContractOwner("Seller");
	expect(this.readOutput.err).to.equal("AlreadyPaused");
});
