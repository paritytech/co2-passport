const { Given, When, Then } = require("cucumber");
const { expect } = require("chai");

 Given('I have the environment prepared.', async function () {
     await this.prepareEnvironment();
 });

 When('{string} creates an asset with metadata: {string} and {string} emissions with the amount: {int} Grams per kilo CO2 emitted from date: {int}.', async function (caller, metadata, emission_category, emissions, date) {
     await this.blastAsset(caller, metadata, emission_category, emissions, date);
 });

 Then('the following events will be emitted:', function (jsonString) {
    let events = JSON.parse(jsonString);
    expect(this.events).to.deep.equal(events);
 });

 Given('the {string} has blasted the asset with the following parameters:', async function (caller, jsonString) {
    let asset = JSON.parse(jsonString);

    await this.prepareEnvironment();
    await this.blastAsset(caller, asset.metadata, asset.emission_category, asset.emissions, asset.date);

  });

  When('{string} transfers asset with ID {int} to {string} with new {string} emission with the amount of {int} grams per kilo on the date {int}',
  async function (seller, assetId, buyer, emission_category, emissions, date) {
        await this.transferAsset(seller, assetId, buyer, emission_category, emissions, date);
  });

  Then('the following transfer events will be emitted:', function (jsonString) {
    let events = JSON.parse(jsonString);
    expect(this.events).to.deep.equal(events);
  });