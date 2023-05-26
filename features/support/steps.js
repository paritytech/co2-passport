const { Given, When, Then } = require("cucumber");
const { expect } = require("chai");
const { hexToString } = require("@polkadot/util");
const fs = require("fs");

/* User Story 1 (us1) */

Given("I have the environment prepared.", async function () {
	await this.prepareEnvironment();
});

When(
	"{string} creates an asset with metadata: {string} and {string} emissions with the amount: {int} Grams per kilo CO2 emitted from date: {int}.",
	async function (caller, metadata, emission_category, emissions, date) {
		let emissionInfo = [
			{
				category: emission_category,
				primary: true,
				balanced: true,
				date: date,
				emissions: emissions,
			},
		];
		let assetParent = null;
		await this.blastAsset(caller, metadata, assetParent, emissionInfo);
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

		let emissionInfo = [
			{
				category: asset.emission_category,
				primary: true,
				balanced: true,
				date: asset.date,
				emissions: asset.emissions,
			},
		];
		let assetParent = null;

		await this.prepareEnvironment();
		await this.blastAsset(
			caller,
			asset.metadata,
			assetParent,
			emissionInfo
		);
	}
);

When(
	"{string} transfers asset with ID {int} to {string} with new {string} emission with the amount of {int} grams per kilo on the date {int}",
	async function (
		seller,
		assetId,
		buyer,
		emission_category,
		emissions,
		date
	) {
		await this.transferAsset(
			seller,
			assetId,
			buyer,
			emission_category,
			emissions,
			date
		);
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
	"The following result should be returned and the offchain calculation for {string} emissions not earlier than the date {int} should be {float}",
	function (filterCategory, filterDate, expectedEmissions, docString) {
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
			let totalAssetEmissions = 0;

			for (emissions of emissions) {
				if (emissions.date < filterDate) {
					continue;
				}

				if (emissions.category === filterCategory) {
					totalAssetEmissions += emissions.emissions;
				}
			}

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
