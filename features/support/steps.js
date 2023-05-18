const { Given, When, Then } = require("cucumber");
const { expect } = require("chai");

/* User Story 1 (us1) */

Given('I have the environment prepared.', async function () {
  await this.prepareEnvironment();
});

When('{string} creates an asset with metadata: {string} and {string} emissions with the amount: {int} Grams per kilo CO2 emitted from date: {int}.', async function (caller, metadata, emission_category, emissions, date) {
  let emissionInfo = {
    "category": emission_category,
    "primary": true,
    "balanced": true,
    "date": date,
    "emissions": emissions
  }
  let assetParent = null
  await this.blastAsset(caller, metadata, assetParent, emissionInfo);
});

Then('the following events will be emitted:', function (jsonString) {
  let events = JSON.parse(jsonString);
  expect(this.events).to.deep.equal(events);
});

Given('the {string} has blasted the asset with the following parameters:', async function (caller, jsonString) {
  let asset = JSON.parse(jsonString);

  let emissionInfo = {
    "category": asset.emission_category,
    "primary": true,
    "balanced": true,
    "date": asset.date,
    "emissions": asset.emissions
  }
  let assetParent = null

  await this.prepareEnvironment();
  await this.blastAsset(caller, asset.metadata, assetParent, emissionInfo);

});

When('{string} transfers asset with ID {int} to {string} with new {string} emission with the amount of {int} grams per kilo on the date {int}',
  async function (seller, assetId, buyer, emission_category, emissions, date) {
    await this.transferAsset(seller, assetId, buyer, emission_category, emissions, date);
  });

Then('the following transfer events will be emitted:', function (jsonString) {
  let events = JSON.parse(jsonString);
  expect(this.events).to.deep.equal(events);
});


/* User Story 2 (us2) */

Given('A {string} creates an asset that is split into child assets where the asset is defined as:', async function (caller, jsonString) {
  let asset = JSON.parse(jsonString);

  let emissionInfo = {
    "category": asset.emission_category,
    "primary": true,
    "balanced": true,
    "date": asset.date,
    "emissions": asset.emissions
  }
  let assetParent = [1, 5];

  await this.prepareEnvironment();
  await this.blastAsset(caller, asset.metadata, null, emissionInfo);
  await this.pauseAsset(caller, 1);
  await this.blastAsset(caller, asset.metadata, assetParent, emissionInfo);

});

When('{string} performs a query on the asset with ID {int}', async function (caller, assetId) {
  await this.queryEmissions(caller, 1);
});

Then('the following result should be returned', function (docString) {
  // Write code here that turns the phrase above into concrete actions
  return 'pending';
});